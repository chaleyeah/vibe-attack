# T04: 02-pipeline-core 04

**Slice:** S02 — **Milestone:** M001

## Description

Add the “proof artifacts” that make Phase 2 success criteria verifiable: a repeatable latency baseline procedure (with clear pass/fail criteria) and a concurrency stress test. Also update the phase validation contract to reflect Wave 0 readiness once its files exist.

## Must-Haves

- [ ] "There is a reproducible procedure to prove the Phase 2 latency budget on target hardware."
- [ ] "There is a stress test artifact proving concurrency (audio capture + recognition) under load."
- [ ] "Nyquist validation bookkeeping reflects Wave 0 readiness once the required test scaffolding exists."

## Files

- `docs/latency-baseline.md`
- `tests/concurrency_stress.rs`
- `.planning/phases/02-pipeline-core/02-VALIDATION.md`
