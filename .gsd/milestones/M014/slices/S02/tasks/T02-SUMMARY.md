---
id: T02
parent: S02
milestone: M014
key_files:
  - src/config.rs
  - src/pipeline/coordinator.rs
  - config.example.yaml
  - config.yaml
key_decisions:
  - (none)
duration: 
verification_result: passed
completed_at: 2026-06-07T21:11:06.813Z
blocker_discovered: false
---

# T02: Updated VAD config defaults and wired onset_window_ms through config.rs and coordinator.rs

**Updated VAD config defaults and wired onset_window_ms through config.rs and coordinator.rs**

## What Happened

Updated config.rs VadConfig: start_threshold 0.60->0.50, stop_threshold 0.45->0.30, end_silence_ms 200->500, min_speech_ms 100->60. Added onset_window_ms: 100 as a new config field with serde default. Updated coordinator.rs seg_cfg construction to include onset_window_frames. Fixed wake mode seg_cfg_wake to use correct onset_window_frames and removed the previous (start - 0.05).max(stop) formula which was too aggressive. Updated config.example.yaml and config.yaml with new values and comments.

## Verification

cargo test — all 105+ tests pass

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test` | 0 | pass | 8000ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/config.rs`
- `src/pipeline/coordinator.rs`
- `config.example.yaml`
- `config.yaml`
