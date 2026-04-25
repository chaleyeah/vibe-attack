---
estimated_steps: 30
estimated_files: 2
skills_used: []
---

# T01: Switch sherpa-onnx to shared feature and add ORT_DYLIB_PATH auto-discovery

## Description

Switch `sherpa-onnx` from static to shared linking in `Cargo.toml` so both sherpa-onnx and `ort` (via silero-vad-rust) share a single `libonnxruntime.so` instead of colliding on two ORT global environments. Add ORT_DYLIB_PATH auto-discovery in `coordinator.rs` so the `ort` crate finds the same `.so` that sherpa-onnx ships.

This is the core fix for the dual-ORT heap corruption that has kept `wake.enabled` defaulting to `false`.

## Steps

1. In `Cargo.toml`, change `sherpa-onnx = "1.12.39"` to `sherpa-onnx = { version = "1.12.39", default-features = false, features = ["shared"] }`.
2. In `src/pipeline/coordinator.rs`, add an ORT_DYLIB_PATH auto-discovery block **before** the Silero VAD init (before the `catch_unwind` at current line 237). The block must:
   - Check if `ORT_DYLIB_PATH` is already set (respect user override)
   - If not set, resolve `std::env::current_exe()` parent directory
   - Set `ORT_DYLIB_PATH` to `<exe_dir>/libonnxruntime.so`
   - Log with `tracing::info!` that it was auto-set
3. Verify the Cargo.toml change is syntactically correct by inspecting the dependency line.
4. Verify the coordinator.rs insertion point is before any `ort` or silero-vad-rust call.

**Critical constraint:** `ORT_DYLIB_PATH` must be set before `load_silero_vad_with_options` is called. The insertion point in coordinator.rs is between the wake-word construction (line 225-229) and the Silero VAD load (line 236-252). This is single-threaded at this point so `set_var` is safe.

**Critical constraint:** Do NOT use `std::env::set_var` from multiple threads — the call happens before any pipeline threads are spawned.

## Must-Haves

- [ ] `sherpa-onnx` dependency uses `default-features = false, features = ["shared"]`
- [ ] `ORT_DYLIB_PATH` auto-discovery block exists before Silero VAD init
- [ ] Auto-discovery respects existing `ORT_DYLIB_PATH` env var (does not overwrite)
- [ ] `tracing::info!` logs when ORT_DYLIB_PATH is auto-set

## Verification

- `grep 'default-features = false' Cargo.toml | grep -q sherpa` exits 0
- `grep -q 'ORT_DYLIB_PATH' src/pipeline/coordinator.rs` exits 0
- `grep -q 'set_var' src/pipeline/coordinator.rs` exits 0
- The `set_var` call appears before the `catch_unwind` / `load_silero_vad` call in coordinator.rs (verify line ordering)

## Inputs

- `Cargo.toml` — current sherpa-onnx dependency line to modify
- `src/pipeline/coordinator.rs` — insertion point for ORT_DYLIB_PATH auto-discovery

## Expected Output

- `Cargo.toml` — sherpa-onnx switched to shared feature
- `src/pipeline/coordinator.rs` — ORT_DYLIB_PATH auto-discovery block added before Silero VAD init

## Inputs

- ``Cargo.toml` — current sherpa-onnx dependency to modify`
- ``src/pipeline/coordinator.rs` — insertion point for ORT_DYLIB_PATH auto-discovery before Silero VAD load`

## Expected Output

- ``Cargo.toml` — sherpa-onnx dependency switched to shared feature`
- ``src/pipeline/coordinator.rs` — ORT_DYLIB_PATH auto-discovery block added before Silero VAD init`

## Verification

grep 'default-features = false' Cargo.toml | grep -q sherpa && grep -q 'ORT_DYLIB_PATH' src/pipeline/coordinator.rs && grep -q 'set_var' src/pipeline/coordinator.rs
