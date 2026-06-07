//! Pipeline coordinator: drain audio ringbuffer on an OS thread, run wake/VAD, queue STT jobs,
//! and emit stdout JSONL transcript events.
//!
//! Thread topology (locked):
//! - CPAL RT callback: push samples to ringbuf (no alloc/no block)
//! - Pipeline thread (OS): drain ringbuf; run wake + VAD segmentation; enqueue utterance jobs
//! - STT thread (OS): whisper-rs inference
//! - Output thread (OS): write JSONL to stdout (only stdout writer)

use anyhow::{Context, Result};
use ringbuf::traits::Consumer;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

use ringbuf::HeapCons;
use crate::config::{Config, PipelineVerbosity};
use crate::control::protocol::ActivationMode;
use crate::pipeline::dispatcher::DispatchOutcome;
use crate::pipeline::jsonl::{JsonlVerbosity, JsonlWriter};
use crate::stt::{SttResult, SttService};
// Alias for readability: this module builds segmentation configs, not generic VAD configs.
use crate::vad::{VadConfig as SegCfg, VadSegmenter, FRAME_SAMPLES};
use crate::wake::WakeWord;

/// Commands sent from the control handler to the running pipeline coordinator.
///
/// Delivered via `std::sync::mpsc`; drained non-blocking (`try_recv`) at the top
/// of each pipeline frame so live changes land between utterances without restart.
#[derive(Debug)]
pub enum RuntimeCommand {
    /// Switch activation mode (PTT ↔ Wake) live.
    SetMode(ActivationMode),
    /// Update the dispatcher confidence threshold live.
    SetThreshold(f32),
    /// Request an input-device change — requires daemon restart; logged and ignored this slice.
    SetInputDevice(Option<String>),
    /// Request a PTT-binding change — requires daemon restart; logged and ignored this slice.
    SetPttBinding(String),
    /// Reload config from disk and apply hot-reloadable fields.
    ReloadConfig,
}

/// Messages flowing from the dispatcher thread to the output thread.
enum OutputMsg {
    Utterance(SttResult),
    Dispatched { utterance_id: u64, macro_id: String, score: f32 },
    NoMatch { utterance_id: u64, transcript: String },
}

/// Handles returned by [`spawn_pipeline`] so the caller can join threads on shutdown.
pub struct PipelineHandles {
    /// OS thread running the audio drain, VAD, and wake-word loop.
    pub pipeline: std::thread::JoinHandle<()>,
    /// OS thread that owns the stdout JSONL writer (sole writer to stdout).
    pub output: std::thread::JoinHandle<()>,
    /// STT service wrapper (holds the whisper-rs inference thread); `None` when STT is disabled.
    pub stt: Option<SttService>,
    /// Shared dispatcher handle — allows live macro registry updates via the control server.
    pub dispatcher: Arc<crate::pipeline::dispatcher::Dispatcher>,
}

/// Spawn the pipeline worker threads and return handles to each.
///
/// ## Thread topology
///
/// ```text
/// CPAL RT callback ──► ringbuf (lock-free) ──► pipeline thread
///                                                    │
///                                             VAD segmenter (Silero)
///                                             wake-word detector
///                                                    │ UtteranceJob
///                                                    ▼
///                                              STT thread (whisper-rs)
///                                                    │ SttResult
///                                                    ▼
///                                           dispatcher thread (phrase match + macro inject)
///                                                    │ OutputMsg
///                                                    ▼
///                                            output thread (stdout JSONL, sole writer)
/// ```
///
/// All cross-thread queues are bounded `crossbeam_channel` channels; the VAD→STT queue
/// uses drop-oldest semantics under back-pressure to prevent unbounded audio buffering.
///
/// ## CPAL stream ownership
///
/// CRITICAL: do NOT pass the full `AudioHandle` here — callers must keep the
/// `StreamGuard` on the main thread (see `src/audio/mod.rs` for rationale).
/// Only the ringbuf `consumer` end should be forwarded into the worker.
pub fn spawn_pipeline(
    audio_consumer: HeapCons<f32>,
    config: Config,
    ptt_active: Arc<AtomicBool>,
    muted: Arc<AtomicBool>,
    macro_tx: std::sync::mpsc::Sender<crate::input::inject::MacroCmd>,
    shutdown: CancellationToken,
    runtime_rx: std::sync::mpsc::Receiver<RuntimeCommand>,
) -> Result<PipelineHandles> {
    let listen_window = Duration::from_secs(config.pipeline.listen_window_secs);
    // Wake pre-roll: keep a short rolling buffer of audio while idle so the first
    // post-wake command doesn't lose its leading syllables.
    //
    // NOTE: this is separate from VAD's own preroll (which only applies once
    // LISTENING is active). This buffer is used to "seed" the VAD segmenter at
    // the moment wake triggers.
    const WAKE_PREROLL_MS: u64 = 600;
    let wake_preroll_frames: usize = WAKE_PREROLL_MS.div_ceil(20) as usize;

    // Build VAD config from config file.
    let ms_to_frames = |ms: u64| ms.div_ceil(20).max(1) as usize;
    let seg_cfg = SegCfg {
        start_threshold: config.vad.start_threshold,
        stop_threshold: config.vad.stop_threshold,
        onset_window_frames: ms_to_frames(config.vad.onset_window_ms),
        min_speech_frames: ms_to_frames(config.vad.min_speech_ms),
        end_silence_frames: ms_to_frames(config.vad.end_silence_ms),
        preroll_frames: config.vad.preroll_ms.div_ceil(20) as usize,
        tail_frames: config.vad.tail_ms.div_ceil(20) as usize,
        max_utterance_frames: (config.vad.max_utterance_secs as usize) * 50, // 50 frames/sec
    };

    // Wake mode wants “snappy” commands, not conservative segmentation.
    // Override a few knobs inside the wake LISTENING window to reduce end-of-speech
    // latency. Keep thresholds same as base VAD config.
    //
    // Targets:
    // - end_silence_ms smaller to cut earlier after last word
    // - min_speech_ms smaller to accept short commands
    // - tail/preroll smaller (wake pre-roll already handled separately)
    // - cap utterance length smaller for wake commands
    const WAKE_END_SILENCE_MS: u64 = 150;
    const WAKE_MIN_SPEECH_MS: u64 = 60;
    const WAKE_VAD_PREROLL_MS: u64 = 80;
    const WAKE_VAD_TAIL_MS: u64 = 80;
    const WAKE_MAX_UTTERANCE_SECS: usize = 4;
    const WAKE_FORCE_FLUSH_MS: u64 = 1200;
    let seg_cfg_wake = SegCfg {
        start_threshold: seg_cfg.start_threshold,
        stop_threshold: seg_cfg.stop_threshold,
        // Wake commands are short; use a tighter onset window to trigger faster.
        onset_window_frames: ms_to_frames(WAKE_MIN_SPEECH_MS).max(2),
        min_speech_frames: ms_to_frames(WAKE_MIN_SPEECH_MS),
        end_silence_frames: ms_to_frames(WAKE_END_SILENCE_MS),
        preroll_frames: ms_to_frames(WAKE_VAD_PREROLL_MS),
        tail_frames: ms_to_frames(WAKE_VAD_TAIL_MS),
        max_utterance_frames: WAKE_MAX_UTTERANCE_SECS * 50,
    };

    // Output thread owns stdout writer and ensures stdout remains JSONL-only.
    let (out_tx, out_rx) = crossbeam_channel::bounded::<OutputMsg>(16);

    // Dispatcher thread receives STT results, runs macros, then forwards to JSONL output
    let (dispatch_tx, dispatch_rx) = crossbeam_channel::bounded::<SttResult>(8);
    let dispatch_rx_for_drop = dispatch_rx.clone();

    let dispatcher = Arc::new(crate::pipeline::dispatcher::Dispatcher::new(
        config.stt.confidence_threshold,
        config.macros.clone(),
        macro_tx,
        config.timing.dwell_ms,
        config.timing.gap_ms,
    ));
    let dispatcher_for_thread = Arc::clone(&dispatcher);
    // Separate clone for the pipeline thread's runtime-command drain.
    let dispatcher_for_pipeline = Arc::clone(&dispatcher);
    let dispatcher_shutdown = shutdown.clone();
    let dispatch_out_tx = out_tx.clone();
    std::thread::spawn(move || {
        tracing::info!("Dispatcher thread started");
        while !dispatcher_shutdown.is_cancelled() {
            match dispatch_rx.recv_timeout(Duration::from_millis(50)) {
                Ok(r) => {
                    let utterance_id = r.utterance_id;
                    let transcript = r.text.clone();
                    match dispatcher_for_thread.process(&r.text) {
                        DispatchOutcome::Fired { macro_id, score } => {
                            let _ = dispatch_out_tx.send(OutputMsg::Dispatched { utterance_id, macro_id, score });
                        }
                        DispatchOutcome::NoMatch => {
                            let _ = dispatch_out_tx.send(OutputMsg::NoMatch { utterance_id, transcript });
                        }
                    }
                    let _ = dispatch_out_tx.send(OutputMsg::Utterance(r));
                }
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
            }
        }
        tracing::info!("Dispatcher thread stopped");
    });

    let jsonl_verbosity = match config.pipeline.verbosity {
        PipelineVerbosity::Summary => JsonlVerbosity::SummaryOnly,
        PipelineVerbosity::Stages => JsonlVerbosity::Stages,
    };

    let output_shutdown = shutdown.clone();
    let output = std::thread::spawn(move || {
        let stdout = std::io::stdout();
        let mut w = JsonlWriter::new(stdout.lock(), jsonl_verbosity);

        // Emit a startup event to force a flush and verify stdout is working (D-19).
        if let Err(e) = w.write_event(&crate::pipeline::jsonl::JsonlEvent::Status {
            message: "pipeline_started",
            mono_ms: 0,
        }) {
            tracing::error!("Failed to write startup JSONL: {e}");
        }

        while !output_shutdown.is_cancelled() {
            match out_rx.recv_timeout(Duration::from_millis(50)) {
                Ok(OutputMsg::Utterance(mut r)) => {
                    r.timings.mark_output_done();
                    if let Err(e) = w.write_utterance(
                        r.utterance_id,
                        &r.text,
                        r.audio_ms,
                        r.stt_ms,
                        r.timings,
                        r.start_frame_idx,
                        r.end_frame_idx,
                    ) {
                        tracing::error!("Failed to write stdout JSONL: {e}");
                    }
                }
                Ok(OutputMsg::Dispatched { utterance_id, macro_id, score }) => {
                    if let Err(e) = w.write_dispatch(utterance_id, &macro_id, score) {
                        tracing::error!("Failed to write dispatch JSONL: {e}");
                    }
                }
                Ok(OutputMsg::NoMatch { utterance_id, transcript }) => {
                    if let Err(e) = w.write_no_match(utterance_id, &transcript) {
                        tracing::error!("Failed to write no_match JSONL: {e}");
                    }
                }
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
            }
        }
    });

    // Build the Whisper initial_prompt: use the explicit config value if set, otherwise
    // auto-generate from the active macro phrase list so Whisper biases toward known vocabulary.
    // A comma-separated phrase list dramatically reduces hallucinations on short commands.
    let effective_initial_prompt: Option<String> = config.stt.initial_prompt.clone().or_else(|| {
        let phrases: Vec<&str> = config
            .macros
            .iter()
            .filter_map(|m| m.phrase.as_deref())
            .filter(|p| !p.is_empty())
            .collect();
        if phrases.is_empty() {
            None
        } else {
            let prompt = phrases.join(", ");
            tracing::info!(phrase_count = phrases.len(), "STT initial_prompt auto-built from active pack phrases");
            Some(prompt)
        }
    });

    // STT service (optional, feature-gated) is created before threads (fail-fast).
    let stt: Option<SttService> = if config.stt.enabled {
        let model_path = config
            .stt
            .model_path
            .as_ref()
            .context("stt.enabled but stt.model_path is not set")?;

        #[cfg(not(feature = "stt"))]
        {
            let _ = model_path;
            let _ = &effective_initial_prompt;
            return Err(anyhow::anyhow!(
                "Config enables STT, but this build was compiled without `--features stt`."
            ));
        }

        #[cfg(feature = "stt")]
        {
            Some(
                SttService::new(model_path, effective_initial_prompt, shutdown.clone())
                    .context("create STT service")?
                    .spawn()
                    .context("spawn STT thread")?,
            )
        }
    } else {
        None
    };

    let stt_submit = stt.as_ref().map(|s| s.submitter());
    let stt_results = stt.as_ref().map(|s| s.result_receiver());

    // Wake word (optional). Construct before pipeline loop (fail-fast).
    let wake = if config.wake.enabled {
        Some(WakeWord::new(&config.wake).context("init wake word")?)
    } else {
        None
    };

    // ORT_DYLIB_PATH auto-discovery: point the `ort` crate at the same
    // libonnxruntime.so that sherpa-onnx ships (shared feature), so both
    // runtimes share one ORT global environment instead of colliding.
    // Respect any existing user-supplied value.
    if std::env::var_os("ORT_DYLIB_PATH").is_none() {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(exe_dir) = exe.parent() {
                // Check next to the binary first (dev/cargo build), then ../lib/
                // (AppImage layout: binary in usr/bin/, .so in usr/lib/).
                // Only set if the file actually exists — fall back to LD_LIBRARY_PATH otherwise.
                let candidates = [
                    exe_dir.join("libonnxruntime.so"),
                    exe_dir.join("../lib/libonnxruntime.so"),
                ];
                if let Some(so_path) = candidates.iter().find(|p| p.exists()) {
                    // Canonicalize to remove any ".." components — ort's load-dynamic
                    // treats ORT_DYLIB_PATH as absolute and skips existence checks, so
                    // a path with ".." can silently fail on some FUSE implementations.
                    let resolved = so_path.canonicalize().unwrap_or_else(|_| so_path.clone());
                    // SAFETY: single-threaded at this point — no pipeline threads spawned yet.
                    unsafe { std::env::set_var("ORT_DYLIB_PATH", &resolved) };
                    tracing::info!(path = %resolved.display(), "ORT_DYLIB_PATH auto-set to sherpa-onnx shared library");
                }
            }
        }
    }

    // VAD model is required for utterance segmentation (CPU-only baseline, D-16).
    //
    // IMPORTANT: `ort` (ONNX Runtime) can panic at runtime if `libonnxruntime.so`
    // is not discoverable. Convert that into a normal, actionable startup error
    // instead of crashing the process.
    let start_vad = std::time::Instant::now();
    let mut silero = std::panic::catch_unwind(|| {
        match find_silero_model() {
            Some(path) => silero_vad_rust::silero_vad::model::OnnxModel::from_path(path, true),
            // Dev builds: fall back to crate's own resolver (uses CARGO_MANIFEST_DIR, valid in-source)
            None => silero_vad_rust::silero_vad::model::load_silero_vad_with_options(
                silero_vad_rust::silero_vad::model::LoadOptions {
                    force_onnx_cpu: true,
                    ..Default::default()
                },
            ),
        }
    })
    .map_err(|_| {
        anyhow::anyhow!(
            "Failed to load Silero VAD: ONNX Runtime could not be loaded.\n\
             Fix: install ONNX Runtime so `libonnxruntime.so` is in your library path, or set ORT_DYLIB_PATH to the full path of `libonnxruntime.so`.\n\
             Example (Arch/CachyOS): `sudo pacman -S onnxruntime`"
        )
    })?
    .context("load silero VAD model")?;
    tracing::info!("Silero VAD loaded in {}ms", start_vad.elapsed().as_millis());

    // NOTE: the CPAL stream guard is intentionally NOT in this function — the
    // caller (main) keeps it alive. We only own the ringbuf consumer end.
    let pipeline_shutdown = shutdown.clone();

    let pipeline = std::thread::spawn(move || {
        tracing::info!("Pipeline thread started");

        let mut seg = VadSegmenter::new(seg_cfg.clone());
        let mut consumer = audio_consumer;

        let mut pending = Vec::<f32>::with_capacity(FRAME_SAMPLES * 8);
        let mut pending_idx: usize = 0;
        let mut tmp = [0.0f32; 1024];
        let mut frame = [0.0f32; FRAME_SAMPLES];

        let mut listening_until: Option<Instant> = None;
        let mut listening_started_at: Option<Instant> = None;
        let mut prev_ptt = false;

        // Rolling wake pre-roll ring: N frames of 20ms each.
        let mut wake_preroll: Vec<[f32; FRAME_SAMPLES]> =
            vec![[0.0; FRAME_SAMPLES]; wake_preroll_frames.max(1)];
        let mut wake_preroll_pos: usize = 0; // next write index
        let mut wake_preroll_len: usize = 0; // valid frames in ring (<= capacity)

        // Heartbeat: log samples/frames received every 5s to confirm audio is flowing.
        let mut heartbeat_samples: usize = 0;
        let mut heartbeat_last = Instant::now();

        // PTT-mode direct capture: all audio while PTT is held lands here.
        // On release the entire buffer is sent to STT unconditionally, bypassing
        // the VAD speech-gate. VAD is still used for the wake-word listen window.
        let mut ptt_audio: Vec<f32> = Vec::with_capacity(FRAME_SAMPLES * 200); // ~4 sec
        let mut ptt_utterance_id: u64 = 0;
        let mut ptt_next_id: u64 = 1; // monotonic ID counter for PTT utterances
        let mut ptt_timings = crate::pipeline::timing::UtteranceTimings::new();
        let mut ptt_start_frame: u64 = 0;
        let mut ptt_frame_count: u64 = 0;

        // Current activation mode; defaults to PTT. Changed live via RuntimeCommand.
        let mut active_mode = ActivationMode::Ptt;

        while !pipeline_shutdown.is_cancelled() {
            // Drain runtime commands before processing audio so mode/threshold changes
            // land between utterances without restart.
            while let Ok(cmd) = runtime_rx.try_recv() {
                match cmd {
                    RuntimeCommand::SetMode(m) => {
                        tracing::info!(cmd = "set_mode", mode = ?m, "runtime_command_applied");
                        match (&active_mode, &m) {
                            (ActivationMode::Ptt, ActivationMode::Wake) => {
                                // PTT → Wake: discard any in-progress PTT capture and reset VAD.
                                ptt_audio.clear();
                                prev_ptt = false;
                                seg = VadSegmenter::new(seg_cfg.clone());
                            }
                            (ActivationMode::Wake, ActivationMode::Ptt) => {
                                // Wake → PTT: clear wake listen window state.
                                listening_until = None;
                                listening_started_at = None;
                                wake_preroll_len = 0;
                                seg = VadSegmenter::new(seg_cfg.clone());
                            }
                            _ => {}
                        }
                        active_mode = m;
                    }
                    RuntimeCommand::SetThreshold(t) => {
                        let old = dispatcher_for_pipeline.threshold();
                        dispatcher_for_pipeline.update_threshold(t);
                        tracing::info!(cmd = "set_threshold", old, new = t, "runtime_command_applied");
                    }
                    RuntimeCommand::SetInputDevice(ref device) => {
                        tracing::warn!(cmd = "set_input_device", device = ?device, "command requires daemon restart in S01; ignored");
                    }
                    RuntimeCommand::SetPttBinding(ref key) => {
                        tracing::warn!(cmd = "set_ptt_binding", key = %key, "command requires daemon restart in S01; ignored");
                    }
                    RuntimeCommand::ReloadConfig => {
                        match crate::config::load(None) {
                            Ok(new_cfg) => {
                                tracing::info!(cmd = "reload_config", "runtime_command_applied");
                                // Apply hot-reloadable fields only.
                                if new_cfg.stt.confidence_threshold != 0.0 {
                                    dispatcher_for_pipeline.update_threshold(new_cfg.stt.confidence_threshold);
                                }
                            }
                            Err(e) => {
                                tracing::warn!(cmd = "reload_config", error = %e, "config reload failed; ignoring");
                            }
                        }
                    }
                }
            }

            // Always drain completed STT results first so they are never lost regardless
            // of PTT state.  Previously this was inside the VAD branch and was skipped by
            // `continue` while PTT was idle, causing results to accumulate forever.
            if let Some(stt_results) = &stt_results {
                while let Ok(r) = stt_results.try_recv() {
                    if crate::vad::try_send_drop_oldest(&dispatch_tx, &dispatch_rx_for_drop, r).is_err() {
                        break;
                    }
                }
            }

            // Drain some samples from ringbuf.
            let n = consumer.pop_slice(&mut tmp);
            if n == 0 {
                std::thread::sleep(Duration::from_millis(5));
            } else {
                pending.extend_from_slice(&tmp[..n]);
                heartbeat_samples += n;
            }

            // Periodic heartbeat so we can confirm audio is flowing (or diagnose silence).
            if heartbeat_last.elapsed() >= Duration::from_secs(5) {
                tracing::debug!(samples = heartbeat_samples, "pipeline heartbeat: audio samples received in last 5s");
                heartbeat_samples = 0;
                heartbeat_last = Instant::now();
            }

            // Compact pending buffer occasionally.
            if pending_idx > FRAME_SAMPLES * 16 {
                pending.drain(0..pending_idx);
                pending_idx = 0;
            }

            while pending.len().saturating_sub(pending_idx) >= FRAME_SAMPLES {
                frame.copy_from_slice(&pending[pending_idx..pending_idx + FRAME_SAMPLES]);
                pending_idx += FRAME_SAMPLES;

                let now = Instant::now();
                let ptt = ptt_active.load(Ordering::Relaxed);
                let listening = listening_until.map(|t| now < t).unwrap_or(false);

                // Mute gate: drop frame and reset any in-progress state.
                if muted.load(Ordering::Relaxed) {
                    if prev_ptt { prev_ptt = false; }
                    if listening_until.is_some() {
                        listening_until = None;
                        listening_started_at = None;
                        wake_preroll_len = 0;
                        seg = VadSegmenter::new(seg_cfg.clone());
                    }
                    continue;
                }

                // PTT pressed: start or continue recording directly into ptt_audio.
                // In Wake mode, ignore PTT rising/falling edges entirely.
                if ptt && active_mode == ActivationMode::Ptt {
                    if !prev_ptt {
                        // Rising edge: begin new PTT utterance.
                        ptt_audio.clear();
                        ptt_utterance_id = ptt_next_id;
                        ptt_next_id += 1;
                        ptt_timings = crate::pipeline::timing::UtteranceTimings::new();
                        ptt_start_frame = ptt_frame_count;
                        tracing::debug!(utterance_id = ptt_utterance_id, "PTT utterance started");
                    }
                    ptt_audio.extend_from_slice(&frame);
                    ptt_frame_count += 1;
                    prev_ptt = true;
                    continue;
                }

                // PTT just released: send entire PTT buffer to STT.
                // In Wake mode prev_ptt stays false, so this block is never entered.
                if prev_ptt && active_mode == ActivationMode::Ptt {
                    let audio_len = ptt_audio.len();
                    let min_samples = (crate::vad::SAMPLE_RATE_HZ as usize) / 10; // 100 ms minimum
                    tracing::debug!(
                        utterance_id = ptt_utterance_id,
                        samples = audio_len,
                        "PTT released – submitting to STT"
                    );
                    if audio_len >= min_samples {
                        // For PTT captures we bypass VAD, but we still want a stable
                        // monotonic milestone so latency fields are meaningful.
                        ptt_timings.mark_vad_done();
                        let job = crate::vad::UtteranceJob {
                            utterance_id: ptt_utterance_id,
                            audio: std::mem::take(&mut ptt_audio),
                            timings: ptt_timings,
                            start_frame_idx: ptt_start_frame,
                            end_frame_idx: ptt_frame_count,
                        };
                        if let Some(stt_submit) = &stt_submit {
                            if let Err(e) = stt_submit.try_submit(job) {
                                tracing::warn!(utterance_id = ptt_utterance_id, "STT submit failed: {e}");
                            }
                        }
                    } else {
                        tracing::debug!(utterance_id = ptt_utterance_id, samples = audio_len, "PTT too short, skipping STT");
                        ptt_audio.clear();
                    }
                    // Reset the VAD segmenter so stale state doesn't bleed into
                    // the next wake-word listen window.
                    seg = VadSegmenter::new(seg_cfg.clone());
                }
                prev_ptt = false;

                // Wake-word detection while idle — only in Wake activation mode.
                // In PTT mode skip this branch entirely so the wake detector stays dormant.
                if active_mode == ActivationMode::Wake && !listening {
                    if let Some(w) = &wake {
                        // Maintain rolling pre-roll while idle so we can seed VAD
                        // immediately when wake triggers.
                        wake_preroll[wake_preroll_pos] = frame;
                        wake_preroll_pos = (wake_preroll_pos + 1) % wake_preroll.len();
                        wake_preroll_len = wake_preroll_len.saturating_add(1).min(wake_preroll.len());

                        w.accept_audio(16_000, &frame);
                        w.decode_until_not_ready();
                        if let Some(keyword) = w.take_keyword() {
                            tracing::info!(keyword = %keyword, "Wake word triggered; entering LISTENING window");
                            listening_until = Some(now + listen_window);
                            listening_started_at = Some(now);
                            w.reset();
                            // Seed VAD with pre-roll frames captured while idle.
                            // This helps include the first command word if it started
                            // right before the wake trigger fired.
                            seg = VadSegmenter::new(seg_cfg_wake.clone());
                            if wake_preroll_len > 0 {
                                let cap = wake_preroll.len();
                                let start = (wake_preroll_pos + cap - wake_preroll_len) % cap;
                                for i in 0..wake_preroll_len {
                                    let idx = (start + i) % cap;
                                    let _ = seg.push_frame_silero(&mut silero, &wake_preroll[idx]);
                                }
                            }
                            // Also feed current frame into VAD below (do not `continue`).
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }

                // Wake-word listen window: run VAD segmentation.
                match seg.push_frame_silero(&mut silero, &frame) {
                    Ok(Some(mut job)) => {
                        job.timings.mark_vad_done();

                        if let Some(stt_submit) = &stt_submit {
                            if let Err(e) = stt_submit.try_submit(job) {
                                tracing::warn!("STT submit failed: {e}");
                            }
                        }
                    }
                    Ok(None) => {}
                    Err(e) => tracing::warn!("VAD error: {e}"),
                }

                // Wake mode fallback: if VAD never observes "enough silence" due to
                // noise/AGC, force-flush after a short budget once LISTENING started.
                if listening {
                    if let Some(t0) = listening_started_at {
                        if t0.elapsed() >= Duration::from_millis(WAKE_FORCE_FLUSH_MS) {
                            if let Some(mut job) = seg.force_flush() {
                                tracing::debug!(
                                    utterance_id = job.utterance_id,
                                    "Wake force-flush after {}ms",
                                    WAKE_FORCE_FLUSH_MS
                                );
                                job.timings.mark_vad_done();
                                if let Some(stt_submit) = &stt_submit {
                                    if let Err(e) = stt_submit.try_submit(job) {
                                        tracing::warn!("STT submit failed: {e}");
                                    }
                                }
                            }
                            // Reset timer so we don't spam force_flush.
                            listening_started_at = Some(now);
                        }
                    }
                }

                // If LISTENING window expired, reset wake pre-roll so next command
                // doesn't get polluted by prior-window audio.
                if listening_until.map(|t| now >= t).unwrap_or(false) {
                    listening_until = None;
                    listening_started_at = None;
                    wake_preroll_len = 0;
                    seg = VadSegmenter::new(seg_cfg.clone());
                }
            }
        }

        tracing::info!("Pipeline thread stopped");
    });

    Ok(PipelineHandles { pipeline, output, stt, dispatcher })
}

/// Locate `silero_vad.onnx` from well-known installed paths.
///
/// Returns `None` for dev builds — callers fall back to the crate's own resolver
/// which uses `CARGO_MANIFEST_DIR` (valid only when the cargo registry is present).
fn find_silero_model() -> Option<std::path::PathBuf> {
    const MODEL: &str = "silero_vad.onnx";
    const PKG: &str = "vibe-attack";

    // 1. Standard install location (deb/rpm/PKGBUILD)
    let system = std::path::Path::new("/usr/share").join(PKG).join(MODEL);
    if system.exists() {
        return Some(system);
    }

    // 2. XDG data dirs (user-local or alternate prefix installs)
    let xdg = xdg::BaseDirectories::with_prefix(PKG);
    if let Some(p) = xdg.find_data_file(MODEL) {
        return Some(p);
    }

    None
}

