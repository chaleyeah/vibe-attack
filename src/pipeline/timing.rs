use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Capture wall-clock time as unix milliseconds (D-21).
///
/// Chosen over RFC3339 to avoid extra dependencies; the contract allows either.
pub fn wall_time_ms() -> u64 {
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0));
    d.as_millis() as u64
}

/// A monotonic timer anchored at creation time.
#[derive(Debug, Clone, Copy)]
pub struct MonoClock {
    start: Instant,
}

impl MonoClock {
    pub fn start_now() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.elapsed().as_millis() as u64
    }
}

/// Utterance timing container that can be carried through pipeline stages.
///
/// Design goal: cheap to copy and carry (no allocations), and easy to attach
/// to JSON events for monotonic timing (D-21).
#[derive(Debug, Clone, Copy)]
pub struct UtteranceTimings {
    pub created_wall_time_ms: u64,
    created_instant: Instant,
    pub vad_done_ms: Option<u64>,
    pub stt_done_ms: Option<u64>,
    pub output_done_ms: Option<u64>,
}

impl UtteranceTimings {
    pub fn new() -> Self {
        Self {
            created_wall_time_ms: wall_time_ms(),
            created_instant: Instant::now(),
            vad_done_ms: None,
            stt_done_ms: None,
            output_done_ms: None,
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.created_instant.elapsed().as_millis() as u64
    }

    pub fn mark_vad_done(&mut self) {
        self.vad_done_ms = Some(self.elapsed_ms());
    }

    pub fn mark_stt_done(&mut self) {
        self.stt_done_ms = Some(self.elapsed_ms());
    }

    pub fn mark_output_done(&mut self) {
        self.output_done_ms = Some(self.elapsed_ms());
    }
}

