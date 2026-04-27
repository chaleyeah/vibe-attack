//! Pipeline building blocks: timing helpers + stdout JSONL contracts.
//!
//! This module intentionally does **not** perform any heavy work. It provides:
//! - Stable, serde-serializable event types for stdout JSONL (D-19..D-22)
//! - Monotonic + wall-clock timing helpers (D-21)
//!
//! All diagnostics must go through `tracing` (stderr) elsewhere; stdout must remain JSONL-pure.

/// Stdout JSONL event types and writer (D-19..D-22).
pub mod jsonl;
/// Monotonic and wall-clock timing helpers (D-21).
pub mod timing;
/// Thread coordinator: spawns audio-drain, STT, dispatch, and output threads.
pub mod coordinator;
/// Fuzzy phrase matcher for mapping STT transcripts to macro names.
pub mod matcher;
/// Fire-and-forget audio playback for macro confirmation sounds.
pub mod sound;
/// Dispatcher: matches transcripts and fires macro key sequences.
pub mod dispatcher;

