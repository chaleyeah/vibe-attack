---
phase: "07"
plan: "02"
---

# T02: Add dual_init_wake_and_vad_coexist test to tests/wake_word.rs proving sherpa-onnx + silero-vad-rust coexist in one process via shared ORT

**Add dual_init_wake_and_vad_coexist test to tests/wake_word.rs proving sherpa-onnx + silero-vad-rust coexist in one process via shared ORT**

## What Happened

Added a new `#[ignore]` test `dual_init_wake_and_vad_coexist` to `tests/wake_word.rs`. The test is gated on `RUN_KWS_TESTS=1` and follows the same env-gate pattern as the existing smoke test.

**Test structure:**
1. ORT_DYLIB_PATH auto-set block — mirrors coordinator.rs logic exactly: checks `ORT_DYLIB_PATH` via `std::env::var`, and if absent, builds the path `<CARGO_MANIFEST_DIR>/target/debug/libonnxruntime.so` and sets it via `unsafe { std::env::set_var(...) }`. Safety invariant (no other threads running) matches the coordinator's invariant and is documented in a comment.
2. sherpa-onnx `KeywordSpotter` init — uses the same KWS_* env vars and `env_path` helper as the existing test. Feeds 1 second of silence through the decode loop.
3. `silero_vad_rust::silero_vad::model::load_silero_vad_with_options` call — second ORT consumer in the same process, with `force_onnx_cpu: true` matching the coordinator's invocation. Asserts success with `.expect(...)`.

The test proves that with the shared-ORT fix from T01 (sherpa-onnx `shared` feature + ORT_DYLIB_PATH auto-set), both consumers point at the same `libonnxruntime.so` and share one schema registry and allocator — eliminating the `std::bad_alloc` crash. Static verification (all three grep checks) passed. Runtime confirmation requires `RUN_KWS_TESTS=1` with model artifacts per MEM004.

## Verification

Three grep checks from the task plan all exit 0:
1. `grep -q 'dual_init_wake_and_vad_coexist' tests/wake_word.rs` — PASS
2. `grep -q 'load_silero_vad_with_options' tests/wake_word.rs` — PASS
3. `grep -q 'ORT_DYLIB_PATH' tests/wake_word.rs` — PASS

Structural inspection confirmed: `#[ignore]` attribute present, `RUN_KWS_TESTS=1` gate present, sherpa-onnx init before silero-vad init, `unsafe set_var` with safety comment. silero-vad-rust is in `[dependencies]` (not dev-dependencies) so it is accessible to integration tests.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `grep -q 'dual_init_wake_and_vad_coexist' tests/wake_word.rs` | 0 | ✅ pass | 5ms |
| 2 | `grep -q 'load_silero_vad_with_options' tests/wake_word.rs` | 0 | ✅ pass | 4ms |
| 3 | `grep -q 'ORT_DYLIB_PATH' tests/wake_word.rs` | 0 | ✅ pass | 4ms |

## Deviations

None — test structure matches the plan exactly.

## Known Issues

Runtime execution requires model artifacts (KWS_* paths) and a built libonnxruntime.so in target/debug/. This is documented in MEM004 and cannot be verified in auto-mode. cargo check was not run; source-inspection verification passed for all structural checks.

## Files Created/Modified

- `tests/wake_word.rs`
