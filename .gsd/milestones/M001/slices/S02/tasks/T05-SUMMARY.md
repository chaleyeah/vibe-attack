---
id: T05
parent: S02
milestone: M001
provides:
  - Utterance JSONL includes explicit end-to-end duration field `e2e_ms`
  - Utterance JSONL includes measured VAD compute cost field `vad_ms`
  - Schema stability tests lock `e2e_ms`/`vad_ms` keys and u64 types
  - Latency baseline doc references `e2e_ms`, `vad_ms`, and `stt_ms` with exact derivation
requires: []
affects: []
key_files: []
key_decisions: []
patterns_established: []
observability_surfaces: []
drill_down_paths: []
duration: 10m
verification_result: passed
completed_at: 2026-04-22
blocker_discovered: false
---
# T05: 02-pipeline-core 05

**# Phase 02: Pipeline Core Plan 05 Summary**

## What Happened

# Phase 02: Pipeline Core Plan 05 Summary

**Added explicit end-to-end latency (`e2e_ms`) and measured VAD compute cost (`vad_ms`) to the stable stdout JSONL utterance schema.**

## Performance

- **Duration:** 10m
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Added `e2e_ms` to the utterance JSONL event as `output_done_ms - vad_done_ms` (monotonic, ms).
- Measured and propagated per-utterance VAD compute time (`vad_ms`) distinct from utterance timeline markers.
- Updated schema stability tests and the Phase 2 latency baseline doc to reference `e2e_ms`, `vad_ms`, and `stt_ms` explicitly.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add explicit `e2e_ms` and measured `vad_ms` to the JSONL utterance schema** - `bea6b68` (feat)
2. **Task 2: Preserve schema stability tests and update proof doc to reference exact fields** - `bfcc141` (test)

## Files Created/Modified

- `src/pipeline/jsonl.rs` - Emits `e2e_ms` + `vad_ms` in `JsonlEvent::Utterance`
- `src/pipeline/timing.rs` - Adds `vad_ms` to `UtteranceTimings` for cross-thread propagation
- `src/vad/mod.rs` - Measures per-frame VAD compute time and accumulates it per utterance
- `tests/jsonl_schema.rs` - Locks required keys/types including `e2e_ms` + `vad_ms`
- `docs/latency-baseline.md` - Documents exact field names and formula for Phase 2 proof runs

## Decisions Made

None beyond the plan’s specified definitions.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated JSONL schema fixture to compile after adding new fields**
- **Found during:** Task 1 (schema implementation verification)
- **Issue:** `tests/jsonl_schema.rs` constructed `JsonlEvent::Utterance` and failed to compile due to missing `e2e_ms`/`vad_ms` initializers.
- **Fix:** Added the new fields to the existing fixture initializers (full key/type assertions were added in Task 2 as planned).
- **Files modified:** `tests/jsonl_schema.rs`
- **Verification:** `cargo test -q`
- **Committed in:** `bea6b68` (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Required to keep the suite green; no scope creep.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Schema + docs now align with verification needs; Phase 2 proof runs can directly read `e2e_ms` and `vad_ms` from captured JSONL artifacts.

## Self-Check: PASSED

- FOUND: `.planning/phases/02-pipeline-core/02-05-SUMMARY.md`
- FOUND: `bea6b68`
- FOUND: `bfcc141`
