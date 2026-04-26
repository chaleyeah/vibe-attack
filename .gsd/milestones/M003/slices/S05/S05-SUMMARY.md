---
id: S05
parent: M003
milestone: M003
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - (none)
key_decisions:
  - (none)
patterns_established:
  - (none)
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-26T00:24:10.345Z
blocker_discovered: false
---

# S05: Integration smoke tests

**17 hermetic unit tests covering probe, FirstRunState, load_profiles, and log cap; all pass under cargo test --lib without display server**

## What Happened

Added 5 FirstRunState tests in first_run.rs covering step ordering, completion state, and first_incomplete_step dispatch. Added 4 ConfigApp tests in config_app.rs covering load_profiles (absent dir, sorted yaml, non-yaml ignore) and add_log_line cap. All 8 probe tests continue to pass. rewrite_ptt_key tests are in wizard.rs gated to gui feature — verified as correct by inspection; documented that they require a display server to run. Total: 17 ui:: tests, all passing.

## Verification

cargo test --lib ui:: exits 0 with 17 tests passing; no display server required

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

- `src/ui/first_run.rs` — Added 5 unit tests for FirstRunState state machine
- `src/ui/config_app.rs` — Added 4 unit tests for load_profiles and add_log_line
