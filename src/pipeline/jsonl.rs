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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StageName {
    Vad,
    Stt,
}

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
        wall_time_ms: u64,
        mono_ms: u64,
        text: &'a str,
        audio_ms: u64,
    },
    /// Optional stage-level event (gated by verbosity, D-22).
    Stage {
        utterance_id: u64,
        wall_time_ms: u64,
        mono_ms: u64,
        stage: StageName,
        status: StageStatus,
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
    pub fn new(w: W, verbosity: JsonlVerbosity) -> Self {
        Self {
            w,
            verbosity,
            clock: MonoClock::start_now(),
        }
    }

    pub fn verbosity(&self) -> JsonlVerbosity {
        self.verbosity
    }

    pub fn write_utterance(&mut self, utterance_id: u64, text: &str, audio_ms: u64) -> IoResult<()> {
        let evt = JsonlEvent::Utterance {
            utterance_id,
            wall_time_ms: wall_time_ms(),
            mono_ms: self.clock.elapsed_ms(),
            text,
            audio_ms,
        };
        self.write_event(&evt)
    }

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

    pub fn write_event(&mut self, evt: &JsonlEvent<'_>) -> IoResult<()> {
        serde_json::to_writer(&mut self.w, evt)?;
        self.w.write_all(b"\n")?;
        self.w.flush()?;
        Ok(())
    }
}

