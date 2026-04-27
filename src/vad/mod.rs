//! Silero-based VAD scoring + utterance segmentation.
//!
//! Phase 2 constraints / locked decisions:
//! - 20ms pacing @ 16kHz (320 samples per "frame") (D-01/D-09)
//! - Silero expects 512-sample windows; score using a sliding 512-sample window,
//!   advanced every 20ms (320 samples) while still producing 20ms-resolution scores
//! - No inference inside the CPAL callback (segmentation runs on an OS thread)
//! - Hysteresis start threshold > stop threshold (D-10)
//! - Minimum speech before committing (D-12)
//! - End-of-speech requires ~400ms silence (D-11)
//! - Include preroll + tail audio (D-06/D-07)
//! - Cap utterance length (~10s) and force-cut (D-08)
//! - Bounded queues use drop-oldest semantics (D-03)

use crate::pipeline::timing::UtteranceTimings;
use crossbeam_channel::{Receiver, Sender, TrySendError};
use std::collections::VecDeque;
use std::time::Instant;

/// Required audio sample rate for all VAD and downstream processing (16 kHz).
pub const SAMPLE_RATE_HZ: u32 = 16_000;
/// Number of samples per VAD processing frame (320 = 20ms @ 16kHz, per D-01/D-09).
pub const FRAME_SAMPLES: usize = 320; // 20ms @ 16kHz
/// Sliding window size fed to the Silero model; advanced every [`FRAME_SAMPLES`].
pub const SILERO_WINDOW_SAMPLES: usize = 512;

/// Non-blocking bounded send with **drop-oldest** behavior (D-03).
///
/// Intended for VAD → STT queueing under load: preserves responsiveness by keeping the newest items.
pub fn try_send_drop_oldest<T>(tx: &Sender<T>, rx: &Receiver<T>, item: T) -> Result<(), T> {
    match tx.try_send(item) {
        Ok(()) => Ok(()),
        Err(TrySendError::Disconnected(v)) => Err(v),
        Err(TrySendError::Full(v)) => {
            // Drop one oldest item (if any) then retry once.
            let _ = rx.try_recv();
            match tx.try_send(v) {
                Ok(()) => Ok(()),
                Err(TrySendError::Full(v2)) => Err(v2),
                Err(TrySendError::Disconnected(v2)) => Err(v2),
            }
        }
    }
}

/// Hysteresis and segmentation parameters for [`VadSegmenter`].
///
/// All frame counts use 20ms frames (see [`FRAME_SAMPLES`]). The default values implement the
/// locked decisions D-06 through D-12 and are tuned for push-to-talk gaming use.
#[derive(Debug, Clone)]
pub struct VadConfig {
    /// Silero speech-probability threshold to begin an utterance (D-10).
    pub start_threshold: f32,
    /// Silero speech-probability threshold to end an utterance; must be < `start_threshold` (D-10).
    pub stop_threshold: f32,
    /// Minimum consecutive speech frames before the utterance is committed (D-12).
    pub min_speech_frames: usize,
    /// Number of consecutive non-speech frames required to close an utterance (D-11).
    pub end_silence_frames: usize,
    /// Frames of audio buffered before speech onset and prepended to every utterance (D-06).
    pub preroll_frames: usize,
    /// Frames of post-speech audio appended to every utterance to capture trailing sounds (D-07).
    pub tail_frames: usize,
    /// Hard cap on utterance length in frames; forces a cut when reached (D-08).
    pub max_utterance_frames: usize,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            // Thresholds are intentionally conservative defaults; caller can tune.
            start_threshold: 0.60,
            stop_threshold: 0.35,
            // D-12: ~100ms minimum speech
            min_speech_frames: 5,
            // D-11: ~400ms silence to cut
            end_silence_frames: 20,
            // D-06/D-07: 100–200ms buffers
            preroll_frames: 8, // 160ms
            tail_frames: 8,    // 160ms
            // D-08: hard cap ~10s
            max_utterance_frames: 500,
        }
    }
}

/// A complete speech utterance emitted by [`VadSegmenter`] and queued for wake/STT processing.
#[derive(Debug, Clone)]
pub struct UtteranceJob {
    /// Monotonically increasing ID assigned by the segmenter; used for ordering and JSONL logging.
    pub utterance_id: u64,
    /// Audio buffer assembled on the VAD thread (never in CPAL callback).
    pub audio: Vec<f32>,
    /// Stage timing markers set as the utterance progresses through the pipeline.
    pub timings: UtteranceTimings,
    /// VAD frame index at which this utterance started (for ordering and gap detection).
    pub start_frame_idx: u64,
    /// VAD frame index at which this utterance ended.
    pub end_frame_idx: u64,
}

/// Stateful voice-activity detector that converts a stream of 20ms audio frames into [`UtteranceJob`]s.
///
/// Feed frames via `push_frame` (from the VAD OS thread, never the CPAL RT callback). Each
/// completed utterance is returned as an `Option<UtteranceJob>` from that call.
#[derive(Debug)]
pub struct VadSegmenter {
    cfg: VadConfig,
    next_utterance_id: u64,
    frame_idx: u64,

    // rolling buffers (bounded)
    preroll: VecDeque<f32>,
    pending_silence: VecDeque<f32>,

    // state
    in_speech: bool,
    start_run_frames: usize,
    silence_run_frames: usize,

    // current utterance under construction
    cur_audio: Vec<f32>,
    cur_start_frame_idx: u64,
    cur_timings: UtteranceTimings,
    cur_vad_compute_ms: u64,

    // sliding window for Silero scoring (512 samples)
    silero_window: [f32; SILERO_WINDOW_SAMPLES],
    silero_window_initialized: bool,
}

impl VadSegmenter {
    /// Create a new segmenter with the given VAD configuration.
    pub fn new(cfg: VadConfig) -> Self {
        let preroll_cap = cfg.preroll_frames * FRAME_SAMPLES;
        let pending_cap = cfg.end_silence_frames * FRAME_SAMPLES;
        Self {
            cfg,
            next_utterance_id: 1,
            frame_idx: 0,
            preroll: VecDeque::with_capacity(preroll_cap),
            pending_silence: VecDeque::with_capacity(pending_cap),
            in_speech: false,
            start_run_frames: 0,
            silence_run_frames: 0,
            cur_audio: Vec::new(),
            cur_start_frame_idx: 0,
            cur_timings: UtteranceTimings::new(),
            cur_vad_compute_ms: 0,
            silero_window: [0.0; SILERO_WINDOW_SAMPLES],
            silero_window_initialized: false,
        }
    }

    /// Update the internal 512-sample window using the newest 20ms frame (320 samples).
    ///
    /// This matches the plan constraint: advance by 320 samples but score on a 512-sample window.
    fn update_silero_window(&mut self, frame: &[f32; FRAME_SAMPLES]) {
        if !self.silero_window_initialized {
            // Start with zero-padded history; place the first frame at the tail.
            self.silero_window = [0.0; SILERO_WINDOW_SAMPLES];
            let tail_start = SILERO_WINDOW_SAMPLES - FRAME_SAMPLES;
            self.silero_window[tail_start..].copy_from_slice(frame);
            self.silero_window_initialized = true;
            return;
        }

        // Shift left by 320 (dropping oldest) and append the new frame at the end.
        // Remaining overlap = 512 - 320 = 192 samples.
        self.silero_window.copy_within(FRAME_SAMPLES.., 0);
        let tail_start = SILERO_WINDOW_SAMPLES - FRAME_SAMPLES;
        self.silero_window[tail_start..].copy_from_slice(frame);
    }

    /// Push a 20ms frame and a VAD score (0..1), returning an utterance job when one completes.
    ///
    /// This method is deterministic and is the basis for unit tests. In production, the score
    /// is computed from the current 512-sample Silero window.
    pub fn push_scored_frame(
        &mut self,
        frame: &[f32; FRAME_SAMPLES],
        score: f32,
    ) -> Option<UtteranceJob> {
        self.frame_idx += 1;

        // Maintain bounded preroll samples regardless of state.
        self.preroll.extend(frame.iter().copied());
        let preroll_cap = self.cfg.preroll_frames * FRAME_SAMPLES;
        while self.preroll.len() > preroll_cap {
            let _ = self.preroll.pop_front();
        }

        if !self.in_speech {
            if score >= self.cfg.start_threshold {
                self.start_run_frames += 1;
            } else {
                self.start_run_frames = 0;
            }

            if self.start_run_frames >= self.cfg.min_speech_frames {
                self.in_speech = true;
                self.silence_run_frames = 0;
                self.pending_silence.clear();
                self.cur_audio.clear();

                // Start utterance with preroll (includes the first speech frames already in preroll).
                self.cur_audio.extend(self.preroll.iter().copied());
                if self.cur_audio.is_empty() {
                    // When preroll is disabled, ensure we still capture the triggering frame.
                    self.cur_audio.extend(frame.iter().copied());
                }
                self.cur_start_frame_idx = self
                    .frame_idx
                    .saturating_sub(self.cfg.preroll_frames as u64);
                self.cur_timings = UtteranceTimings::new();
                self.cur_vad_compute_ms = 0;
            }

            return None;
        }

        // In speech: enforce cap by frames.
        let cur_frames = self.cur_audio.len() / FRAME_SAMPLES;
        if cur_frames >= self.cfg.max_utterance_frames {
            tracing::warn!(
                utterance_id = self.next_utterance_id,
                max_frames = self.cfg.max_utterance_frames,
                "Max utterance length exceeded; force-cutting"
            );
            return Some(self.finish_current_utterance_with_tail());
        }

        if score >= self.cfg.stop_threshold {
            // Speech (or stable continuation): flush pending silence into the utterance.
            self.silence_run_frames = 0;
            if !self.pending_silence.is_empty() {
                self.cur_audio.extend(self.pending_silence.drain(..));
            }
            self.cur_audio.extend(frame.iter().copied());
            return None;
        }

        // Potential end-of-speech: buffer silence frames until we confirm end.
        self.silence_run_frames += 1;
        self.pending_silence.extend(frame.iter().copied());
        let pending_cap = self.cfg.end_silence_frames * FRAME_SAMPLES;
        while self.pending_silence.len() > pending_cap {
            let _ = self.pending_silence.pop_front();
        }

        if self.silence_run_frames < self.cfg.end_silence_frames {
            return None;
        }

        Some(self.finish_current_utterance_with_tail())
    }

    fn finish_current_utterance_with_tail(&mut self) -> UtteranceJob {
        // Append tail portion (D-07) from the buffered silence.
        let tail_samples = self.cfg.tail_frames * FRAME_SAMPLES;
        let mut appended = 0usize;
        while appended < tail_samples {
            match self.pending_silence.pop_front() {
                Some(s) => {
                    self.cur_audio.push(s);
                    appended += 1;
                }
                None => break,
            }
        }

        let id = self.next_utterance_id;
        self.next_utterance_id += 1;

        self.cur_timings.vad_ms = self.cur_vad_compute_ms;
        let job = UtteranceJob {
            utterance_id: id,
            audio: std::mem::take(&mut self.cur_audio),
            timings: self.cur_timings,
            start_frame_idx: self.cur_start_frame_idx,
            end_frame_idx: self.frame_idx,
        };

        // Reset state for next utterance.
        self.in_speech = false;
        self.start_run_frames = 0;
        self.silence_run_frames = 0;
        self.pending_silence.clear();
        self.preroll.clear();
        self.cur_vad_compute_ms = 0;

        job
    }

    /// Score the newest 20ms frame using Silero on a 512-sample sliding window.
    pub fn score_with_silero(
        &mut self,
        model: &mut silero_vad_rust::silero_vad::model::OnnxModel,
        frame: &[f32; FRAME_SAMPLES],
    ) -> anyhow::Result<f32> {
        self.update_silero_window(frame);
        let probs = model.forward_chunk(&self.silero_window, SAMPLE_RATE_HZ)?;
        Ok(probs[[0, 0]])
    }

    /// Convenience: update internal window + push the frame using a Silero-derived score.
    pub fn push_frame_silero(
        &mut self,
        model: &mut silero_vad_rust::silero_vad::model::OnnxModel,
        frame: &[f32; FRAME_SAMPLES],
    ) -> anyhow::Result<Option<UtteranceJob>> {
        let was_in_speech = self.in_speech;
        let t0 = Instant::now();
        let score = self.score_with_silero(model, frame)?;
        let res = self.push_scored_frame(frame, score);
        let dt_ms = t0.elapsed().as_millis() as u64;

        // Accumulate compute time only for frames that are part of an utterance.
        // If this frame starts an utterance, it should count.
        if was_in_speech || self.in_speech {
            self.cur_vad_compute_ms = self.cur_vad_compute_ms.saturating_add(dt_ms);
        }

        Ok(res)
    }

    /// Force-cut the current utterance (if any) and return it as a job.
    ///
    /// This is used when the upstream audio source stops abruptly (e.g. PTT released),
    /// which would otherwise prevent VAD from ever observing trailing silence and ending
    /// the utterance naturally.
    pub fn force_flush(&mut self) -> Option<UtteranceJob> {
        if !self.in_speech {
            // Nothing in progress; reset any partial start state.
            self.start_run_frames = 0;
            self.silence_run_frames = 0;
            self.pending_silence.clear();
            return None;
        }

        Some(self.finish_current_utterance_with_tail())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame_with(value: f32) -> [f32; FRAME_SAMPLES] {
        [value; FRAME_SAMPLES]
    }

    #[test]
    fn min_speech_required_before_starting_utterance() {
        let cfg = VadConfig { min_speech_frames: 5, preroll_frames: 2, ..VadConfig::default() };
        let mut seg = VadSegmenter::new(cfg.clone());

        // 4 frames above start threshold: not enough to commit.
        for _ in 0..4 {
            assert!(seg
                .push_scored_frame(&frame_with(0.1), cfg.start_threshold + 0.01)
                .is_none());
        }

        // 5th frame commits start; still no job emitted.
        assert!(seg
            .push_scored_frame(&frame_with(0.2), cfg.start_threshold + 0.01)
            .is_none());
        assert!(seg.in_speech, "must be in_speech after min_speech_frames");
    }

    #[test]
    fn end_requires_silence_and_includes_only_tail_not_full_silence() {
        let cfg = VadConfig { preroll_frames: 2, tail_frames: 2, min_speech_frames: 1, end_silence_frames: 4, ..VadConfig::default() };
        let mut seg = VadSegmenter::new(cfg.clone());

        // Start speech immediately.
        assert!(seg
            .push_scored_frame(&frame_with(1.0), cfg.start_threshold + 0.1)
            .is_none());

        // Continue speech for 2 frames.
        assert!(seg
            .push_scored_frame(&frame_with(1.0), cfg.stop_threshold + 0.1)
            .is_none());
        assert!(seg
            .push_scored_frame(&frame_with(1.0), cfg.stop_threshold + 0.1)
            .is_none());

        // Now provide 4 silent frames: end triggers on the 4th.
        assert!(seg
            .push_scored_frame(&frame_with(0.0), cfg.stop_threshold - 0.1)
            .is_none());
        assert!(seg
            .push_scored_frame(&frame_with(0.0), cfg.stop_threshold - 0.1)
            .is_none());
        assert!(seg
            .push_scored_frame(&frame_with(0.0), cfg.stop_threshold - 0.1)
            .is_none());
        let job = seg
            .push_scored_frame(&frame_with(0.0), cfg.stop_threshold - 0.1)
            .expect("must emit job on end");

        // Tail is 2 frames; we should not include all 4 silent frames.
        // job.audio contains preroll + speech frames + tail frames (silence).
        let samples_per_frame = FRAME_SAMPLES;
        let min_expected_frames = cfg.preroll_frames + 3; // 1 start + 2 speech
        let max_expected_frames = min_expected_frames + cfg.tail_frames;
        let got_frames = job.audio.len() / samples_per_frame;
        assert!(
            got_frames <= max_expected_frames,
            "should include at most tail silence, got frames={got_frames}, max={max_expected_frames}"
        );
    }

    #[test]
    fn max_length_forces_cut() {
        let cfg = VadConfig { min_speech_frames: 1, max_utterance_frames: 3, end_silence_frames: 100, preroll_frames: 0, tail_frames: 0, ..VadConfig::default() };
        let mut seg = VadSegmenter::new(cfg.clone());

        // Start.
        assert!(seg
            .push_scored_frame(&frame_with(1.0), cfg.start_threshold + 0.1)
            .is_none());

        // Push frames until cap exceeded; should force-cut.
        assert!(seg
            .push_scored_frame(&frame_with(1.0), cfg.stop_threshold + 0.1)
            .is_none());
        assert!(seg
            .push_scored_frame(&frame_with(1.0), cfg.stop_threshold + 0.1)
            .is_none());
        let job = seg
            .push_scored_frame(&frame_with(1.0), cfg.stop_threshold + 0.1)
            .expect("must force-cut at cap");

        let got_frames = job.audio.len() / FRAME_SAMPLES;
        assert!(
            got_frames <= cfg.max_utterance_frames,
            "forced cut must not exceed cap"
        );
    }
}

