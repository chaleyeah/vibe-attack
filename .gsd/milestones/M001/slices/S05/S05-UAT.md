# S05: UI + Distribution — egui config window, system tray, first-run wizard, AppImage, AUR/PKGBUILD — UAT

**Milestone:** M001
**Written:** 2026-04-25T19:49:47.671Z

# S05 UAT — UI + Distribution

## Preconditions
- Rust toolchain installed (`cargo --version` works)
- Working directory is project root (`/home/chadmin/Github/hd-linux-voice`)
- No `gui` feature in `default` features of Cargo.toml

## Test Cases

### TC-01: ui_distribution test suite passes (≥15 tests)
1. Run `cargo test --test ui_distribution`
2. Expected: `test result: ok. 16 passed; 0 failed` (or more if tests were added)
3. Expected: All 16 tests listed with `ok` status

### TC-02: FirstRunState logic — complete setup
1. Inspect `tests/ui_distribution.rs` test `first_run_complete_when_all_checks_pass`
2. Logic: `FirstRunState::from_checks(true,true,true,true).is_setup_complete()` must return `true`
3. Expected: test passes without panic

### TC-03: FirstRunState step ordering
1. Run `cargo test first_run_step_ordering`
2. Expected: wizard returns CreateConfig before InstallModel (canonical wizard order)

### TC-04: ConfigApp log cap
1. Run `cargo test config_app_log_capped_at_max`
2. Expected: after 110 add_log_line calls, log_lines.len() == MAX_LOG_LINES (100); oldest entries dropped

### TC-05: Daemon default build excludes GUI libraries
1. Run `cargo build` (no `--features gui`)
2. Expected: build succeeds
3. Run `ldd target/debug/hd-linux-voice`
4. Expected: output does NOT contain libwayland, libX11, libxcb, libgtk, libgdk

### TC-06: Config binary builds with gui feature
1. Run `cargo build --features gui`
2. Expected: build succeeds, `target/debug/hd-linux-voice-config` binary produced

### TC-07: PKGBUILD structural correctness
1. Read `packaging/PKGBUILD`
2. Expected: contains `pkgname=hd-linux-voice`, `pkgver=`, `license=('AGPL-3.0-only')`, `url=`, `makedepends=('rust' 'cargo')`, `depends=('alsa-lib')`
3. Expected: `build()` function calls `cargo build --release`
4. Expected: `package()` function installs binary

### TC-08: .desktop file correctness
1. Read `packaging/appimage/hd-linux-voice.desktop`
2. Expected: `[Desktop Entry]` header present
3. Expected: `Exec=hd-linux-voice` (no full path prefix)
4. Expected: `Type=Application`, `Name=HD Linux Voice`, `Categories=Game;Utility;`, `Terminal=false`

### TC-09: AppImage build script
1. Read `packaging/appimage/build.sh`
2. Expected: first line is `#!/usr/bin/env bash`
3. Expected: `set -euo pipefail` present
4. Expected: LD_LIBRARY_PATH comment for ORT FUSE mount present
5. Run `chmod +x packaging/appimage/build.sh` (fix execute bit skipped in auto-mode)
6. Expected: file is now executable

### TC-10: gui feature gate structural test
1. Run `cargo test daemon_default_features_exclude_gui`
2. Expected: test passes — reads Cargo.toml, confirms `default = []` does not contain "gui"

### TC-11: daemon_headless test still passes
1. Run `cargo test --test daemon_headless`
2. Expected: all daemon headless tests pass (regression check — S05 must not break prior passing tests)

## Edge Cases
- Running `cargo build` without `--features gui` must NOT attempt to compile `src/bin/hd-linux-voice-config.rs` (required-features gate)
- Adding 200 log lines to ConfigApp must result in exactly 100 entries, not 101 or 99
- SetupStep ordering in steps_remaining() must always be CreateConfig → InstallModel → SetupUinput → ConfigurePtt regardless of which subset is incomplete
