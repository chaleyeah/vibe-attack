---
id: T03
parent: S02
milestone: M008
key_files:
  - src/bin/vibe-attack-config.rs
key_decisions:
  - cargo clippy is not installed on this Ubuntu 25.10 system (distro package ships only cargo/rustc, not the clippy component); cargo check --all-targets used as the lint substitute — catches all type/import errors at the type-check level
  - ComboBox device list clones app.device_names to avoid borrowing app mutably while the closure also borrows it mutably for input_device write
  - handle_save dispatches commands with label derived from Debug formatting of the ControlRequest variant to produce actionable error messages without duplicating variant names
duration: 
verification_result: passed
completed_at: 2026-04-28T01:40:29.148Z
blocker_discovered: false
---

# T03: Wired egui main config panel with mode/threshold/device/PTT widgets and Save dispatcher that persists to disk and sends control commands to the running daemon

**Wired egui main config panel with mode/threshold/device/PTT widgets and Save dispatcher that persists to disk and sends control commands to the running daemon**

## What Happened

Rewrote `src/bin/vibe-attack-config.rs` with the full T03 scope:

**New fields on `VibeAttackConfigApp`:** `cached_config: Option<vibe_attack::config::Config>` and `device_names: Vec<String>`, both populated once — devices at struct construction via `cpal::default_host().input_devices()`, config at startup (if `setup_complete`) or at wizard-complete transition via `load_config_into_app`. CPAL enumeration failure degrades to an empty `Vec`; config load failure sets `status_message` and leaves `cached_config = None`.

**`show_main_config` signature changed** from `(ui, &ConfigApp)` to `(ui, &mut VibeAttackConfigApp)`. The caller at the `else` branch was updated from `show_main_config(ui, &self.config)` to `show_main_config(ui, self)`.

**Panel layout (top to bottom):** daemon status colored row (green/amber), mic level progress bar, activation-mode radio buttons (Ptt/Wake), threshold `Slider` (0–100 u8), input-device `ComboBox` with leading `<system default>` → `None` option, PTT binding read-only label, Save button, status message label, profiles list, log scroll area.

**`daemon_running` refresh:** called each frame inside `ui()` after the log drain via `vibe_attack::control::client::is_daemon_running()` (cheap socket stat).

**`handle_save` free function:** (1) guard on `cached_config.is_none()` → status "No config loaded — cannot save."; (2) call `save_app_to_config` → on error set status "Save failed: …" and return; (3) on success replace `cached_config`; (4) if daemon not running set disk-only status and return; (5) otherwise dispatch four `ControlRequest` variants in order (SetMode, SetThreshold, SetInputDevice, SetPttBinding), collect errors into `Vec<String>`, join with "; ", set status "Saved and applied." on full success.

**Clippy unavailable:** `cargo clippy` is not installed on this Ubuntu system (only `cargo check` is available via the distro package). Substituted `cargo check --all-targets` for both feature sets as the lint verification step; this catches all type errors and unused-import warnings at the type-check level.

## Verification

cargo build --features gui --bin vibe-attack-config: exit 0 (4.49s build). cargo test --features gui: 58 tests pass, 0 fail (8.49s). cargo check --all-targets --features gui: exit 0. cargo check --all-targets (default features): exit 0. All must-haves confirmed: show_main_config takes &mut VibeAttackConfigApp; cached_config/device_names populated once not per-frame; daemon_running refreshed each frame; Save persists to disk then sends commands only when daemon is running; daemon-absent path sets status message and returns without panic.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features gui --bin vibe-attack-config` | 0 | ✅ pass | 4490ms |
| 2 | `cargo test --features gui` | 0 | ✅ pass — 58 passed, 0 failed | 8490ms |
| 3 | `cargo check --all-targets --features gui` | 0 | ✅ pass (clippy unavailable on this system; cargo check used) | 590ms |
| 4 | `cargo check --all-targets` | 0 | ✅ pass (default features) | 110ms |

## Deviations

cargo clippy unavailable — substituted cargo check --all-targets for both feature sets. The task plan's verification command included cargo clippy; that command exits 101 with \"no such command\" on this system. cargo check provides equivalent type-error coverage. All other must-haves met exactly as specified.

## Known Issues

none

## Files Created/Modified

- `src/bin/vibe-attack-config.rs`
