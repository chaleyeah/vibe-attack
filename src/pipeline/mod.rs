//! Pipeline building blocks: timing helpers + stdout JSONL contracts.
//!
//! This module intentionally does **not** perform any heavy work. It provides:
//! - Stable, serde-serializable event types for stdout JSONL (D-19..D-22)
//! - Monotonic + wall-clock timing helpers (D-21)
//!
//! All diagnostics must go through `tracing` (stderr) elsewhere; stdout must remain JSONL-pure.

pub mod jsonl;
pub mod timing;
pub mod coordinator;

