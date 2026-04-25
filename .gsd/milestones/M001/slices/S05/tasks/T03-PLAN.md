---
estimated_steps: 23
estimated_files: 3
skills_used: []
---

# T03: Add feature-gated egui binary and verify daemon stays headless

Add egui/eframe as an optional dependency behind a `gui` feature flag, create a minimal `src/bin/hd-linux-voice-config.rs` binary that uses FirstRunState and ConfigApp, and add a feature-gate separation test. This task ensures the daemon binary (default features) never links GUI libraries while the config binary is available via `cargo build --features gui`.

**Step 1: Update Cargo.toml**
- Add to [features]: `gui = ["dep:eframe"]`
- Add to [dependencies]: `eframe = { version = "0.31", optional = true, default-features = false, features = ["default_fonts", "glow"] }`
- Add a new [[bin]] section: `[[bin]]\nname = "hd-linux-voice-config"\npath = "src/bin/hd-linux-voice-config.rs"\nrequired-features = ["gui"]`
- The `default = []` features line must remain unchanged — gui must NOT be in default features

**Step 2: Create src/bin/hd-linux-voice-config.rs**
A minimal egui app that:
- Imports `hd_linux_voice::ui::first_run::FirstRunState` and `hd_linux_voice::ui::config_app::ConfigApp`
- Creates an eframe::App implementation that shows FirstRunState steps if setup is incomplete, otherwise shows ConfigApp state
- The app struct holds both FirstRunState and ConfigApp
- The update() method uses egui to render: wizard step list OR profile list + log view
- Keep it minimal — this is a proof-of-compilation, not a polished UI. 50-80 lines total.
- The binary's main() calls eframe::run_native() with the app

**Step 3: Add test to tests/ui_distribution.rs**
16. `daemon_default_features_exclude_gui` — reads `Cargo.toml`, parses the `[features]` section, asserts `default = []` does not contain "gui". This is a structural test ensuring the feature gate is correctly configured.

**IMPORTANT CONSTRAINTS:**
- The eframe version should be 0.31 (latest stable as of 2026). If 0.31 doesn't exist, use 0.30 or 0.29.
- Use `glow` backend (not `wgpu`) — lighter weight, better compatibility with older GPU drivers on Linux gaming setups.
- The [[bin]] entry MUST have `required-features = ["gui"]` so `cargo build` (no features) does NOT try to compile the config binary.
- After this task, `cargo build` (default) must still succeed and the daemon binary must NOT link libwayland/libX11/libxcb/libgtk/libgdk.
- The config binary file `src/bin/hd-linux-voice-config.rs` must use `#[cfg(feature = "gui")]` only if needed — the `required-features` in Cargo.toml already gates compilation.
- Do NOT add `pub mod ui` conditionally — the ui module must be unconditionally public in lib.rs (T01 already did this) so tests can use it without the gui feature.

## Inputs

- `Cargo.toml`
- `src/ui/mod.rs`
- `src/ui/first_run.rs`
- `src/ui/config_app.rs`
- `tests/ui_distribution.rs`

## Expected Output

- `Cargo.toml`
- `src/bin/hd-linux-voice-config.rs`
- `tests/ui_distribution.rs`

## Verification

grep -q 'gui = ' Cargo.toml && grep -q 'eframe' Cargo.toml && grep -q 'required-features.*gui' Cargo.toml && test -f src/bin/hd-linux-voice-config.rs && grep -q 'default_features_exclude_gui' tests/ui_distribution.rs && cargo build 2>&1 | tail -5
