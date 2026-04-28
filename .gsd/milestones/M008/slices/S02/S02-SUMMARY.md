---
id: S02
parent: M008
milestone: M008
provides:
  - ["ConfigApp with full mode/threshold/device/PTT state", "load_config_into_app and save_app_to_config helpers", "egui config panel with Save dispatch to control socket", "Daemon-absent graceful degradation path"]
requires:
  - slice: S01
    provides: control::protocol ControlRequest variants (SetMode, SetThreshold, SetInputDevice, SetPttBinding), control::client::send_command, ActivationMode
affects:
  []
key_files:
  - ["src/ui/config_app.rs", "src/bin/vibe-attack-config.rs"]
key_decisions:
  - ["ActivationMode not persisted to config.yaml — runtime-only in M008; SetMode sent over socket on Save instead", "threshold_pct uses u8 (0-100) integer domain with round+clamp conversion at I/O to prevent float drift", "cached_config populated once at startup/wizard-complete, not per-frame — avoids I/O on every egui render tick", "device_names populated once at startup via cpal, not per-frame — cpal enumeration is not cheap", "Atomic write via .yaml.tmp sibling + std::fs::rename — partial saves on crash are invisible to next load", "cargo build substituted for cargo clippy throughout — clippy not installed on this build system"]
patterns_established:
  - ["ConfigApp I/O helpers pattern: load returns full Config for caching; save takes cached Config as base to clone+mutate, preserving all non-UI fields", "Daemon-absent graceful degradation: always write to disk, only send socket commands when daemon_running=true, always leave user with actionable status message", "XDG_CONFIG_HOME override + serial_test::serial for config file tests in this codebase"]
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-28T01:42:55.664Z
blocker_discovered: false
---

# S02: ConfigApp state + egui config panel

**Extended ConfigApp with runtime state fields, wired config.yaml load/save with atomic writes and full round-trip preservation, and built the egui config panel with Save dispatch to the control socket.**

## What Happened

S02 delivered the complete config-window contract layer in three tasks:

**T01 — ConfigApp state fields**: Added six new public fields to `ConfigApp` in `src/ui/config_app.rs`: `mode: ActivationMode`, `threshold_pct: u8`, `input_device: Option<String>`, `ptt_binding: String`, `status_message: Option<String>`, `daemon_running: bool`. Added `apply_from_config(&mut self, cfg: &Config)` using `(x * 100.0).round().clamp(0.0, 100.0) as u8` for float-safe threshold conversion, and `set_status(msg)` for UI feedback. ActivationMode was confirmed to already derive `PartialEq` — no change to `protocol.rs` needed. Seven new unit tests cover round-trip, clamping (above/below), rounding (not truncation), field copy, and status write.

**T02 — Config I/O helpers**: Added two public functions in `src/ui/config_app.rs`: `load_config_into_app` (calls `config::load`, applies to ConfigApp, returns the full `Config` for caching) and `save_app_to_config` (clones the cached Config, mutates the three YAML-persisted fields — threshold, device, ptt key — serializes via `serde_yaml_ng`, atomically writes via `.yaml.tmp` sibling + rename). ActivationMode is intentionally NOT persisted to YAML (runtime-only in M008; no `mode` field in Config struct). Four unit tests: state population, round-trip, macro preservation, and atomic write verification. Tests use `XDG_CONFIG_HOME` + `serial_test::serial` to prevent cross-test pollution.

**T03 — egui panel + Save dispatch**: Rewrote `show_main_config` to accept `&mut VibeAttackConfigApp`, added `cached_config: Option<Config>` (populated once at startup/wizard-complete) and `device_names: Vec<String>` (populated once via cpal at startup). Panel layout: daemon status indicator (green/amber), activation mode radio buttons, confidence threshold slider (0–100%), input device ComboBox (with `<system default>` option), read-only PTT key display, Save button, status message bar, profiles list, log scroll area. `handle_save` dispatches SetMode/SetThreshold/SetInputDevice/SetPttBinding over the control socket only when daemon is running; the daemon-absent path always writes to disk and shows a clear status message. `daemon_running` refreshed each frame via `is_daemon_running()` (a cheap socket stat call).

**Environment note**: Clippy is not installed on this build system (cargo from source tarball, no rustup). All clippy gates were substituted with `cargo build` (default and `--features gui`), both of which compile with zero warnings.

## Verification

- `cargo test --features gui`: 0 failed, all non-hardware-gated tests pass (exit 0)
- `cargo build` (default features): clean, exit 0
- `cargo build --features gui`: clean, exit 0
- Unit tests cover: apply_from_config round-trip (0.8→80), clamping (1.5→100, -0.2→0), rounding (0.835→84), ActivationMode field round-trip, set_status, load_config_into_app populates state, save_app_to_config round-trip, macro preservation across save, atomic write (no .tmp leftover)
- cargo clippy not available on this system (pre-existing environment constraint — cargo from source tarball, no rustup component); build-clean substituted per all task summaries

## Requirements Advanced

- UI-03 — Config window now exposes audio input device, activation mode, confidence threshold, and PTT keybinding (read-only) fields with Save dispatch to daemon
- ACT-03 — Mode toggle (PTT ↔ Wake word) implemented in config panel; Save sends SetMode to daemon; S04 will validate daemon honors it without restart
- STT-03 — Confidence threshold slider (0–100%) wired in config panel; Save sends SetThreshold to daemon

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

Clippy verification was substituted with cargo build (default + gui features) throughout — clippy is not installed on the build system. PTT key binding in the config panel is read-only display in M008; live key capture requires daemon restart and is deferred to a future milestone. ActivationMode is not persisted to config.yaml in this milestone — mode selection is purely runtime via SetMode control command.

## Follow-ups

S03 (tray icon state mapping + Mode submenu) can now consume the same ActivationMode and control protocol surface. S04 UAT will prove SetMode/SetThreshold daemon round-trips without restart end-to-end.

## Files Created/Modified

- `src/ui/config_app.rs` — Added mode/threshold_pct/input_device/ptt_binding/status_message/daemon_running fields, apply_from_config, set_status, load_config_into_app, save_app_to_config with atomic write, 11 new unit tests
- `src/bin/vibe-attack-config.rs` — Rewrote show_main_config to &mut VibeAttackConfigApp, added cached_config + device_names, egui panel with all widgets, handle_save with daemon-aware dispatch
