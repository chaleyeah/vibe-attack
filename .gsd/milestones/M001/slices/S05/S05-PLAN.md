# S05: UI + Distribution — egui config window, system tray, first Run wizard, AppImage, AUR/PKGBUILD

**Goal:** Create testable pure-logic UI state modules (`src/ui/first_run.rs`, `src/ui/config_app.rs`), packaging scaffolding (`packaging/PKGBUILD`, `packaging/appimage/`), and a feature-gated egui config binary — all proven by 15+ tests in `tests/ui_distribution.rs` and the existing daemon headless ldd test continuing to pass.
**Demo:** unit tests prove UI + Distribution — egui config window, system tray, first-run wizard, AppImage, AUR/PKGBUILD works

## Must-Haves

- `cargo test --test ui_distribution` passes with 15+ tests covering FirstRunState logic, ConfigApp logic, packaging file structure, and feature gate separation. `cargo test --test daemon_headless` continues to pass (daemon binary does not link GUI libraries). `cargo build` (default features) does not pull in egui/eframe.

## Proof Level

- This slice proves: - This slice proves: contract (pure-logic UI state + packaging structure)
- Real runtime required: no (tests exercise pure structs; egui render is compile-check only)
- Human/UAT required: no

## Integration Closure

- Upstream surfaces consumed: `src/config.rs` (Config, MacroConfig structs), `src/control/client.rs` (send_command for IPC), `src/pack/mod.rs` (get_profiles_dir), `src/pack/manager.rs` (ProfileManager), `src/error.rs` (DaemonError)
- New wiring introduced in this slice: `src/ui/` module added to `src/lib.rs`, `src/bin/hd-linux-voice-config.rs` binary behind `gui` feature, `packaging/` directory with PKGBUILD + AppImage scaffolding
- What remains before the milestone is truly usable end-to-end: S06 (docs), S07 (wake word ORT conflict resolution)

## Verification

- Not provided.

## Tasks

- [x] **T01: Create src/ui/ module with FirstRunState and ConfigApp pure-logic structs plus tests** `est:45m`
  Create the `src/ui/` module tree with two pure-logic state files: `first_run.rs` (FirstRunState struct with from_checks constructor, is_setup_complete, steps_remaining, first_incomplete_step methods; SetupStep enum with CreateConfig/InstallModel/SetupUinput/ConfigurePtt variants) and `config_app.rs` (ConfigApp struct with profiles vec, active_profile, log_lines, mic_level fields; profile_count, add_log_line, new methods; MAX_LOG_LINES cap). Add `pub mod ui;` to `src/lib.rs`. Then write `tests/ui_distribution.rs` with 11 pure-logic tests:

1. `first_run_complete_when_all_checks_pass` — from_checks(true,true,true,true).is_setup_complete() == true
2. `first_run_incomplete_when_config_missing` — steps_remaining contains CreateConfig
3. `first_run_incomplete_when_model_missing` — steps_remaining contains InstallModel
4. `first_run_incomplete_when_uinput_inaccessible` — steps_remaining contains SetupUinput
5. `first_run_incomplete_when_ptt_missing` — steps_remaining contains ConfigurePtt
6. `first_run_step_ordering_config_before_model` — wizard returns CreateConfig before InstallModel
7. `first_run_all_steps_when_fresh_install` — from_checks(false,false,false,false) returns 4 steps
8. `first_incomplete_step_returns_none_when_done` — first_incomplete_step() is None when complete
9. `config_app_profile_count_reflects_profiles` — profile_count() matches profiles.len()
10. `config_app_add_log_line_grows_log` — add_log_line increments log_lines.len()
11. `config_app_log_capped_at_max` — add_log_line doesn't grow past MAX_LOG_LINES

IMPORTANT CONSTRAINTS:
- `src/ui/first_run.rs` and `src/ui/config_app.rs` must NOT import egui/eframe — they are pure Rust structs with no GUI dependencies
- `src/ui/mod.rs` re-exports both modules publicly
- `src/lib.rs` adds `pub mod ui;` at the end of the existing module list
- Tests use `use hd_linux_voice::ui::first_run::{FirstRunState, SetupStep};` and `use hd_linux_voice::ui::config_app::ConfigApp;`
- ConfigApp::add_log_line must cap at MAX_LOG_LINES (100) by dropping oldest lines
- SetupStep ordering in steps_remaining: CreateConfig, InstallModel, SetupUinput, ConfigurePtt (this order matches the first-run wizard flow)
  - Files: `src/ui/mod.rs`, `src/ui/first_run.rs`, `src/ui/config_app.rs`, `src/lib.rs`, `tests/ui_distribution.rs`
  - Verify: cargo test --test ui_distribution 2>&1 | grep -E 'test result|running' && test $(grep -c '#\[test\]' tests/ui_distribution.rs) -ge 11

- [x] **T02: Create packaging scaffolding (PKGBUILD, AppImage, .desktop) and add structural tests** `est:30m`
  Create the `packaging/` directory tree with three distribution artifacts and add 4 structural tests to `tests/ui_distribution.rs`.

**Files to create:**

1. `packaging/PKGBUILD` — AUR-style PKGBUILD template with required fields:
   - pkgname=hd-linux-voice
   - pkgver=0.1.0
   - pkgrel=1
   - pkgdesc='Voice-macro daemon for Helldivers 2 on Linux'
   - arch=('x86_64')
   - url='https://github.com/yourusername/hd-linux-voice'
   - license=('AGPL-3.0-only')
   - depends=('alsa-lib')
   - makedepends=('rust' 'cargo')
   - build() function calling cargo build --release
   - package() function installing binary + .desktop + docs

2. `packaging/appimage/hd-linux-voice.desktop` — Freedesktop .desktop file:
   - [Desktop Entry] section
   - Name=HD Linux Voice
   - Exec=hd-linux-voice
   - Type=Application
   - Icon=hd-linux-voice
   - Comment=Voice macro daemon for Helldivers 2
   - Categories=Game;Utility;
   - Terminal=false

3. `packaging/appimage/build.sh` — AppImage build script (executable):
   - Shebang #!/usr/bin/env bash
   - set -euo pipefail
   - cargo build --release
   - Creates AppDir structure
   - Copies binary, .desktop, icon into AppDir
   - Sets LD_LIBRARY_PATH for ORT .so (per research ORT AppImage constraint)
   - Calls linuxdeploy + appimagetool (commented-out final steps since tools may not be installed)

**Tests to ADD to existing tests/ui_distribution.rs (append after T01's tests):**

12. `pkgbuild_file_exists_and_has_required_fields` — reads `packaging/PKGBUILD`, asserts pkgname=, pkgver=, url=, license= lines present
13. `desktop_file_exists_and_has_required_keys` — reads `packaging/appimage/hd-linux-voice.desktop`, asserts Name=, Exec=, Type= lines present
14. `appimage_build_script_exists` — asserts `packaging/appimage/build.sh` exists and is non-empty
15. `appimage_build_script_has_shebang` — reads first line of build.sh, asserts starts with #!/

IMPORTANT CONSTRAINTS:
- build.sh must be created with execute permission (chmod +x after writing, or use std::os::unix::fs::PermissionsExt in test if checking permissions)
- PKGBUILD must use AGPL-3.0-only license (matching Cargo.toml)
- .desktop file must NOT include a full path in Exec= (just the binary name)
- Tests read files relative to env!("CARGO_MANIFEST_DIR") to find project root
- The build.sh script must include a comment about LD_LIBRARY_PATH for ORT .so in AppImage FUSE mount (per research constraint)
  - Files: `packaging/PKGBUILD`, `packaging/appimage/hd-linux-voice.desktop`, `packaging/appimage/build.sh`, `tests/ui_distribution.rs`
  - Verify: test -f packaging/PKGBUILD && test -f packaging/appimage/hd-linux-voice.desktop && test -f packaging/appimage/build.sh && grep -q 'pkgname=' packaging/PKGBUILD && grep -q 'Name=' packaging/appimage/hd-linux-voice.desktop && cargo test --test ui_distribution 2>&1 | grep -E 'test result|running'

- [x] **T03: Add feature-gated egui binary and verify daemon stays headless** `est:45m`
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
  - Files: `Cargo.toml`, `src/bin/hd-linux-voice-config.rs`, `tests/ui_distribution.rs`
  - Verify: grep -q 'gui = ' Cargo.toml && grep -q 'eframe' Cargo.toml && grep -q 'required-features.*gui' Cargo.toml && test -f src/bin/hd-linux-voice-config.rs && grep -q 'default_features_exclude_gui' tests/ui_distribution.rs && cargo build 2>&1 | tail -5

## Files Likely Touched

- src/ui/mod.rs
- src/ui/first_run.rs
- src/ui/config_app.rs
- src/lib.rs
- tests/ui_distribution.rs
- packaging/PKGBUILD
- packaging/appimage/hd-linux-voice.desktop
- packaging/appimage/build.sh
- Cargo.toml
- src/bin/hd-linux-voice-config.rs
