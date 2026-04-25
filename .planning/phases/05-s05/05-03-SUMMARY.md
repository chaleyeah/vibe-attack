---
phase: "05"
plan: "03"
---

# T03: Added gui feature flag, eframe optional dep, hd-linux-voice-config binary, and structural test 16 confirming default features exclude gui

**Added gui feature flag, eframe optional dep, hd-linux-voice-config binary, and structural test 16 confirming default features exclude gui**

## What Happened

Three changes were made to complete the feature-gated egui binary:

**Cargo.toml** — added `gui = ["dep:eframe"]` to `[features]` (default remains `[]`), added `eframe = { version = "0.31", optional = true, default-features = false, features = ["default_fonts", "glow"] }` to `[dependencies]`, and added a new `[[bin]]` section for `hd-linux-voice-config` with `required-features = ["gui"]`. This ensures `cargo build` (no flags) never touches eframe and the daemon binary remains GUI-library-free.

**src/bin/hd-linux-voice-config.rs** — a ~60-line minimal eframe 0.31 app implementing `eframe::App`. It holds both `FirstRunState` and `ConfigApp` from the library crate's unconditionally-public `ui` module. The `update()` method renders a wizard step list when setup is incomplete, or a profile count + scrollable log view when complete. The `main()` uses `eframe::run_native()` with `glow` backend and a 600×400 viewport.

**tests/ui_distribution.rs** — added test 16 `daemon_default_features_exclude_gui` which reads `Cargo.toml` at runtime via `CARGO_MANIFEST_DIR`, locates the `[features]` section, finds the `default` line, and asserts it does not contain the string "gui". The slice now has 16 tests total (11 pure-logic + 4 packaging structural + 1 feature gate).

The `ui` module was confirmed unconditionally public in `src/lib.rs` (no `#[cfg(feature="gui")]` wrapper), so all 16 tests in `tests/ui_distribution.rs` remain runnable without the gui feature enabled.

## Verification

Structural grep checks: (1) `gui = ` present in Cargo.toml ✅, (2) `eframe` present in Cargo.toml ✅, (3) `required-features.*gui` present in Cargo.toml ✅, (4) `src/bin/hd-linux-voice-config.rs` exists ✅, (5) `default_features_exclude_gui` present in tests/ui_distribution.rs ✅, (6) `default = []` (no gui) confirmed ✅, (7) test count = 16 ✅. cargo build was blocked by auto-mode sandbox (MEM004 convention — requires approval). Static verification confirmed correct API usage (eframe 0.31 run_native closure, eframe::App trait, egui::Context/Frame signatures).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `grep -q 'gui = ' Cargo.toml && echo PASS` | 0 | ✅ pass | 20ms |
| 2 | `grep -q 'eframe' Cargo.toml && echo PASS` | 0 | ✅ pass | 18ms |
| 3 | `grep -q 'required-features.*gui' Cargo.toml && echo PASS` | 0 | ✅ pass | 19ms |
| 4 | `test -f src/bin/hd-linux-voice-config.rs && echo PASS` | 0 | ✅ pass | 15ms |
| 5 | `grep -q 'default_features_exclude_gui' tests/ui_distribution.rs && echo PASS` | 0 | ✅ pass | 16ms |
| 6 | `grep '^default' Cargo.toml  # must be 'default = []'` | 0 | ✅ pass — output: default = [] | 14ms |
| 7 | `grep -c '^#\[test\]' tests/ui_distribution.rs  # must be 16` | 0 | ✅ pass — output: 16 | 17ms |
| 8 | `cargo build 2>&1 | tail -5` | -1 | ⚠️ blocked — auto-mode sandbox requires approval for cargo build (MEM004) | 0ms |

## Deviations

none — all three steps followed the task plan exactly; eframe 0.31 exists and was used as specified

## Known Issues

cargo build (default features) and cargo build --features gui were not run due to auto-mode sandbox blocking shell commands requiring approval. Static verification of API signatures and Cargo.toml structure was performed instead. A CI run or manual `cargo build` / `cargo build --features gui` should be the final confirmation gate.

## Files Created/Modified

- `Cargo.toml`
- `src/bin/hd-linux-voice-config.rs`
- `tests/ui_distribution.rs`
