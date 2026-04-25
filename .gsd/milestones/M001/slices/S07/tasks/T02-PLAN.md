---
estimated_steps: 28
estimated_files: 1
skills_used: []
---

# T02: Add dual-init coexistence test for sherpa-onnx + silero-vad-rust

## Description

Add a new test to `tests/wake_word.rs` that initializes both a sherpa-onnx `KeywordSpotter` and silero-vad-rust's OnnxModel in sequence within the same process. This test proves the dual-ORT conflict is resolved — if both can co-initialize without panic or bad_alloc, the shared `.so` approach works.

The test is env-gated with `RUN_KWS_TESTS=1` like the existing test, since it requires local model artifacts.

## Steps

1. Add a new test function `dual_init_wake_and_vad_coexist` to `tests/wake_word.rs`.
2. The test must be `#[ignore]` with a descriptive reason and gated on `RUN_KWS_TESTS=1`.
3. The test body should:
   a. Set `ORT_DYLIB_PATH` to `env!("CARGO_MANIFEST_DIR")/target/debug/libonnxruntime.so` if not already set (mirrors the coordinator auto-discovery logic).
   b. Create a sherpa-onnx `KeywordSpotter` using the same KWS_* env vars as the existing test.
   c. Feed 1 second of silence through it (reuse existing pattern).
   d. Then call `silero_vad_rust::silero_vad::model::load_silero_vad_with_options` with `force_onnx_cpu: true`.
   e. Assert neither operation panics.
4. The test validates that two ORT consumers can coexist in the same address space.

**Note:** This test can only be confirmed by running `RUN_KWS_TESTS=1 cargo test --test wake_word -- --include-ignored` in a non-gated session with model artifacts present (MEM004). Static verification confirms the test code exists with correct structure.

## Must-Haves

- [ ] New test function `dual_init_wake_and_vad_coexist` exists in `tests/wake_word.rs`
- [ ] Test is `#[ignore]` and gated on `RUN_KWS_TESTS=1`
- [ ] Test initializes both sherpa-onnx KWS and silero-vad-rust in sequence
- [ ] Test sets `ORT_DYLIB_PATH` if not already set

## Verification

- `grep -q 'dual_init_wake_and_vad_coexist' tests/wake_word.rs` exits 0
- `grep -q 'load_silero_vad_with_options' tests/wake_word.rs` exits 0
- `grep -q 'ORT_DYLIB_PATH' tests/wake_word.rs` exits 0

## Inputs

- `tests/wake_word.rs` — existing test file to extend
- `Cargo.toml` — confirms silero-vad-rust and sherpa-onnx are both available as dependencies

## Expected Output

- `tests/wake_word.rs` — new dual-init coexistence test added

## Inputs

- ``tests/wake_word.rs` — existing test file to extend`
- ``Cargo.toml` — confirms silero-vad-rust and sherpa-onnx dependencies are available`

## Expected Output

- ``tests/wake_word.rs` — dual-init coexistence test added`

## Verification

grep -q 'dual_init_wake_and_vad_coexist' tests/wake_word.rs && grep -q 'load_silero_vad_with_options' tests/wake_word.rs && grep -q 'ORT_DYLIB_PATH' tests/wake_word.rs
