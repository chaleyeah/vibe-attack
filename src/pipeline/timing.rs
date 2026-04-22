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
    pub created_mono: MonoClock,
}

impl UtteranceTimings {
    pub fn new() -> Self {
        Self {
            created_wall_time_ms: wall_time_ms(),
            created_mono: MonoClock::start_now(),
        }
    }
}

