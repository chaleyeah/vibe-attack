---
id: T01
parent: S02
milestone: M001
provides:
  - vad-config
  - stt-config
  - wake-config
  - wave0-tests
requires: []
affects: []
key_files: []
key_decisions: []
patterns_established: []
observability_surfaces: []
drill_down_paths: []
duration: 
verification_result: passed
completed_at: 
blocker_discovered: false
---
# T01: 02-pipeline-core 01

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
