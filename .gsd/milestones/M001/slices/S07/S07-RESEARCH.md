# S07 Research: Wake Word Activation — Dual ONNX Runtime Conflict Resolution

**Researched:** 2026-04-25
**Domain:** ONNX Runtime dual-instance conflict between sherpa-onnx (statically-linked ORT) and `ort` crate (dynamically-loaded ORT)
**Confidence:** HIGH — root cause is fully characterised from crate source; resolution path is well-understood

---

## Summary

The wake-word path is **code-complete** (`src/wake/mod.rs`, `src/pipeline/coordinator.rs`) but disabled by default in config because initialising both sherpa-onnx and silero-vad-rust in the same process causes a `bad_alloc` / ORT initialisation crash. This happens because:

1. **sherpa-onnx** (static feature, the default) statically links its own copy of ONNX Runtime into the binary at link time.
2. **silero-vad-rust** depends on the `ort` crate with `load-dynamic` feature, which `dlopen`s `libonnxruntime.so` at runtime into the same address space.

Two ORT global runtimes collide — the static initialisation in sherpa-onnx's static archive and the dynamic global in the `dlopen`'d `.so` both try to own the singleton ORT environment and global allocators, producing heap corruption / `bad_alloc` after a small number of decode calls.

The resolution requires making **both consumers share a single ORT instance**. There are three viable paths, ordered by preference.

---

## Root Cause: Precise Mechanism

### sherpa-onnx static linking

`sherpa-onnx-sys` build.rs downloads a prebuilt static archive (`sherpa-onnx-v{version}-linux-x64-static-lib.tar.bz2`) containing:

```
SHERPA_ONNX_STATIC_LIBS = [
    "sherpa-onnx-c-api", "sherpa-onnx-core", "kaldi-decoder-core",
    "sherpa-onnx-kaldifst-core", "sherpa-onnx-fstfar", "sherpa-onnx-fst",
    "kaldi-native-fbank-core", "kissfft-float", "piper_phonemize",
    "espeak-ng", "ucd", "onnxruntime",  ← ORT baked in here
    "ssentencepiece_core"
]
```

The `onnxruntime` static archive is linked directly into the binary. The ORT global environment is initialised the first time any sherpa-onnx API is called.

### silero-vad-rust / ort load-dynamic

`silero-vad-rust 6.2.1` depends on `ort = "2.0.0-rc.10"` with `features = ["load-dynamic", "ndarray"]`. The `load-dynamic` feature causes ort to skip linking against any ORT native library at build time and instead use `libloading` to `dlopen("libonnxruntime.so")` at the first call to `ort::init()` or the first session creation. This loads a second, independent ORT global environment into the address space alongside the statically-baked one.

Both ORT instances try to:
- Register ONNX opsets in a global registry
- Manage the ONNX Runtime global allocator (defaulting to the ORT arena allocator that holds a large pre-allocated arena)
- Own the C++ global heap singletons in ORT

Result: `std::bad_alloc` or silent memory corruption, typically on the second or third decode cycle.

---

## Existing Codebase State

| File | Status |
|------|--------|
| `src/wake/mod.rs` | Complete. `WakeWord` struct wraps `sherpa_onnx::KeywordSpotter`. Works when enabled alone. |
| `src/pipeline/coordinator.rs:225` | Wake word constructed with `if config.wake.enabled`. Silero VAD loaded at line 237 with `catch_unwind`. Both on the same pipeline thread. |
| `src/config.rs` | `WakeConfig.enabled` defaults to `false`. Wake is off by default — this is the existing mitigation. |
| `docs/troubleshooting.md` | Documents: "Two concurrent ONNX Runtime instances can conflict. If you see a `bad_alloc` or ORT initialization error, ensure only one feature (STT *or* wake) is enabled at a time." |
| `docs/configuration.md` | Notes the conflict under the `wake` section. |
| `tests/wake_word.rs` | Env-gated `#[ignore]` smoke test. Only tests sherpa-onnx in isolation; does not exercise the dual-ORT conflict path. |
| `Cargo.toml` | `sherpa-onnx = "1.12.39"` (static, default). `ort = "=2.0.0-rc.10"` (pinned for silero-vad-rust compat). `silero-vad-rust = "6.2.1"`. |

The current workaround (document the conflict, default `wake.enabled = false`) is stable for v1 but prevents simultaneous use of VAD+wake — which is required for the real pipeline (wake triggers LISTENING, then VAD segments the utterance).

---

## Resolution Strategies

### Option A (Recommended): Switch sherpa-onnx to `shared` feature (two `.so` files, same ABI)

**Mechanism:** `sherpa-onnx` exposes a `shared` feature. With `sherpa-onnx = { version = "1.12.39", default-features = false, features = ["shared"] }`, the build.rs downloads `sherpa-onnx-v{version}-linux-x64-shared-lib.tar.bz2` instead of the static archive. This produces `libsherpa-onnx-c-api.so` and `libonnxruntime.so` as separate shared libraries. Now:

- `sherpa-onnx` links dynamically against the **same `libonnxruntime.so`** that `ort`'s `load-dynamic` will `dlopen`.
- The dynamic linker deduplicates the `.so` — only one ORT global environment exists.
- No static archive baking in a second ORT instance.

**Cargo.toml change:**
```toml
sherpa-onnx = { version = "1.12.39", default-features = false, features = ["shared"] }
```

**Build implications:**
- `sherpa-onnx-sys` build.rs for the shared path emits `cargo:rustc-link-lib=dylib=sherpa-onnx-c-api` and `cargo:rustc-link-lib=dylib=onnxruntime` (see `emit_shared_link_directives`).
- The build.rs also calls `copy_unix_runtime_libs` which copies `libsherpa-onnx-c-api.so` and `libonnxruntime.so` next to the binary (same profile output dir), and sets `-rpath,$ORIGIN` so the binary finds them at runtime without `LD_LIBRARY_PATH`.
- The `ORT_DYLIB_PATH` environment variable (used by `ort`'s `load-dynamic` to locate the `.so`) must point to the same `libonnxruntime.so` file copied by sherpa-onnx. **At runtime, set `ORT_DYLIB_PATH` to `$ORIGIN/libonnxruntime.so`**, or the daemon binary's directory.
- AppImage packaging must include both `.so` files in the AppDir and set `ORT_DYLIB_PATH` in the AppImage run script (MEM007 already documents this requirement).

**Tradeoffs:**
- Shared `.so` means both components share one ORT version — they must be ABI-compatible. sherpa-onnx 1.12.39's bundled ORT is 1.22.x; `ort` 2.0.0-rc.10 wraps ORT 1.22.x. These are the same major.minor — ABI compatible.
- Binary is smaller (ORT not baked into the `.a`).
- Runtime requires the `.so` files alongside the binary or in `LD_LIBRARY_PATH` / rpath.
- Symbols visible via `nm` / `objdump` (acceptable for this use case).

**Risk:** LOW. The shared build path in sherpa-onnx-sys is explicitly supported and tested by the upstream project. The `copy_unix_runtime_libs` + `-rpath,$ORIGIN` mechanism makes self-contained deployment straightforward.

---

### Option B: Replace silero-vad-rust with whisper.cpp's built-in VAD

**Mechanism:** `whisper-rs` 0.16.0 exposes `WhisperVadContext` (built-in Silero VAD ported into whisper.cpp). Since whisper.cpp does not use ONNX Runtime for VAD, removing `silero-vad-rust` (and therefore the `ort` dependency) eliminates the conflict entirely. sherpa-onnx continues with its static ORT with no competing instance.

**Tradeoffs:**
- Eliminates `silero-vad-rust`, `ort` from the dependency tree entirely — significant simplification.
- `WhisperVadContext` requires the Whisper model to be loaded and uses its internal VAD model, which means VAD is only available when `--features stt` is enabled. PTT-only mode (no STT) would lose VAD gating entirely.
- The existing `VadSegmenter` in `src/vad/mod.rs` uses `silero_vad_rust::silero_vad::model::OnnxModel` directly; switching would require replacing the scoring call at `score_with_silero()` with whisper.cpp's VAD API — a non-trivial refactor.
- Locking VAD to the Whisper model binary is a semantic coupling the current architecture intentionally avoids (D-16 "CPU-only baseline", VAD independent of STT).
- **Assessment:** Viable as a fallback if Option A fails, but the architectural cost is high and it creates a hard dependency between VAD and the `stt` feature.

---

### Option C: Compile sherpa-onnx from source, linking against the system ORT

**Mechanism:** Set `SHERPA_ONNX_LIB_DIR` to point to a directory containing static or shared sherpa-onnx libraries compiled from source (not the prebuilt archive). If the custom build uses the same ORT `.so` that `ort` will `load-dynamic`, the conflict is resolved the same way as Option A.

**Tradeoffs:**
- Requires users / CI to build sherpa-onnx from source — eliminates the prebuilt download convenience.
- Much higher maintenance burden.
- **Assessment:** Not worth pursuing when Option A (shared prebuilt) achieves the same result.

---

## Recommended Implementation Plan

### T01: Switch sherpa-onnx to shared feature + verify build

**Files:** `Cargo.toml`

1. Change `sherpa-onnx = "1.12.39"` to `sherpa-onnx = { version = "1.12.39", default-features = false, features = ["shared"] }`.
2. Run `cargo build` (no `--features stt` needed). Build.rs downloads `sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2` and copies `.so` files into the target profile dir.
3. Verify `libsherpa-onnx-c-api.so` and `libonnxruntime.so` appear in `target/debug/`.
4. Set `ORT_DYLIB_PATH=$(pwd)/target/debug/libonnxruntime.so` and confirm the daemon starts without panic.

**Verify command:**
```bash
ORT_DYLIB_PATH=$(pwd)/target/debug/libonnxruntime.so cargo build 2>&1 | grep -E "error|warning.*onnx"
ldd target/debug/hd-linux-voice | grep onnx
```

### T02: Enable wake in config and test simultaneous VAD + wake

**Files:** `src/pipeline/coordinator.rs`, `tests/wake_word.rs`

1. Update `src/pipeline/coordinator.rs` to set `ORT_DYLIB_PATH` programmatically at startup if not already set, using the binary's directory as the search path:
   ```rust
   // Set ORT_DYLIB_PATH to locate libonnxruntime.so next to the binary,
   // unless the caller already set it (allows override for custom installs).
   if std::env::var_os("ORT_DYLIB_PATH").is_none() {
       if let Ok(exe) = std::env::current_exe() {
           if let Some(dir) = exe.parent() {
               std::env::set_var("ORT_DYLIB_PATH", dir.join("libonnxruntime.so"));
           }
       }
   }
   ```
   This must be set **before** the first `ort::init()` call (i.e., before `load_silero_vad_with_options`).

2. Extend `tests/wake_word.rs` with a dual-init test: initialise a `KeywordSpotter` (sherpa-onnx) and then call `load_silero_vad_with_options` (silero-vad-rust / ort) in sequence. Assert neither panics. Gate this test with `RUN_KWS_TESTS=1`.

3. Update `src/config.rs` — remove the "enable only one at a time" comment from `WakeConfig` since the constraint is lifted.

### T03: Update documentation

**Files:** `docs/troubleshooting.md`, `docs/configuration.md`

1. Remove the "ensure only one feature (STT or wake) is enabled at a time" guidance from `docs/troubleshooting.md` Models section.
2. Remove the ORT conflict note from `docs/configuration.md` wake section.
3. Add a note explaining that `libonnxruntime.so` and `libsherpa-onnx-c-api.so` are copied next to the binary at build time and are required for the daemon to run when wake is enabled.
4. Update `docs/troubleshooting.md` Build section to document `ORT_DYLIB_PATH` for custom installs.
5. Update the documentation tests in `tests/documentation.rs` if any assertions are tied to the old conflict text.

---

## Verification Architecture

| Check | Command | Expected |
|-------|---------|---------|
| Build succeeds with shared sherpa-onnx | `cargo build` | Zero errors |
| `.so` files present in output dir | `ls target/debug/*.so \| grep onnx` | `libonnxruntime.so`, `libsherpa-onnx-c-api.so` |
| Binary links against shared ORT | `ldd target/debug/hd-linux-voice \| grep onnx` | Shows `libsherpa-onnx-c-api.so` |
| VAD + wake co-init without panic | `RUN_KWS_TESTS=1 cargo test --test wake_word` | All pass |
| Doc tests still pass | `cargo test --test documentation` | All pass |
| ONNX conflict note removed from docs | `grep -i "only one" docs/troubleshooting.md` | No match |

---

## Implementation Landscape

### Files That Change

| File | Change |
|------|--------|
| `Cargo.toml` | `sherpa-onnx` dep switches to `default-features = false, features = ["shared"]` |
| `src/pipeline/coordinator.rs` | Add `ORT_DYLIB_PATH` auto-discovery block before Silero init (lines 233–252) |
| `docs/troubleshooting.md` | Remove ORT conflict warning; add ORT `.so` deployment note |
| `docs/configuration.md` | Remove ORT conflict note from `wake` section |
| `tests/wake_word.rs` | Add dual-init co-existence test |
| `tests/documentation.rs` | Update any assertions that grep for the old conflict text |

### Files That Do Not Change

| File | Reason |
|------|--------|
| `src/wake/mod.rs` | No change needed — already correct |
| `src/vad/mod.rs` | No change needed |
| `src/config.rs` | Minor comment cleanup only (not structural) |
| `src/stt/mod.rs` | Not involved in this conflict |

---

## Risk Assessment

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| ORT ABI version mismatch between sherpa-onnx shared prebuilt and ort 2.0.0-rc.10 | LOW — both use ORT 1.22.x | Verify with `nm libonnxruntime.so | grep OrtGetApiBase` |
| `ORT_DYLIB_PATH` auto-set interferes with user override | NONE — only sets when env var is absent | Already guarded with `if std::env::var_os("ORT_DYLIB_PATH").is_none()` |
| Shared `.so` not found at runtime in CI / fresh checkouts | MEDIUM — build.rs copies `.so` to profile dir but not source tree | Document in CONTRIBUTING.md; CI must `cargo build` before running tests |
| AppImage `.so` bundling regression | LOW — MEM007 already documents the requirement | Verify in S08 (packaging) when AppImage is built |
| `cargo test` vs `cargo build` divergence | LOW — build.rs copies `.so` on `cargo build` not `cargo test` alone | Tests that need `libonnxruntime.so` are env-gated (#[ignore]) already |

---

## Key Constraints for the Planner

1. **Set `ORT_DYLIB_PATH` before `load_silero_vad_with_options`** — the `ort` crate reads this env var at session-creation time, not at link time. Setting it after the first session is created has no effect.
2. **Do not use `std::env::set_var` from multiple threads simultaneously** — the call in coordinator must happen before any VAD/wake threads are spawned (it already does — coordinator sets up VAD before spawning the pipeline thread).
3. **`cargo test` alone does not trigger build.rs** — the `libonnxruntime.so` copy happens during `cargo build`. The dual-init test needs a prior `cargo build` or must be run as `cargo test --test wake_word` after a successful `cargo build`.
4. **Static → shared is a build-time switch, not a runtime config** — the `sherpa-onnx` feature flag change affects what the build downloads; it cannot be toggled per-user at runtime.
5. **MEM004 constraint applies** — `cargo test` requires user approval in auto-mode. Static verification (file existence, symbol grep, `ldd` output) is the available in-auto-mode verification path; compiled test confirmation deferred to CI / manual run.

---

## Sources

- `sherpa-onnx-sys-1.12.39/build.rs` — `emit_static_link_directives` links `onnxruntime` as a static lib; `emit_shared_link_directives` links `libonnxruntime.so` as dynamic. `copy_unix_runtime_libs` copies `.so` + sets `-rpath,$ORIGIN`. [VERIFIED: local registry cache]
- `sherpa-onnx-1.12.39/Cargo.toml` — `shared` feature maps to `sherpa-onnx-sys/shared`. [VERIFIED: local registry cache]
- `silero-vad-rust-6.2.1/Cargo.toml.orig` — `ort = { version = "2.0.0-rc.10", features = ["load-dynamic", "ndarray"] }`. [VERIFIED: local registry cache]
- `ort-2.0.0-rc.10/Cargo.toml` — `load-dynamic` feature enables `libloading`-based dlopen. [VERIFIED: local registry cache]
- `src/pipeline/coordinator.rs:233–252` — existing `catch_unwind` around Silero load; existing actionable error message. [VERIFIED: local code]
- `src/wake/mod.rs` — implementation complete, only `WakeConfig.enabled` gates usage. [VERIFIED: local code]
- MEM007 — AppImage must set `LD_LIBRARY_PATH` / bundle `.so` files for ORT to load. [VERIFIED: memory store]
- MEM012 — ORT conflict documented in troubleshooting.md and configuration.md. [VERIFIED: memory store]
