# S07: Wake Word Activation (DEFERRED from Phase 2) — Resolve dual ONNX Runtime conflict between sherpa Onnx (statically Linked ORT) and `ort` crate (dynamically Loaded ORT) so the wake Word path runs without heap corruption

**Goal:** Resolve the dual ONNX Runtime conflict so wake-word detection (sherpa-onnx) and VAD (silero-vad-rust/ort) can run simultaneously without heap corruption or bad_alloc crashes.
**Demo:** unit tests prove Wake-word Activation (DEFERRED from Phase 2) — Resolve dual-ONNX-Runtime conflict between sherpa-onnx (statically-linked ORT) and `ort` crate (dynamically-loaded ORT) so the wake-word path runs without heap corruption works

## Must-Haves

- `cargo build` succeeds with `sherpa-onnx` using `shared` feature (no static ORT baked in)
- `libonnxruntime.so` and `libsherpa-onnx-c-api.so` present in `target/debug/`
- `ldd target/debug/hd-linux-voice | grep onnx` shows dynamic linking to shared ORT
- `ORT_DYLIB_PATH` auto-discovery code in coordinator.rs sets the env var before Silero VAD init
- Dual-init coexistence test in `tests/wake_word.rs` exercises both sherpa-onnx KWS and silero-vad-rust in the same process (env-gated)
- ORT conflict warnings removed from `docs/troubleshooting.md` and `docs/configuration.md`
- Shared `.so` deployment requirements documented in `docs/troubleshooting.md`
- `tests/documentation.rs` assertions still pass (no references to old conflict text in test assertions)

## Proof Level

- This slice proves: - This slice proves: integration
- Real runtime required: yes (but deferred to CI/manual per MEM004 — static verification in auto-mode)
- Human/UAT required: no

## Integration Closure

- Upstream surfaces consumed: `sherpa-onnx` crate (shared feature), `silero-vad-rust` crate (ort load-dynamic), `src/pipeline/coordinator.rs` (VAD init path)
- New wiring introduced: ORT_DYLIB_PATH auto-discovery block in coordinator.rs before Silero VAD load
- What remains: AppImage packaging must bundle both .so files (documented for future S08/packaging work, MEM007 already tracks this)

## Verification

- Runtime signals: tracing::info log when ORT_DYLIB_PATH is auto-set; existing Silero VAD load timing log
- Inspection surfaces: `ldd` on binary shows shared ORT linkage; `ls target/debug/*.so` shows .so presence
- Failure visibility: catch_unwind around Silero init already produces actionable error with ORT_DYLIB_PATH guidance
- Redaction constraints: none

## Tasks

- [x] **T01: Switch sherpa-onnx to shared feature and add ORT_DYLIB_PATH auto-discovery** `est:30m`
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
  - Files: `Cargo.toml`, `src/pipeline/coordinator.rs`
  - Verify: grep 'default-features = false' Cargo.toml | grep -q sherpa && grep -q 'ORT_DYLIB_PATH' src/pipeline/coordinator.rs && grep -q 'set_var' src/pipeline/coordinator.rs

- [x] **T02: Add dual-init coexistence test for sherpa-onnx + silero-vad-rust** `est:20m`
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
  - Files: `tests/wake_word.rs`
  - Verify: grep -q 'dual_init_wake_and_vad_coexist' tests/wake_word.rs && grep -q 'load_silero_vad_with_options' tests/wake_word.rs && grep -q 'ORT_DYLIB_PATH' tests/wake_word.rs

- [ ] **T03: Update docs to remove ORT conflict warnings and document shared .so deployment** `est:20m`
  ## Description

Now that the dual-ORT conflict is resolved by the shared feature switch, update documentation to remove the stale "enable only one at a time" warnings and add deployment guidance for the shared `.so` files.

## Steps

1. In `docs/troubleshooting.md`, **Models** section (around line 89-91):
   - Remove the paragraph: "**ONNX Runtime errors:** Two concurrent ONNX Runtime instances can conflict. If you see a `bad_alloc` or ORT initialization error, ensure only one feature (STT *or* wake) is enabled at a time."
   - Replace with a note about `ORT_DYLIB_PATH` for custom installs: if `libonnxruntime.so` is not next to the binary, set `ORT_DYLIB_PATH=/path/to/libonnxruntime.so`.

2. In `docs/troubleshooting.md`, **Build** section (at the end):
   - Add a note that `cargo build` copies `libonnxruntime.so` and `libsherpa-onnx-c-api.so` to the target directory, and these must be present at runtime for wake-word and VAD to work.

3. In `docs/configuration.md`, **wake** section (around line 172-173):
   - Remove the blockquote: "> **Note:** Running both STT and wake-word simultaneously may cause ONNX Runtime conflicts. If you see initialization errors, enable only one at a time."
   - Replace with a note that `libonnxruntime.so` and `libsherpa-onnx-c-api.so` are automatically placed next to the binary at build time. For custom installs, set `ORT_DYLIB_PATH`.

4. Check `tests/documentation.rs` for any assertions that grep for the old conflict text (e.g. "only one", "conflict", "bad_alloc"). The current tests check for section headings (uinput, ptt, Installation, Usage) and project name — none reference the ORT conflict text, so no test changes should be needed. Verify this by reading the file.

## Must-Haves

- [ ] ORT conflict "ensure only one feature" warning removed from `docs/troubleshooting.md`
- [ ] ORT conflict "enable only one at a time" note removed from `docs/configuration.md`
- [ ] `ORT_DYLIB_PATH` guidance added to `docs/troubleshooting.md`
- [ ] Shared `.so` deployment note added to docs
- [ ] `tests/documentation.rs` still passes (no assertions reference removed text)

## Verification

- `! grep -qi 'ensure only one feature' docs/troubleshooting.md` exits 0
- `! grep -qi 'enable only one at a time' docs/configuration.md` exits 0
- `grep -qi 'ORT_DYLIB_PATH' docs/troubleshooting.md` exits 0
- `grep -qi 'libonnxruntime' docs/configuration.md` exits 0
- No documentation.rs test assertions reference the removed text (static check)

## Inputs

- `docs/troubleshooting.md` — ORT conflict warning to remove (Models section, lines 89-91)
- `docs/configuration.md` — ORT conflict note to remove (wake section, lines 172-173)
- `tests/documentation.rs` — verify no assertions reference removed text
- `Cargo.toml` — confirms shared feature is now used (from T01)
- `src/pipeline/coordinator.rs` — confirms ORT_DYLIB_PATH auto-discovery is in place (from T01)

## Expected Output

- `docs/troubleshooting.md` — ORT conflict warning replaced with ORT_DYLIB_PATH guidance and shared .so deployment note
- `docs/configuration.md` — ORT conflict note replaced with shared .so deployment note
  - Files: `docs/troubleshooting.md`, `docs/configuration.md`, `tests/documentation.rs`
  - Verify: ! grep -qi 'ensure only one feature' docs/troubleshooting.md && ! grep -qi 'enable only one at a time' docs/configuration.md && grep -qi 'ORT_DYLIB_PATH' docs/troubleshooting.md && grep -qi 'libonnxruntime' docs/configuration.md

## Files Likely Touched

- Cargo.toml
- src/pipeline/coordinator.rs
- tests/wake_word.rs
- docs/troubleshooting.md
- docs/configuration.md
- tests/documentation.rs
