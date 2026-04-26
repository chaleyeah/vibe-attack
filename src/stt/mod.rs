//! Speech-to-text (whisper.cpp via `whisper-rs`) running on a dedicated OS thread.
//!
//! Constraints:
//! - Must run on a dedicated blocking OS thread (never Tokio).
//! - Model is loaded from a local path at startup (no network).
//! - Inference is CPU-only; final-only output (no streaming partials by default).
//! - Input queue is bounded with drop-oldest semantics; send path must not block.

use anyhow::{anyhow, Context, Result};
use crossbeam_channel::{Receiver, Sender};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

use crate::vad::{try_send_drop_oldest, UtteranceJob};

#[derive(Debug)]
pub enum SttMsg {
    Job(UtteranceJob),
    Shutdown,
}

#[derive(Debug, Clone)]
pub struct SttResult {
    pub utterance_id: u64,
    pub text: String,
    pub audio_ms: u64,
    pub stt_ms: u64,
    pub timings: crate::pipeline::timing::UtteranceTimings,
    pub start_frame_idx: u64,
    pub end_frame_idx: u64,
}

#[derive(Debug)]
pub struct SttService {
    model_path: PathBuf,
    initial_prompt: Option<String>,
    job_tx: Sender<SttMsg>,
    job_rx: Receiver<SttMsg>,
    result_tx: Sender<SttResult>,
    pub result_rx: Receiver<SttResult>,
    shutdown: CancellationToken,
    handle: Option<std::thread::JoinHandle<()>>,
}

/// Cloneable submit-only handle for the STT bounded queue.
#[derive(Debug, Clone)]
pub struct SttSubmitter {
    job_tx: Sender<SttMsg>,
    job_rx: Receiver<SttMsg>,
}

impl SttSubmitter {
    /// Non-blocking submit. If queue is full, drops the oldest pending job (D-03).
    pub fn try_submit(&self, job: UtteranceJob) -> Result<()> {
        try_send_drop_oldest(&self.job_tx, &self.job_rx, SttMsg::Job(job))
            .map_err(|_| anyhow!("STT job queue is disconnected"))
    }
}

impl SttService {
    pub fn submitter(&self) -> SttSubmitter {
        SttSubmitter {
            job_tx: self.job_tx.clone(),
            job_rx: self.job_rx.clone(),
        }
    }

    /// Convenience for single-owner use: submit directly via the service.
    pub fn try_submit(&self, job: UtteranceJob) -> Result<()> {
        self.submitter().try_submit(job)
    }

    pub fn result_receiver(&self) -> Receiver<SttResult> {
        self.result_rx.clone()
    }

    /// Create the STT service and preload the model from `model_path`.
    ///
    /// This function spawns a long-lived `std::thread` and returns immediately.
    pub fn new(
        model_path: impl AsRef<Path>,
        initial_prompt: Option<String>,
        shutdown: CancellationToken,
    ) -> Result<Self> {
        let model_path = model_path.as_ref().to_path_buf();
        ensure_model_exists(&model_path)?;

        // Bounded queue: must not allow unbounded growth (T-02-07).
        let (job_tx, job_rx) = crossbeam_channel::bounded::<SttMsg>(4);
        let (result_tx, result_rx) = crossbeam_channel::bounded::<SttResult>(8);

        Ok(Self {
            model_path,
            initial_prompt,
            job_tx,
            job_rx,
            result_tx,
            result_rx,
            shutdown,
            handle: None,
        })
    }

    pub fn spawn(mut self) -> Result<Self> {
        let job_rx = self.job_rx.clone();
        let result_tx = self.result_tx.clone();
        let result_rx_for_drop = self.result_rx.clone();
        let shutdown = self.shutdown.clone();

        let model_path = self.model_path.clone();
        let _initial_prompt = self.initial_prompt.clone();

        let handle = std::thread::spawn(move || {
            tracing::info!("STT thread started");

            #[cfg(not(feature = "stt"))]
            {
                let _ = model_path; // silence unused warning
                let _ = (&job_rx, &result_tx, &result_rx_for_drop, &shutdown);
                tracing::error!("STT is enabled but binary was built without `--features stt`");
                tracing::info!("STT thread stopped");
                return;
            }

            #[cfg(feature = "stt")]
            {
                use whisper_rs::{
                    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters,
                };
                use std::time::Instant;

                // Load model once at startup (D-15).
                let start_load = Instant::now();
                let ctx = match WhisperContext::new_with_params(
                    model_path.to_string_lossy().to_string(),
                    WhisperContextParameters::default(),
                ) {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::error!("Failed to load whisper model: {e}");
                        return;
                    }
                };
                tracing::info!("Whisper model loaded in {}ms", start_load.elapsed().as_millis());

                let base_params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

                // Create state once and reuse across jobs — avoids heap fragmentation from
                // repeated 190 MB alloc/free cycles that cause bad_alloc after ~N utterances.
                let mut state = match ctx.create_state() {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::error!("Failed to create whisper state: {e}");
                        return;
                    }
                };

                loop {
                    if shutdown.is_cancelled() {
                        break;
                    }

                    match job_rx.recv_timeout(Duration::from_millis(50)) {
                        Ok(SttMsg::Shutdown) => break,
                        Ok(SttMsg::Job(job)) => {
                            let stt_start = Instant::now();

                            let mut params = base_params.clone();
                            params.set_print_special(false);
                            params.set_print_progress(false);
                            params.set_print_realtime(false);
                            params.set_print_timestamps(false);
                            params.set_translate(false);
                            if let Some(ref prompt) = initial_prompt {
                                params.set_initial_prompt(prompt);
                            }

                            if let Err(e) = state.full(params, &job.audio) {
                                tracing::warn!(utterance_id = job.utterance_id, "whisper full() failed: {e}");
                                continue;
                            }

                            let n = state.full_n_segments();

                            let mut text = String::new();
                            for i in 0..n {
                                if let Some(seg) = state.get_segment(i) {
                                    let seg = seg
                                        .to_str_lossy()
                                        .unwrap_or_default();
                                    let seg = seg.trim();
                                    if !seg.is_empty() {
                                        text.push_str(seg);
                                        if !text.ends_with(' ') {
                                            text.push(' ');
                                        }
                                    }
                                }
                            }
                            let text = text.trim().to_string();

                            let stt_ms = stt_start.elapsed().as_millis() as u64;
                            let audio_ms = ((job.audio.len() as u64) * 1000) / (crate::vad::SAMPLE_RATE_HZ as u64);

                            let mut timings = job.timings;
                            timings.mark_stt_done();

                            let result = SttResult {
                                utterance_id: job.utterance_id,
                                text,
                                audio_ms,
                                stt_ms,
                                timings,
                                start_frame_idx: job.start_frame_idx,
                                end_frame_idx: job.end_frame_idx,
                            };

                            // Non-blocking bounded send with drop-oldest behavior.
                            if try_send_drop_oldest(&result_tx, &result_rx_for_drop, result).is_err() {
                                break;
                            }
                        }
                        Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
                        Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
                    }
                }

                tracing::info!("STT thread stopped");
            }
        });

        self.handle = Some(handle);
        Ok(self)
    }

    pub fn request_shutdown(&self) {
        let _ = self.job_tx.try_send(SttMsg::Shutdown);
    }

    pub fn join_best_effort(&mut self, timeout: Duration) {
        if let Some(handle) = self.handle.take() {
            let joiner = std::thread::spawn(move || handle.join());
            std::thread::sleep(timeout);
            drop(joiner);
        }
    }
}

fn ensure_model_exists(path: &Path) -> Result<()> {
    std::fs::metadata(path).with_context(|| {
        format!(
            "STT model file missing: {}\n\
             Provide a local whisper.cpp model file path (e.g. tiny.en.bin).",
            path.display()
        )
    })?;
    Ok(())
}

