# S02: Pipeline Core

**Goal:** Establish Phase 2’s dependency + configuration + test scaffolding foundation: local-only model paths for VAD/STT/wake word, and opt-in heavy tests that can be run on developer machines without breaking default `cargo test`.
**Demo:** Establish Phase 2’s dependency + configuration + test scaffolding foundation: local-only model paths for VAD/STT/wake word, and opt-in heavy tests that can be run on developer machines without breaking default `cargo test`.

## Must-Haves


## Tasks

- [x] **T01: 02-pipeline-core 01**
  - Establish Phase 2’s dependency + configuration + test scaffolding foundation: local-only model paths for VAD/STT/wake word, and opt-in heavy tests that can be run on developer machines without breaking default `cargo test`.

Purpose: Unblock deterministic implementation in later waves while keeping CI fast and respecting “on-device only” constraints.
Output: Updated dependencies + config schema + Wave 0 test harness files (including env-gated integration tests).
- [x] **T02: 02-pipeline-core 02** `est:5m`
  - Create the core pipeline contracts and VAD segmentation implementation: stable JSONL event schema (stdout), timing capture utilities (monotonic + wall clock), and Silero-based utterance detection/assembly that yields bounded, measurable jobs for STT.

Purpose: Make the pipeline observable and bounded so end-to-end latency work is measurable and resilient under load.
Output: `pipeline` module (jsonl + timing + coordinator skeleton) and `vad` module that produces utterance jobs with stage timing hooks.
- [x] **T03: 02-pipeline-core 03** `est:7m`
  - Wire the full Phase 2 pipeline end-to-end: drain audio from the existing RT-safe ringbuffer, run wake-word + VAD on an OS thread, push bounded utterance jobs (drop-oldest) to a dedicated blocking STT OS thread (whisper-rs), and emit final transcript JSONL events to stdout with stage timing instrumentation to stderr.

Purpose: Prove the pipeline meets on-device + <500ms budget constraints and establishes the output contract consumed by later phases.
Output: `stt` and `wake` modules, plus `main.rs` wiring that honors all thread/IO constraints.
- [x] **T04: 02-pipeline-core 04** `est:2min`
  - Add the “proof artifacts” that make Phase 2 success criteria verifiable: a repeatable latency baseline procedure (with clear pass/fail criteria) and a concurrency stress test. Also update the phase validation contract to reflect Wave 0 readiness once its files exist.
- [x] **T05: 02-pipeline-core 05** `est:10m`
  - Close the Phase 2 latency instrumentation gaps by (1) adding an explicit end-to-end field to the JSONL utterance schema (`e2e_ms`), (2) adding a measured VAD compute duration field (`vad_ms`) that is separate from utterance timeline markers, and (3) updating docs + schema stability tests accordingly.

Purpose: Phase 2 verification requires per-stage budgets and total E2E to be directly measurable from committed artifacts without guesswork.
Output: Updated JSONL schema + updated docs + updated schema tests.
- [x] **T06: 02-pipeline-core 06** `est:1m13s`
  - Close the “target hardware proof artifacts” gap by adding an in-repo place to store evidence and a template that captures the required outputs (stdout JSONL, stderr timing logs, and computed p95) so Phase 2 can be marked complete after a single human run.

Purpose: Verification requires committed evidence, not just a runbook.
Output: Proof artifact templates + doc wiring to the baseline procedure.

## Files Likely Touched

- `Cargo.toml`
- `src/config.rs`
- `config.example.yaml`
- `tests/drop_oldest_queue.rs`
- `tests/jsonl_schema.rs`
- `tests/stt_smoke.rs`
- `tests/wake_word.rs`
- `src/lib.rs`
- `src/pipeline/mod.rs`
- `src/pipeline/jsonl.rs`
- `src/pipeline/timing.rs`
- `src/vad/mod.rs`
- `tests/jsonl_schema.rs`
- `tests/drop_oldest_queue.rs`
- `src/main.rs`
- `src/pipeline/mod.rs`
- `src/stt/mod.rs`
- `src/wake/mod.rs`
- `src/lib.rs`
- `tests/stt_smoke.rs`
- `tests/wake_word.rs`
- `docs/latency-baseline.md`
- `tests/concurrency_stress.rs`
- `.planning/phases/02-pipeline-core/02-VALIDATION.md`
- `src/pipeline/jsonl.rs`
- `src/pipeline/timing.rs`
- `src/vad/mod.rs`
- `src/pipeline/coordinator.rs`
- `tests/jsonl_schema.rs`
- `docs/latency-baseline.md`
- `docs/latency-baseline.md`
- `docs/latency-proofs/phase-02-target-hardware/README.md`
- `docs/latency-proofs/phase-02-target-hardware/RESULTS.template.md`
