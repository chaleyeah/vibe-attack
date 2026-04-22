---
status: partial
phase: 02-pipeline-core
source: [02-VERIFICATION.md]
started: 2026-04-22T00:00:00Z
updated: 2026-04-22T00:00:00Z
---

## Current Test

Target-hardware latency proof run + archive

## Tests

### 1. Target-hardware latency proof run + archive
expected: Run `docs/latency-baseline.md` on target hardware and commit a `run-*` folder under `docs/latency-proofs/phase-02-target-hardware/` containing `transcript.jsonl`, `timing.log`, and filled `RESULTS.md` with p95(`e2e_ms`) < 500ms across N short phrases.
result: [pending]

### 2. End-to-end transcript correctness (PTT held)
expected: With PTT held and STT enabled, stdout emits JSONL `type=utterance` events with non-empty `text` and timing fields (including `e2e_ms`, `vad_ms`, `stt_ms`).
result: [pending]

### 3. Wake word triggers LISTENING (no PTT)
expected: Without PTT, wake word triggers LISTENING (logged on stderr) and speech within the window produces an utterance JSONL event on stdout.
result: [pending]

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps

