# S07: Wake Word Activation (DEFERRED from Phase 2) — Resolve dual ONNX Runtime conflict — UAT

**Milestone:** M001
**Written:** 2026-04-25T20:13:08.208Z

# S07 UAT — Wake-word / VAD Dual-ORT Coexistence

## Preconditions

- Rust toolchain installed, `cargo build` completes successfully
- `libonnxruntime.so` and `libsherpa-onnx-c-api.so` present in `target/debug/` after build
- Local KWS model artifacts available (paths set in `KWS_MODEL_DIR`, `KWS_TOKENS`, `KWS_KEYWORDS`)
- `ORT_DYLIB_PATH` **not** set in the shell environment before tests (to exercise auto-discovery)

---

## Test Case 1 — Shared feature in Cargo.toml

**Steps:**
1. Run: `grep 'default-features = false' Cargo.toml | grep sherpa`

**Expected:** Line matches `sherpa-onnx = { version = "1.12.39", default-features = false, features = ["shared"] }` — exit code 0.

---

## Test Case 2 — ORT_DYLIB_PATH auto-discovery in coordinator.rs

**Steps:**
1. Run: `grep -n 'ORT_DYLIB_PATH' src/pipeline/coordinator.rs`
2. Run: `grep -n 'catch_unwind\|load_silero_vad' src/pipeline/coordinator.rs`

**Expected:** The `ORT_DYLIB_PATH` / `set_var` lines appear at a lower line number than the `catch_unwind` / `load_silero_vad_with_options` lines — confirming the env var is set before VAD is initialized.

---

## Test Case 3 — Dynamic linking confirmed after build

**Steps:**
1. Run: `cargo build 2>&1 | tail -5`
2. Run: `ldd target/debug/hd-linux-voice | grep onnx`
3. Run: `ls target/debug/libonnxruntime.so target/debug/libsherpa-onnx-c-api.so`

**Expected:**
- `cargo build` exits 0
- `ldd` output shows `libonnxruntime.so` dynamically linked (not "not found")
- Both `.so` files exist in `target/debug/`

---

## Test Case 4 — ORT_DYLIB_PATH auto-set log at startup

**Steps:**
1. Run the binary with `RUST_LOG=info` and wake-word enabled in config: `RUST_LOG=info ./target/debug/hd-linux-voice`
2. Observe startup logs

**Expected:** A `tracing::info` log line appears containing `ORT_DYLIB_PATH` and the auto-set path (e.g. `/path/to/target/debug/libonnxruntime.so`).

---

## Test Case 5 — User ORT_DYLIB_PATH override is respected

**Steps:**
1. Run: `ORT_DYLIB_PATH=/custom/path/libonnxruntime.so RUST_LOG=info ./target/debug/hd-linux-voice`
2. Observe startup logs

**Expected:** The auto-discovery log line does NOT appear (existing value is respected); the binary uses `/custom/path/libonnxruntime.so`.

---

## Test Case 6 — Dual-init coexistence test (runtime, requires model artifacts)

**Steps:**
1. Set KWS env vars pointing to local model files.
2. Run: `RUN_KWS_TESTS=1 cargo test --test wake_word -- dual_init_wake_and_vad_coexist --include-ignored 2>&1`

**Expected:** Test passes (exit 0). No panic, no `bad_alloc`, no ORT initialization error. Both `KeywordSpotter` and silero-vad model load successfully in the same process.

---

## Test Case 7 — Stale conflict warnings removed from docs

**Steps:**
1. Run: `grep -i 'ensure only one\|enable only one at a time\|bad_alloc' docs/troubleshooting.md docs/configuration.md`

**Expected:** Zero matches — old conflict warnings are gone.

---

## Test Case 8 — New deployment guidance present in docs

**Steps:**
1. Run: `grep -i 'ORT_DYLIB_PATH' docs/troubleshooting.md`
2. Run: `grep -i 'libonnxruntime' docs/configuration.md`

**Expected:** At least one match in each file — deployment guidance is present.

---

## Test Case 9 — documentation.rs tests still pass

**Steps:**
1. Run: `cargo test --test documentation 2>&1`

**Expected:** All tests pass (exit 0). No assertions reference the removed conflict text.

---

## Edge Cases

- **Missing .so at runtime:** If `libonnxruntime.so` is absent and `ORT_DYLIB_PATH` is unset, Silero VAD init should fail inside `catch_unwind` and produce an actionable error message with `ORT_DYLIB_PATH` guidance rather than a bare panic.
- **Stale build artifacts:** If `target/debug/` contains an old static-linked binary, re-run `cargo clean && cargo build` before testing dynamic linking.
- **AppImage distribution:** Packaged AppImage must include both `libonnxruntime.so` and `libsherpa-onnx-c-api.so` alongside the binary — verify with `ldd` inside the AppImage mount point before publishing.
