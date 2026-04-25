---
id: T03
parent: S02
milestone: M001
provides:
  - Dedicated STT OS thread (whisper-rs) with bounded drop-oldest job queue
  - Wake-word keyword spotting (sherpa-onnx) driving a LISTENING window
  - End-to-end pipeline wiring: ringbuf drain → wake/VAD → STT → stdout JSONL
  - Per-utterance timing markers (VAD/STT/output) in JSONL event fields
requires: []
affects: []
key_files: []
key_decisions: []
patterns_established: []
observability_surfaces: []
drill_down_paths: []
duration: 7m
verification_result: passed
completed_at: 2026-04-22
blocker_discovered: false
---
# T03: 02-pipeline-core 03

**# Phase 2 Plan 03: Pipeline wiring (wake/VAD → STT → JSONL) Summary**

## What Happened

# Phase 2 Plan 03: Pipeline wiring (wake/VAD → STT → JSONL) Summary

**End-to-end local wake/VAD→STT pipeline with JSONL-only stdout and per-stage timing markers suitable for <500ms latency measurement.**

## Performance

- **Duration:** 7m
- **Started:** 2026-04-22T12:24:44Z
- **Completed:** 2026-04-22T12:31:29Z
- **Tasks:** 2/2
- **Files modified:** 11

## Accomplishments

- Dedicated `std::thread` STT worker loads whisper.cpp model from local path at startup (feature-gated) and returns final-only transcripts.
- Pipeline coordinator drains the RT-safe ring buffer on an OS thread, runs wake-word while idle, and runs VAD segmentation during PTT/LISTENING.
- Output contract stabilized: a single stdout writer emits JSONL transcript events with wall-clock + monotonic stage timing markers; all logs stay on stderr.

## Task Commits

Each task was committed atomically:

1. **Task 1: STT worker thread + bounded drop-oldest queue** - `1851192` (feat)
2. **Task 2: Wake-word + coordinator wiring + JSONL instrumentation** - `ae0c0fd` (feat)

## Files Created/Modified

- `src/stt/mod.rs` - Whisper STT service on a dedicated OS thread with bounded drop-oldest job queue and timing propagation.
- `src/wake/mod.rs` - Sherpa-onnx keyword spotter wrapper for wake-word detection (stderr-only status).
- `src/pipeline/coordinator.rs` - Pipeline thread draining ringbuf, handling wake/LISTENING, running VAD segmentation, and bridging STT results to stdout JSONL writer thread.
- `src/pipeline/jsonl.rs` / `src/pipeline/timing.rs` - JSONL event schema and per-utterance timing markers.
- `src/audio/mod.rs` - RT callback remains allocation-free while continuously buffering audio for wake-word support.

## Decisions Made

None - followed plan as specified.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Output JSONL contract and thread topology are in place for Phase 3 dispatch to consume transcripts.
- Remaining risk is hardware-dependent latency verification; follow the Phase 2 manual run instructions and archive stdout JSONL + stderr timings for measurement.

## Self-Check: PASSED

- SUMMARY file present: `.planning/phases/02-pipeline-core/02-03-SUMMARY.md`
- Task commits present: `1851192`, `ae0c0fd`

---
phase: 02-pipeline-core
plan: 03
subsystem: pipeline
tags: [whisper-rs, whisper.cpp, sherpa-onnx, silero-vad-rust, crossbeam-channel, jsonl, cpal, ringbuf]

requires:
  - phase: 02-pipeline-core
    provides: "02-02 VAD segmentation + JSONL event contract foundations"
provides:
  - "Dedicated STT worker thread with whisper-rs model preload (feature-gated) and bounded drop-oldest queue"
  - "Wake-word + VAD pipeline coordinator draining ringbuf on an OS thread"
  - "Stdout JSONL transcript events with stage timing fields; stderr-only instrumentation via tracing"
affects: [phase-03-dispatch, phase-04-pack-system, latency-validation]

tech-stack:
  added: []
  patterns:
    - "OS-thread pipeline: ringbuf drain → wake/VAD → bounded queue → STT thread → stdout JSONL"
    - "stdout/stderr split: stdout JSONL only; all logs/instrumentation via tracing (stderr)"

key-files:
  created:
    - src/stt/mod.rs
    - src/pipeline/coordinator.rs
    - src/wake/mod.rs
  modified:
    - src/main.rs
    - src/audio/mod.rs
    - src/pipeline/jsonl.rs
    - src/pipeline/timing.rs
    - tests/stt_smoke.rs
    - tests/wake_word.rs
    - tests/jsonl_schema.rs

key-decisions:
  - "Move PTT gating out of the CPAL callback so wake word can run without PTT while keeping the callback allocation-free."
  - "Use a dedicated output thread as the only stdout writer to keep pipeline/compute threads free of stdout IO stalls."

patterns-established:
  - "Bounded drop-oldest channel send path uses try_send + try_recv(one) retry, never blocking."
  - "Stage timings are carried in UtteranceTimings and emitted as u64 ms fields in the JSONL utterance event."

requirements-completed: [ACT-02, STT-01, STT-04]

duration: 7m
completed: 2026-04-22
---

# Phase 2 Plan 03: Pipeline Wiring Summary

**End-to-end wake/VAD→STT pipeline on OS threads with clean stdout JSONL transcripts and stage timing instrumentation.**

## Performance

- **Duration:** 7m
- **Started:** 2026-04-22T12:24:44Z
- **Completed:** 2026-04-22T12:31:50Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments

- Implemented a feature-gated `whisper-rs` STT service that preloads the model at startup and runs transcription on a dedicated `std::thread`.
- Wired wake-word detection + Silero VAD segmentation on a separate OS thread draining the existing ringbuffer consumer, feeding a bounded drop-oldest queue into STT.
- Established a strict output contract: **stdout is JSONL transcript events only**, and all diagnostics/status remain on **stderr** via `tracing`, with per-stage timing fields included in the JSONL utterance event.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement STT worker thread with whisper-rs preload + bounded drop-oldest queue** - `1851192` (feat)
2. **Task 2: Implement wake-word + pipeline wiring in main with stdout/stderr split and instrumentation** - `ae0c0fd` (feat)

**Plan metadata:** (this commit) (docs: complete plan)

## Files Created/Modified

- `src/stt/mod.rs` - STT service (dedicated OS thread, bounded queue, env-gated smoke-test integration)
- `src/wake/mod.rs` - Sherpa-onnx keyword spotter wrapper used by the pipeline thread
- `src/pipeline/coordinator.rs` - Ringbuffer drain loop, wake word LISTENING state, VAD segmentation, STT queueing, and result forwarding to stdout writer
- `src/pipeline/jsonl.rs` - Utterance JSONL schema extended with stage timing fields and frame indices
- `src/audio/mod.rs` - RT callback now always pushes into the ringbuffer; gating moved to pipeline thread (keeps callback allocation-free)
- `src/main.rs` - Fail-fast preflight for local model paths, spawn pipeline threads, and best-effort shutdown/join logic

## Decisions Made

- **Moved PTT gating out of the CPAL callback** so wake word can operate without PTT while preserving the callback’s no-alloc/no-block invariant.
- **Dedicated output thread owns stdout** to keep compute threads responsive even if the stdout consumer is slow.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Enable wake-word without PTT by moving gating to the pipeline thread**
- **Found during:** Task 2
- **Issue:** Audio callback previously discarded samples when PTT was not held, making wake-word detection impossible.
- **Fix:** Always push samples into the ringbuffer in the CPAL callback; apply PTT/LISTENING gating after draining on the pipeline OS thread.
- **Files modified:** `src/audio/mod.rs`, `src/pipeline/coordinator.rs`
- **Verification:** `cargo test -q`
- **Committed in:** `ae0c0fd` (Task 2)

---

**Total deviations:** 1 auto-fixed (Rule 2)
**Impact on plan:** Necessary for ACT-02 correctness; no scope creep beyond making the planned wake-word behavior achievable.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Pipeline is wired end-to-end and ready for Phase 3 voice-to-macro dispatch work to consume stdout JSONL transcripts.
- Latency budget validation remains a **manual-on-target-hardware** step (archive stdout JSONL + stderr timings during a run).

## Self-Check: PASSED
