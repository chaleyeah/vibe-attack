# T05: 02-pipeline-core 05

**Slice:** S02 — **Milestone:** M001

## Description

Close the Phase 2 latency instrumentation gaps by (1) adding an explicit end-to-end field to the JSONL utterance schema (`e2e_ms`), (2) adding a measured VAD compute duration field (`vad_ms`) that is separate from utterance timeline markers, and (3) updating docs + schema stability tests accordingly.

Purpose: Phase 2 verification requires per-stage budgets and total E2E to be directly measurable from committed artifacts without guesswork.
Output: Updated JSONL schema + updated docs + updated schema tests.

## Must-Haves

- [ ] "The stdout JSONL utterance schema includes an explicit end-to-end duration field suitable for Phase 2 proof runs."
- [ ] "VAD compute cost is measured as a duration field distinct from utterance timeline markers."
- [ ] "Schema stability tests continue to pass after adding these fields."

## Files

- `src/pipeline/jsonl.rs`
- `src/pipeline/timing.rs`
- `src/vad/mod.rs`
- `src/pipeline/coordinator.rs`
- `tests/jsonl_schema.rs`
- `docs/latency-baseline.md`
