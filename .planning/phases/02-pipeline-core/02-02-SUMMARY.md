---
phase: 02-pipeline-core
plan: 02
subsystem: pipeline
tags: [jsonl, timing, vad, silero, crossbeam-channel]

# Dependency graph
requires:
  - phase: 02-pipeline-core
    provides: "02-01: groundwork for Phase 2 module exports/tests/deps"
provides:
  - "Stable stdout JSONL event schema + writer with verbosity gating"
  - "Monotonic + wall-clock timing helpers for stage instrumentation"
  - "Silero-windowed (512 samples) VAD scoring + 20ms-paced utterance segmentation producing bounded jobs"
  - "Shared drop-oldest bounded-queue helper with unit test coverage"
affects: [stt, wake, pipeline-orchestration, latency-instrumentation]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "stdout JSONL purity: JSON writer takes explicit `Write`, diagnostics via tracing/stderr only"
    - "20ms frame clock with 512-sample Silero sliding window advanced every frame"
    - "Bounded queue backpressure via try_send + drop-oldest"

key-files:
  created:
    - src/pipeline/mod.rs
    - src/pipeline/timing.rs
    - src/pipeline/jsonl.rs
    - src/vad/mod.rs
  modified:
    - src/lib.rs
    - tests/jsonl_schema.rs
    - tests/drop_oldest_queue.rs

key-decisions:
  - "Use unix milliseconds for wall clock timestamps (allowed by D-21) to avoid new time-format deps."
  - "Keep segmentation logic deterministic/testable by accepting scored frames, while providing a Silero-scoring path built on a 512-sample sliding window."

patterns-established:
  - "Pipeline JSONL events are `#[serde(tag = \"type\")]` enums with stable key names."
  - "VAD segmentation uses hysteresis + min-speech + end-silence gating with bounded buffers (preroll/pending silence) and a max-utterance cap."

requirements-completed: [STT-04]

# Metrics
duration: 5m
completed: 2026-04-22
---

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

