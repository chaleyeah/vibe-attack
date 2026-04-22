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

use crate::audio::AudioHandle;
use crate::config::{Config, PipelineVerbosity};
use crate::pipeline::jsonl::{JsonlVerbosity, JsonlWriter};
use crate::stt::{SttResult, SttService};
use crate::vad::{VadConfig as SegCfg, VadSegmenter, FRAME_SAMPLES};
use crate::wake::WakeWord;

pub struct PipelineHandles {
    pub pipeline: std::thread::JoinHandle<()>,
    pub output: std::thread::JoinHandle<()>,
    pub stt: Option<SttService>,
}

pub fn spawn_pipeline(
    audio: AudioHandle,
    config: Config,
    ptt_active: Arc<AtomicBool>,
    shutdown: CancellationToken,
) -> Result<PipelineHandles> {
    let listen_window = Duration::from_secs(config.pipeline.listen_window_secs);

    // Build VAD config from config file.
    let ms_to_frames = |ms: u64| ((ms + 19) / 20).max(1) as usize;
    let seg_cfg = SegCfg {
        start_threshold: config.vad.start_threshold,
        stop_threshold: config.vad.stop_threshold,
        min_speech_frames: ms_to_frames(config.vad.min_speech_ms),
        end_silence_frames: ms_to_frames(config.vad.end_silence_ms),
        preroll_frames: ((config.vad.preroll_ms + 19) / 20) as usize,
        tail_frames: ((config.vad.tail_ms + 19) / 20) as usize,
        max_utterance_frames: (config.vad.max_utterance_secs as usize) * 50, // 50 frames/sec
    };

    // Output thread owns stdout writer and ensures stdout remains JSONL-only.
    let (out_tx, out_rx) = crossbeam_channel::bounded::<SttResult>(8);
    let out_rx_for_drop = out_rx.clone();

    let jsonl_verbosity = match config.pipeline.verbosity {
        PipelineVerbosity::Summary => JsonlVerbosity::SummaryOnly,
        PipelineVerbosity::Stages => JsonlVerbosity::Stages,
    };

    let output_shutdown = shutdown.clone();
    let output = std::thread::spawn(move || {
        let stdout = std::io::stdout();
        let mut w = JsonlWriter::new(stdout.lock(), jsonl_verbosity);

        while !output_shutdown.is_cancelled() {
            match out_rx.recv_timeout(Duration::from_millis(50)) {
                Ok(mut r) => {
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
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
            }
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
            return Err(anyhow::anyhow!(
                "Config enables STT, but this build was compiled without `--features stt`."
            ));
        }

        #[cfg(feature = "stt")]
        {
            Some(
                SttService::new(model_path, shutdown.clone())
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

    // VAD model is required for utterance segmentation (CPU-only baseline, D-16).
    //
    // IMPORTANT: `ort` (ONNX Runtime) can panic at runtime if `libonnxruntime.so`
    // is not discoverable. Convert that into a normal, actionable startup error
    // instead of crashing the process.
    let mut silero = std::panic::catch_unwind(|| {
        silero_vad_rust::silero_vad::model::load_silero_vad_with_options(
            silero_vad_rust::silero_vad::model::LoadOptions {
                force_onnx_cpu: true,
                ..Default::default()
            },
        )
    })
    .map_err(|_| {
        anyhow::anyhow!(
            "Failed to load Silero VAD: ONNX Runtime could not be loaded.\n\
             Fix: install ONNX Runtime so `libonnxruntime.so` is in your library path, or set ORT_DYLIB_PATH to the full path of `libonnxruntime.so`.\n\
             Example (Arch/CachyOS): `sudo pacman -S onnxruntime`"
        )
    })?
    .context("load silero VAD model")?;

    // Move the AudioHandle into the pipeline thread to keep the CPAL stream alive.
    let audio = audio;
    let pipeline_shutdown = shutdown.clone();

    let pipeline = std::thread::spawn(move || {
        tracing::info!("Pipeline thread started");

        let mut seg = VadSegmenter::new(seg_cfg);
        let mut consumer = audio.consumer;

        let mut pending = Vec::<f32>::with_capacity(FRAME_SAMPLES * 8);
        let mut pending_idx: usize = 0;
        let mut tmp = [0.0f32; 1024];
        let mut frame = [0.0f32; FRAME_SAMPLES];

        let mut listening_until: Option<Instant> = None;
        let mut prev_ptt = false;

        while !pipeline_shutdown.is_cancelled() {
            // Drain some samples from ringbuf.
            let n = consumer.pop_slice(&mut tmp);
            if n == 0 {
                std::thread::sleep(Duration::from_millis(5));
            } else {
                pending.extend_from_slice(&tmp[..n]);
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

                // If PTT was just released, force-flush any in-progress utterance.
                // Otherwise we may never see enough trailing silence to end the utterance,
                // because the audio callback stops feeding samples immediately on release.
                if prev_ptt && !ptt {
                    if let Some(mut job) = seg.force_flush() {
                        job.timings.mark_vad_done();
                        if let Some(stt_submit) = &stt_submit {
                            if let Err(e) = stt_submit.try_submit(job) {
                                tracing::warn!("STT submit failed (flush): {e}");
                            }
                        }
                    }
                }
                prev_ptt = ptt;

                // Wake-word detection while idle (no PTT) to enter LISTENING (D-17/D-18).
                if !ptt && !listening {
                    if let Some(w) = &wake {
                        w.accept_audio(16_000, &frame);
                        w.decode_until_not_ready();
                        if w.take_keyword().is_some() {
                            tracing::info!("Wake word triggered; entering LISTENING window");
                            listening_until = Some(now + listen_window);
                            w.reset();
                        }
                    }
                    continue;
                }

                // Active capture: run VAD segmentation.
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

                // Drain any completed STT results and forward to output thread.
                if let Some(stt_results) = &stt_results {
                    while let Ok(r) = stt_results.try_recv() {
                        if crate::vad::try_send_drop_oldest(&out_tx, &out_rx_for_drop, r).is_err() {
                            break;
                        }
                    }
                }
            }
        }

        tracing::info!("Pipeline thread stopped");
    });

    Ok(PipelineHandles { pipeline, output, stt })
}

