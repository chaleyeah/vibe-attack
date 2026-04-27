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
    /// Anchor the clock at the current instant.
    pub fn start_now() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Return the duration since the clock was started.
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Return elapsed milliseconds since the clock was started.
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
    /// Wall-clock time when this utterance was created (ms since unix epoch, D-21).
    pub created_wall_time_ms: u64,
    created_instant: Instant,
    /// Monotonic ms from creation to VAD completion; `None` until [`mark_vad_done`](Self::mark_vad_done) is called.
    pub vad_done_ms: Option<u64>,
    /// Measured VAD compute time (ms) accumulated for frames in this utterance.
    pub vad_ms: u64,
    /// Monotonic ms from creation to STT completion; `None` until [`mark_stt_done`](Self::mark_stt_done) is called.
    pub stt_done_ms: Option<u64>,
    /// Monotonic ms from creation to output write; `None` until [`mark_output_done`](Self::mark_output_done) is called.
    pub output_done_ms: Option<u64>,
}

impl Default for UtteranceTimings {
    fn default() -> Self {
        Self::new()
    }
}

impl UtteranceTimings {
    /// Create timings anchored at the current wall-clock and monotonic instant.
    pub fn new() -> Self {
        Self {
            created_wall_time_ms: wall_time_ms(),
            created_instant: Instant::now(),
            vad_done_ms: None,
            vad_ms: 0,
            stt_done_ms: None,
            output_done_ms: None,
        }
    }

    /// Return monotonic milliseconds elapsed since this utterance was created.
    pub fn elapsed_ms(&self) -> u64 {
        self.created_instant.elapsed().as_millis() as u64
    }

    /// Record the moment VAD segmentation finished for this utterance.
    pub fn mark_vad_done(&mut self) {
        self.vad_done_ms = Some(self.elapsed_ms());
    }

    /// Record the moment STT inference finished for this utterance.
    pub fn mark_stt_done(&mut self) {
        self.stt_done_ms = Some(self.elapsed_ms());
    }

    /// Record the moment the JSONL output line was written for this utterance.
    pub fn mark_output_done(&mut self) {
        self.output_done_ms = Some(self.elapsed_ms());
    }
}

