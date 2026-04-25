# T01: 02-pipeline-core 01

**Slice:** S02 — **Milestone:** M001

## Description

Establish Phase 2’s dependency + configuration + test scaffolding foundation: local-only model paths for VAD/STT/wake word, and opt-in heavy tests that can be run on developer machines without breaking default `cargo test`.

Purpose: Unblock deterministic implementation in later waves while keeping CI fast and respecting “on-device only” constraints.
Output: Updated dependencies + config schema + Wave 0 test harness files (including env-gated integration tests).

## Must-Haves

- [ ] "Daemon can be configured with local wake-word and STT model paths (no network)."
- [ ] "Test harness exists to validate STT and wake-word behavior in an opt-in, model-gated way."

## Files

- `Cargo.toml`
- `src/config.rs`
- `config.example.yaml`
- `tests/drop_oldest_queue.rs`
- `tests/jsonl_schema.rs`
- `tests/stt_smoke.rs`
- `tests/wake_word.rs`
