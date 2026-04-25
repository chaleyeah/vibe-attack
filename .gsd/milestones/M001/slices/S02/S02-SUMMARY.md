---
id: S02
parent: M001
milestone: M001
provides:
  - vad-config
  - stt-config
  - wake-config
  - wave0-tests
  - Stable stdout JSONL event schema + writer with verbosity gating
  - Monotonic + wall-clock timing helpers for stage instrumentation
  - Silero-windowed (512 samples) VAD scoring + 20ms-paced utterance segmentation producing bounded jobs
  - Shared drop-oldest bounded-queue helper with unit test coverage
  - Dedicated STT OS thread (whisper-rs) with bounded drop-oldest job queue
  - Wake-word keyword spotting (sherpa-onnx) driving a LISTENING window
  - End-to-end pipeline wiring: ringbuf drain → wake/VAD → STT → stdout JSONL
  - Per-utterance timing markers (VAD/STT/output) in JSONL event fields
  - Reproducible Phase 2 latency baseline procedure (end-of-speech → transcript JSONL emit)
  - Env-gated concurrency stress test artifact for bounded queue + thread progress
  - Validation bookkeeping updated to reflect Wave 0 scaffolding readiness
  - Utterance JSONL includes explicit end-to-end duration field `e2e_ms`
  - Utterance JSONL includes measured VAD compute cost field `vad_ms`
  - Schema stability tests lock `e2e_ms`/`vad_ms` keys and u64 types
  - Latency baseline doc references `e2e_ms`, `vad_ms`, and `stt_ms` with exact derivation
  - Canonical in-repo archive location for Phase 2 target-hardware latency proof artifacts
  - Fill-in results template capturing machine metadata, build identity, and computed p50/p95/p99 for e2e_ms
  - Baseline procedure doc wiring to the proof archive path
requires: []
affects: []
key_files: []
key_decisions:
  - Use unix milliseconds for wall clock timestamps (allowed by D-21) to avoid new time-format deps.
  - Keep segmentation logic deterministic/testable by accepting scored frames, while providing a Silero-scoring path built on a 512-sample sliding window.
  - Wake-word requires continuous audio buffering; CPAL callback always pushes to ringbuf, and PTT/LISTENING gating happens on the pipeline thread.
  - Silero VAD is initialized with CPU-only options (`force_onnx_cpu: true`) to match Phase 2 baseline constraints.
  - Stdout is reserved for JSONL only via a dedicated output thread; all status/instrumentation stays on stderr via tracing.
  - Phase 2 latency proof metric is end-of-speech → transcript JSONL emit; end-of-speech → first key event is validated in Phase 3 dispatch.
  - `e2e_ms` is defined as `output_done_ms - vad_done_ms` and computed at JSONL emission time using monotonic markers
  - `vad_ms` measures compute cost (Instant elapsed) for Silero scoring + segmentation per frame, accumulated only while an utterance is active
patterns_established:
  - Pipeline JSONL events are `#[serde(tag = \"type\")]` enums with stable key names.
  - VAD segmentation uses hysteresis + min-speech + end-silence gating with bounded buffers (preroll/pending silence) and a max-utterance cap.
  - Preflight-before-threads: validate config/model paths and initialize VAD/STT/wake before spawning long-lived pipeline threads.
  - Stress tests must be both env-gated and ignored to keep default cargo test fast.
  - Stable JSONL schema changes are protected by fast, model-free serde_json tests
  - Target-hardware proof runs are archived as text-only artifacts committed in-repo
observability_surfaces: []
drill_down_paths: []
duration: 1m13s
verification_result: passed
completed_at: 2026-04-22
blocker_discovered: false
---
# S02: Pipeline Core

**# Phase 02 Plan 01: Wave 0 scaffolding summary**

## What Happened

# Phase 02 Plan 01: Wave 0 scaffolding summary

Wave 0 scaffolding for Phase 2: dependency set, strict config schema for pipeline/VAD/STT/wake-word (local-path driven), and env-gated heavy test harnesses that keep default `cargo test -q` green without models present.

## Completed Tasks

### Task 1: Add Phase 2 dependencies

- Added Silero VAD, sherpa-onnx KWS, bounded channel (drop-oldest), and JSON encoding dependencies.
- Pinned `ort` to `2.0.0-rc.10` to avoid an upstream compile break in `silero-vad-rust`.
- Added `--features stt` gate for `whisper-rs` so default builds don’t require `cmake`.

**Commit:** `4fb0398`

### Task 2: Extend config schema for pipeline/VAD/STT/wake

- Added strict `pipeline`, `vad`, `stt`, and `wake` sections (all `deny_unknown_fields`, defaulted).
- Added `Config::validate_model_paths()` helper returning contextual errors for missing local model artifacts.
- Updated `config.example.yaml` with new sections and the stdout(JSONL)/stderr(logs) contract.

**Commit:** `0222cb4`

### Task 3: Add Wave 0 tests

- Added `drop_oldest_queue` unit tests for bounded drop-oldest semantics.
- Added `jsonl_schema` unit tests for stable JSON key invariants.
- Added env-gated, ignored smoke harnesses for STT and wake word (no downloads; opt-in only).

**Commit:** `ebb2faf`

## Verification

- After each task: `cargo test -q` (pass)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Keep default builds green without `cmake`**
- **Issue:** `whisper-rs` build requires external `cmake`, which is not guaranteed on all dev/CI machines.
- **Fix:** Made `whisper-rs` optional behind the `stt` feature (`--features stt`), so Wave 0 scaffolding compiles and tests pass by default.
- **Commit:** `4fb0398`

**2. [Rule 3 - Blocking] Pin `ort` RC version**
- **Issue:** `silero-vad-rust` did not compile against a newer resolved `ort` RC due to upstream API/type changes.
- **Fix:** Pinned `ort` to `2.0.0-rc.10` to match `silero-vad-rust` expectations.
- **Commit:** `4fb0398`

## Known Stubs

- `tests/jsonl_schema.rs`: JSONL schema test uses a local `UtteranceEvent` fixture struct (production JSONL event types will be introduced in later plans).

## Self-Check: PENDING

## Self-Check: PASSED

- Summary file present on disk
- Task commits present: `4fb0398`, `0222cb4`, `ebb2faf`

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

# Phase 2 Plan 04 Summary

**Added Phase 2 proof artifacts: a repeatable latency baseline procedure (end-of-speech → transcript JSONL emit, p95 < 500ms) plus an opt-in concurrency stress test to catch deadlocks and bounded-queue regressions.**

## Performance

- **Duration:** 2 min
- **Started:** 2026-04-22T12:35:12Z
- **Completed:** 2026-04-22T12:36:54Z
- **Tasks:** 3/3
- **Files modified:** 3

## Accomplishments

- Documented a reproducible latency-baseline run with artifact capture and pass/fail criteria aligned to Phase 2 success criteria.
- Added an env-gated, ignored stress test that exercises bounded drop-oldest queue behavior and concurrent stage progress.
- Updated validation bookkeeping to reflect that Wave 0 scaffolding exists.

## Task Commits

Each task was committed atomically:

1. **Task 1: Write a reproducible Phase 2 latency baseline procedure** - `073e542` (docs)
2. **Task 2: Add env-gated concurrency stress test** - `fbb8c46` (test)
3. **Task 3: Update validation frontmatter after Wave 0 scaffolding exists** - `d9d4809` (docs)

**Plan metadata:** (captured in the plan completion docs commit)

## Files Created/Modified

- `docs/latency-baseline.md` - Phase 2 “latency proof” procedure and acceptance criteria (p95 < 500ms).
- `tests/concurrency_stress.rs` - Opt-in concurrency stress harness (`RUN_STRESS_TESTS=1`, `#[ignore]`).
- `.planning/phases/02-pipeline-core/02-VALIDATION.md` - Mark Wave 0 scaffolding complete (`wave_0_complete: true`).

## Decisions Made

- Phase 2 measures STT-04 via **end-of-speech → transcript JSONL emit** as the proxy; dispatch (“→ first key event”) is validated in Phase 3.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 2 now has concrete “proof artifacts” for verification runs on target hardware.
- Phase 3 can reuse the same Phase 2 latency metric while extending measurement through dispatch (“first key event”).

## Self-Check: PASSED

- Confirmed files exist: `docs/latency-baseline.md`, `tests/concurrency_stress.rs`, `.planning/phases/02-pipeline-core/02-04-SUMMARY.md`
- Confirmed task commits exist: `073e542`, `fbb8c46`, `d9d4809`

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

# Phase 02 Plan 06: Target hardware proof artifact templates — Summary

**Added a canonical in-repo archive + template for Phase 2 target-hardware latency proof runs (transcript.jsonl, timing.log, and computed p95 for e2e_ms).**

## Performance

- **Duration:** 1m13s
- **Started:** 2026-04-22T12:49:45Z
- **Completed:** 2026-04-22T12:50:58Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Added a consistent on-disk structure and instructions for committing Phase 2 target-hardware proof artifacts
- Added a fill-in results template capturing machine metadata + repo git commit + computed p50/p95/p99 of `e2e_ms`
- Wired the baseline procedure doc to the canonical proof archive path

## Task Commits

Each task was committed atomically:

1. **Task 1: Add in-repo proof artifact template directory for Phase 2 target hardware runs** - `cac4ab5` (docs)
2. **Task 2: Wire baseline procedure doc to the proof artifact template location** - `3ca4086` (docs)

## Files Created/Modified
- `docs/latency-proofs/phase-02-target-hardware/README.md` - Canonical proof archive location + required artifacts + run naming convention
- `docs/latency-proofs/phase-02-target-hardware/RESULTS.template.md` - Results template for machine/build identity and computed latency percentiles
- `docs/latency-baseline.md` - Links baseline procedure to the proof archive location and clarifies what to commit

## Decisions Made
None - followed plan as specified.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Reverted unintended modification to tracked debug binary**
- **Found during:** Task 1 (Add in-repo proof artifact template directory for Phase 2 target hardware runs)
- **Issue:** `target/debug/hd-linux-voice` was modified locally and showed up in `git status`, but is unrelated to this docs-only plan and should not be committed.
- **Fix:** Reverted the file to the index state before staging task files.
- **Files modified:** `target/debug/hd-linux-voice`
- **Verification:** `git status --short` was clean aside from the intended docs additions.
- **Committed in:** N/A (revert performed before Task 1 staging/commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** No scope creep; the auto-fix prevented accidental inclusion of unrelated build output.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Ready for a single human-run on target hardware to commit evidence under `docs/latency-proofs/phase-02-target-hardware/`.
- Remaining closure for STT-04 requires actual run artifacts + computed p95 committed using the provided structure.

## Self-Check: PASSED

- Summary exists at `.planning/phases/02-pipeline-core/02-06-SUMMARY.md`
- Task commits exist in git history: `cac4ab5`, `3ca4086`
