---
id: T02
parent: S02
milestone: M001
provides:
  - Stable stdout JSONL event schema + writer with verbosity gating
  - Monotonic + wall-clock timing helpers for stage instrumentation
  - Silero-windowed (512 samples) VAD scoring + 20ms-paced utterance segmentation producing bounded jobs
  - Shared drop-oldest bounded-queue helper with unit test coverage
requires: []
affects: []
key_files: []
key_decisions: []
patterns_established: []
observability_surfaces: []
drill_down_paths: []
duration: 5m
verification_result: passed
completed_at: 2026-04-22
blocker_discovered: false
---
# T02: 02-pipeline-core 02

**# Phase 02 Plan 02: Pipeline contracts + VAD segmentation Summary**

## What Happened

# Phase 02 Plan 02: Pipeline contracts + VAD segmentation Summary

**Added stable stdout JSONL event contracts + timing helpers, and a Silero-windowed VAD segmenter that emits bounded utterance jobs with drop-oldest backpressure semantics.**

## Performance

- **Duration:** 5m
- **Started:** 2026-04-22T12:15:18Z
- **Completed:** 2026-04-22T12:18:36Z
- **Tasks:** 2/2
- **Files modified:** 7

## Accomplishments
- Stable JSONL schema + writer that keeps stdout machine-readable and supports optional stage events behind verbosity
- Timing helpers that capture wall-clock (unix ms) + monotonic elapsed time for latency instrumentation
- Deterministic VAD segmentation enforcing preroll/tail/min-speech/end-silence/max-length, with Silero scoring via 512-sample sliding window advanced every 20ms

## Task Commits

Each task was committed atomically:

1. **Task 1: Add `pipeline` module with timing + JSONL event schema** - `37ffa0d` (feat)
2. **Task 2: Implement Silero-based VAD segmentation producing bounded utterance jobs** - `a7e6536` (feat)

**Plan metadata:** `8455e76` (docs: complete plan)

## Files Created/Modified
- `src/pipeline/jsonl.rs` - JSONL event types + `JsonlWriter` that writes one JSON object per line to a provided `Write`
- `src/pipeline/timing.rs` - wall-clock unix ms + monotonic clock helpers (`MonoClock`, `UtteranceTimings`)
- `src/vad/mod.rs` - Silero 512-sample sliding window scoring + utterance segmentation + shared drop-oldest helper
- `tests/jsonl_schema.rs` - asserts JSONL schema required fields and stable keys without any model files
- `tests/drop_oldest_queue.rs` - validates the same drop-oldest helper used in production

## Decisions Made
- Use unix milliseconds for wall-clock timestamps (allowed by D-21) to keep dependencies minimal.
- Keep segmentation unit-testable by separating "score computation" from "segmentation decisions".

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- `target/debug/hd-linux-voice` was tracked and showed as modified after builds; it was explicitly excluded from task commits by checking it out before staging.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- JSONL stdout contracts and timing primitives are ready for wiring into the pipeline coordinator and STT thread.
- VAD segmentation is ready to be driven by the existing `AudioHandle.consumer` drain loop (kept out of the CPAL callback).

## Self-Check: PASSED
