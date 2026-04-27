use crate::pipeline::timing::{wall_time_ms, MonoClock};
use serde::Serialize;
use std::io::{Result as IoResult, Write};

/// Controls which events are emitted to stdout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonlVerbosity {
    /// Emit only the summary utterance event (D-22).
    SummaryOnly,
    /// Emit summary + optional stage events (D-22).
    Stages,
}

/// Identifies the pipeline stage in a [`JsonlEvent::Stage`] event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StageName {
    Vad,
    Stt,
}

/// Whether a [`JsonlEvent::Stage`] event marks the start or end of a pipeline stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StageStatus {
    Start,
    End,
}

/// Stable stdout JSONL event contract (D-19..D-22).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JsonlEvent<'a> {
    /// Always emitted once per utterance (D-22).
    Utterance {
        utterance_id: u64,
        /// Wall clock when the utterance was created (ms since unix epoch).
        created_wall_time_ms: u64,
        wall_time_ms: u64,
        mono_ms: u64,
        text: &'a str,
        audio_ms: u64,
        stt_ms: u64,
        /// End-to-end duration (ms) for Phase 2 verification:
        /// `e2e_ms = output_done_ms - vad_done_ms` (monotonic).
        e2e_ms: u64,
        /// Measured VAD compute cost (ms) for the utterance.
        vad_ms: u64,
        vad_done_ms: Option<u64>,
        stt_done_ms: Option<u64>,
        output_done_ms: Option<u64>,
        start_frame_idx: u64,
        end_frame_idx: u64,
    },
    /// Optional stage-level event (gated by verbosity, D-22).
    Stage {
        utterance_id: u64,
        wall_time_ms: u64,
        mono_ms: u64,
        stage: StageName,
        status: StageStatus,
    },
    /// Status event for startup/shutdown/heartbeat (D-19).
    Status {
        message: &'a str,
        mono_ms: u64,
    },
    /// Emitted when a transcript matches a macro and the macro fires.
    Dispatch {
        utterance_id: u64,
        macro_id: &'a str,
        score: f32,
        wall_time_ms: u64,
        mono_ms: u64,
    },
    /// Emitted when a transcript does not match any phrase above threshold.
    NoMatch {
        utterance_id: u64,
        transcript: &'a str,
        wall_time_ms: u64,
        mono_ms: u64,
    },
}

/// JSONL writer that guarantees "one JSON object per line" on the provided writer.
///
/// It intentionally knows nothing about stderr logging; callers handle diagnostics via `tracing`.
pub struct JsonlWriter<W: Write> {
    w: W,
    verbosity: JsonlVerbosity,
    clock: MonoClock,
}

impl<W: Write> JsonlWriter<W> {
    /// Create a writer that emits JSONL to `w` at the given verbosity level.
    ///
    /// The internal [`MonoClock`] is anchored at construction time; all `mono_ms` fields
    /// in emitted events are relative to this instant.
    pub fn new(w: W, verbosity: JsonlVerbosity) -> Self {
        Self {
            w,
            verbosity,
            clock: MonoClock::start_now(),
        }
    }

    /// Return the verbosity level this writer was constructed with.
    pub fn verbosity(&self) -> JsonlVerbosity {
        self.verbosity
    }

    // Each argument maps to a distinct top-level field in the JSONL event schema; no meaningful grouping reduces them.
    #[allow(clippy::too_many_arguments)]
    pub fn write_utterance(
        &mut self,
        utterance_id: u64,
        text: &str,
        audio_ms: u64,
        stt_ms: u64,
        timings: crate::pipeline::timing::UtteranceTimings,
        start_frame_idx: u64,
        end_frame_idx: u64,
    ) -> IoResult<()> {
        // `UtteranceTimings` monotonic markers are relative to utterance creation.
        // Emit an always-defined end-to-end duration as "creation → output done".
        // (Some capture modes may not set `vad_done_ms`.)
        let e2e_ms = timings.output_done_ms.unwrap_or(0);
        let evt = JsonlEvent::Utterance {
            utterance_id,
            created_wall_time_ms: timings.created_wall_time_ms,
            wall_time_ms: wall_time_ms(),
            mono_ms: self.clock.elapsed_ms(),
            text,
            audio_ms,
            stt_ms,
            e2e_ms,
            vad_ms: timings.vad_ms,
            vad_done_ms: timings.vad_done_ms,
            stt_done_ms: timings.stt_done_ms,
            output_done_ms: timings.output_done_ms,
            start_frame_idx,
            end_frame_idx,
        };
        self.write_event(&evt)
    }

    /// Emit an optional stage-start/end event (no-op when verbosity is `SummaryOnly`).
    pub fn write_stage(
        &mut self,
        utterance_id: u64,
        stage: StageName,
        status: StageStatus,
    ) -> IoResult<()> {
        if self.verbosity != JsonlVerbosity::Stages {
            return Ok(());
        }
        let evt = JsonlEvent::Stage {
            utterance_id,
            wall_time_ms: wall_time_ms(),
            mono_ms: self.clock.elapsed_ms(),
            stage,
            status,
        };
        self.write_event(&evt)
    }

    /// Emit a `dispatch` event recording which macro fired and at what confidence score.
    pub fn write_dispatch(
        &mut self,
        utterance_id: u64,
        macro_id: &str,
        score: f32,
    ) -> IoResult<()> {
        let evt = JsonlEvent::Dispatch {
            utterance_id,
            macro_id,
            score,
            wall_time_ms: wall_time_ms(),
            mono_ms: self.clock.elapsed_ms(),
        };
        self.write_event(&evt)
    }

    /// Emit a `no_match` event when the transcript did not match any macro phrase.
    pub fn write_no_match(&mut self, utterance_id: u64, transcript: &str) -> IoResult<()> {
        let evt = JsonlEvent::NoMatch {
            utterance_id,
            transcript,
            wall_time_ms: wall_time_ms(),
            mono_ms: self.clock.elapsed_ms(),
        };
        self.write_event(&evt)
    }

    /// Serialize `evt` as a single JSON line and flush.
    pub fn write_event(&mut self, evt: &JsonlEvent<'_>) -> IoResult<()> {
        serde_json::to_writer(&mut self.w, evt)?;
        self.w.write_all(b"\n")?;
        self.w.flush()?;
        Ok(())
    }
}

