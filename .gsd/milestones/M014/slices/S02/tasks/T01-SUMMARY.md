---
id: T01
parent: S02
milestone: M014
key_files:
  - src/vad/mod.rs
key_decisions:
  - (none)
duration: 
verification_result: passed
completed_at: 2026-06-07T21:10:59.491Z
blocker_discovered: false
---

# T01: Replaced consecutive-frame onset counter with N-of-M sliding window majority vote in VadSegmenter

**Replaced consecutive-frame onset counter with N-of-M sliding window majority vote in VadSegmenter**

## What Happened

The root cause of missed phrases was the consecutive-frame reset: a single Silero score below start_threshold during speech onset would reset start_run_frames to 0, requiring the speaker to produce N uninterrupted above-threshold frames before triggering. Real speech onset is noisy and this almost never happened cleanly. Replaced with a VecDeque<bool> onset_window that tracks the last M frames' vote results. Onset triggers when speech_count >= min_speech_frames within the window. Added new test onset_tolerates_single_dip_in_sliding_window proving a single dip no longer prevents triggering. Also added tracing::debug! per-frame VAD score to push_frame_silero for diagnostics.

## Verification

cargo test vad — 4 tests pass including new onset_tolerates_single_dip_in_sliding_window

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test vad` | 0 | pass | 80ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/vad/mod.rs`
