# S02: Environment probe

**Goal:** Wire FirstRunState::from_checks() to real environment detection via a new src/ui/probe.rs module. Four checks: config file at XDG path, whisper model file exists and non-empty, /dev/uinput openable O_RDWR, PTT key set in config. Each check logs on failure. probe::run() is the single production call site.
**Demo:** Unit tests (no display server) pass: probe returns correct booleans for each check under hermetic XDG temp dirs; /dev/uinput open failure is correctly classified as inaccessible

## Must-Haves

- probe::run() returns correct FirstRunState under real and hermetic-tempdir conditions; each false result has a tracing::warn log with check name and reason; all four checks have passing hermetic unit tests; vibe-attack-config.rs uses probe::run() instead of from_checks(false,false,false,false)

## Proof Level

- This slice proves: cargo test --lib on probe module with XDG tempdir isolation; manual inspection that vibe-attack-config.rs stub is replaced

## Integration Closure

probe::run() is the only FirstRunState constructor in production code; wizard panels (S03) call it after each action to refresh state

## Verification

- tracing::warn per failed check with check name and reason string; tracing::info on probe::run() entry with XDG config path

## Tasks

- [x] **T01: Create src/ui/probe.rs with check functions and probe::run()** `est:45m`
  Create src/ui/probe.rs. Import xdg, std::fs, std::os::unix::fs::OpenOptionsExt, tracing. Define four private check functions: check_config(xdg) -> bool, check_model(xdg) -> bool, check_uinput() -> bool, check_ptt(config_path) -> bool. Each returns false and emits tracing::warn with reason on failure. Define pub fn run() -> FirstRunState that calls all four and constructs FirstRunState::from_checks(...). Use xdg::BaseDirectories::with_prefix('vibe-attack') for XDG paths. Model path: data_home/vibe-attack/models/whisper/ggml-tiny.en.bin. Config path: config_home/vibe-attack/config.yaml. uinput check: OpenOptions::new().read(true).write(true).custom_flags(libc::O_NONBLOCK).open('/dev/uinput') — if open succeeds it's accessible. PTT check: read config file and grep for 'key: KEY_' pattern.
  - Files: `src/ui/probe.rs`, `src/ui/mod.rs`
  - Verify: cargo check --lib passes with probe module added to src/ui/mod.rs

- [x] **T02: Add libc dependency and write hermetic unit tests** `est:35m`
  Add libc to Cargo.toml dependencies (needed for O_NONBLOCK in probe). Write #[cfg(test)] tests in src/ui/probe.rs covering: check_config returns false when config missing, true when present; check_model returns false when model missing or empty, true when non-empty file present; check_ptt returns false when config has no PTT key, true when 'key: KEY_LEFTCTRL' is present. All tests use tempdir and set XDG_CONFIG_HOME / XDG_DATA_HOME for hermetic isolation. Use unsafe { std::env::set_var } with the established pattern from existing tests.
  - Files: `src/ui/probe.rs`, `Cargo.toml`
  - Verify: cargo test --lib ui::probe exits 0 with all tests passing

- [x] **T03: Wire probe::run() into vibe-attack-config.rs** `est:15m`
  Replace FirstRunState::from_checks(false, false, false, false) in src/bin/vibe-attack-config.rs with a call to vibe_attack::ui::probe::run(). Call probe::run() in VibeAttackConfigApp::new(). No other changes to the binary — wizard UI panels come in S03.
  - Files: `src/bin/vibe-attack-config.rs`
  - Verify: cargo build --bin vibe-attack-config --features gui exits 0; no from_checks(false,false,false,false) remains in production code

## Files Likely Touched

- src/ui/probe.rs
- src/ui/mod.rs
- Cargo.toml
- src/bin/vibe-attack-config.rs
