---
id: S02
parent: M014
milestone: M014
provides:
  - (none)
requires:
  []
affects:
  []
key_files: []
key_decisions: []
patterns_established:
  - (none)
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-06-07T21:11:18.712Z
blocker_discovered: false
---

# S02: VAD overhaul - robust onset and tuned defaults

**Replaced consecutive-frame onset gate with N-of-M sliding window; tuned thresholds and silence duration defaults**

## What Happened

The VAD onset algorithm was the primary cause of missed phrases. The old consecutive-frame counter reset to 0 on any single low-confidence frame, making it nearly impossible to trigger during real speech onset. The new sliding-window majority vote (3 of 5 frames = 60ms of 100ms) tolerates brief score dips. Defaults were tuned based on Silero characteristics: start_threshold 0.60->0.50, stop_threshold 0.45->0.30, end_silence_ms 200->500. Added per-frame DEBUG tracing and a new test proving dip tolerance.

## Verification

cargo test vad: 4 tests pass including onset_tolerates_single_dip_in_sliding_window. Full cargo test: all tests pass.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

None.

## Follow-ups

None.

## Files Created/Modified

- `src/vad/mod.rs` — N-of-M sliding window onset algorithm, tuned defaults, per-frame debug tracing
- `src/config.rs` — Added onset_window_ms field, updated all VAD defaults
- `src/pipeline/coordinator.rs` — Wire onset_window_frames into seg_cfg and seg_cfg_wake
- `config.example.yaml` — Updated VAD defaults and added onset_window_ms documentation
- `config.yaml` — Updated VAD defaults to match new tuned values
