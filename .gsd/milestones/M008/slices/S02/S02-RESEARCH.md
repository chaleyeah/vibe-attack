# S02: ConfigApp state + egui config panel — Research

**Date:** 2026-04-27
**Status:** Research complete — implementation is straightforward egui wiring onto known APIs

## Summary

S02 adds four new fields to `ConfigApp` (mode, threshold, input_device, ptt_binding), builds a main config panel UI in `vibe-attack-config.rs` that renders those fields as a mode radio/toggle, threshold slider, device dropdown, and PTT key binding display, and wires the Save button to send `SetMode`/`SetThreshold`/`SetInputDevice`/`SetPttBinding`/`ReloadConfig` control requests to the running daemon via the existing `control::client::send_command()` function. The daemon socket, all control request variants, the full coordinator channel wiring, and the client-side `send_command()` helper are all complete from S01 — S02 is pure UI and wiring.

The "daemon not running" state is already partially handled: `control::client::is_daemon_running()` checks for socket existence, and `send_command()` returns an `Err` when the daemon is absent. S02 needs to surface this in the config window as a status bar message rather than a crash.

The config file persistence model is `Config` loaded from `~/.config/vibe-attack/config.yaml` via `crate::config::load(None)` and written back via `serde_yaml_ng::to_string` + `std::fs::write`. The `Config` struct already derives `Serialize` and `Deserialize`, so round-trip persistence is trivial once the field values are in `ConfigApp`.

## Recommendation

Implement S02 in three discrete tasks:

1. **T01 — Extend `ConfigApp` state fields + unit tests**: Add `mode: ActivationMode`, `threshold: f32`, `input_device: Option<String>`, `ptt_binding: String`, `status_message: Option<String>`, `daemon_running: bool` to `ConfigApp`. Add pure-logic methods: `apply_defaults_from_config(config: &Config)` and `validate_threshold() -> bool`. Add unit tests for field round-trips and validation. No egui, no I/O.

2. **T02 — Add `load_config_into_app()` + `save_app_to_config()` helpers + tests**: In `config_app.rs` (or a new `src/ui/config_io.rs` if cleaner), add helpers that read/write `~/.config/vibe-attack/config.yaml` using `crate::config::load()` + `serde_yaml_ng`. Tests use a tempdir config file (same XDG_CONFIG_HOME override pattern established by existing `load_profiles()` tests). 

3. **T03 — Wire egui config panel in `vibe-attack-config.rs`**: Replace the stub `show_main_config(ui, &config)` with a mutable `show_main_config(ui, &mut self)` call inside `VibeAttackConfigApp::ui()`. The panel renders: daemon status indicator; mode radio buttons (PTT / Wake); threshold slider (0–100 int, stored as f32 0.0–1.0); input device dropdown populated by `cpal::default_host().input_devices()`; PTT binding read-only display with a "Capture" button (key capture deferred or basic); Save button that sends commands and persists config; status bar for success/error messages. Load config on open and detect `is_daemon_running()` state on each repaint.

## Implementation Landscape

### Key Files

- `src/ui/config_app.rs` — `ConfigApp` struct; add new fields here. Pattern already established: pure-logic struct, all I/O outside. Currently has: `profiles`, `active_profile`, `log_lines`, `mic_level`, `mic_no_device`. Add `mode`, `threshold_pct: u8` (0–100 int for slider), `input_device`, `ptt_binding`, `status_message`, `daemon_running`. Methods `new()` and `add_log_line()` already present; add `apply_from_config(&Config)`.
- `src/bin/vibe-attack-config.rs` — egui binary host. `show_main_config(ui, &config)` at line 281 is currently read-only and takes a shared ref; must change to `&mut VibeAttackConfigApp` or `&mut ConfigApp` to support Save action. `VibeAttackConfigApp::ui()` at line 209 is where the main routing lives.
- `src/control/client.rs` — `send_command(ControlRequest) -> Result<ControlResponse>` at line 8; `is_daemon_running() -> bool` at line 37; `query_status() -> Option<DaemonStatus>` at line 29. These are the only functions S02 needs; all are already implemented and tested.
- `src/control/protocol.rs` — `ControlRequest` variants (`SetMode`, `SetThreshold`, `SetInputDevice`, `SetPttBinding`, `ReloadConfig`) all exist and are serde-ready.
- `src/config.rs` — `Config` struct (fully `Serialize` + `Deserialize`), `load(path_override)` helper, `default_config_path()`. `Config.stt.confidence_threshold: f32`, `Config.ptt.key: String`, `Config.audio.device: Option<String>`. **Important**: Config uses `#[serde(deny_unknown_fields)]` on all substructs — must use the full `Config` round-trip, not partial writes.

### Build Order

**T01 first** (pure logic, no I/O): extend `ConfigApp` with new fields and unit tests. This unblocks T02 and T03 and is independently testable with `cargo test --lib`.

**T02 second** (I/O helpers): add `load_config_into_app` and `save_app_to_config` helpers. These are testable with tempdir fixtures using the XDG_CONFIG_HOME override pattern already established.

**T03 third** (egui panel): wire the panel. Requires T01 + T02 to be done since it calls both. The existing `spawn_mic_level_thread()` and log-drain patterns in `vibe-attack-config.rs` show how side-effectful work is threaded into the render loop. The CPAL device enumeration for the dropdown (`cpal::default_host().input_devices()`) is a blocking call — run it once at open time and cache in `VibeAttackConfigApp`, not on every frame.

### Verification Approach

- `cargo test --lib --features gui` — must pass all `config_app` unit tests (new fields + load/save helpers)
- `cargo test --features gui` — full test suite must pass (no regressions)
- `cargo check --all-targets --features gui` and `cargo check --all-targets` (no gui) — both must be clean
- `cargo clippy --features gui -D warnings` and `cargo clippy -D warnings` — both must pass
- Manual smoke test: launch `vibe-attack-config` with daemon not running → window opens, shows "daemon not running" status, Save button is disabled or shows error; launch with daemon running → change mode toggle → Save → `vibe-attack` log shows `runtime_command_applied` event.

## Constraints

- `Config` struct uses `#[serde(deny_unknown_fields)]` on every sub-struct — must deserialize the full `Config`, modify target fields, and re-serialize via `serde_yaml_ng`. Partial YAML writes will corrupt the file.
- `show_main_config` must become `&mut` because Save triggers I/O and control-socket calls. The current signature `fn show_main_config(ui: &mut egui::Ui, config: &ConfigApp)` (line 281 of `vibe-attack-config.rs`) must change to accept `&mut VibeAttackConfigApp` or pass `&mut ConfigApp` plus a save callback.
- CPAL device enumeration (`host.input_devices()`) can return an error on headless systems — wrap in a `Result` and fall back to an empty list; never panic.
- The `gui` feature gate: `src/ui/tray.rs` and `src/ui/wizard.rs` are both `#[cfg(feature = "gui")]`. `config_app.rs` is NOT currently gated — it stays ungated since it's pure-logic. Any egui imports must go only in `vibe-attack-config.rs` (already gated by `required-features = ["gui"]` in Cargo.toml).
- PTT key capture (a `evdev` event-read loop) is non-trivial in egui. The milestone context explicitly allows "single key capture" for M008. The safe approach: show a "Press key..." button that opens a brief blocking evdev read on a background thread, then signals back via a channel (same pattern used by `PttCaptureState` in `wizard.rs`). Alternatively, display the current PTT key as read-only text with a note that capture is via "first-run wizard" for M008.
- `serde_yaml_ng` is already in the dependency tree (it's what `config::load()` uses) — no new dependency needed for config save.

## Common Pitfalls

- **`show_main_config` ownership**: The function currently takes `config: &ConfigApp` (immutable). Making it mutable by passing the full `&mut VibeAttackConfigApp` is simpler than threading a save-callback, since it already needs access to `mic`, `tray`, and the log-channel receiver to refresh state on open.
- **CPAL device list on every frame**: Calling `host.input_devices()` inside `ui()` will rebuild the device list every 100ms. Cache it in `VibeAttackConfigApp` (e.g. `device_names: Vec<String>`) at construction time and on explicit Refresh button press.
- **Threshold slider precision**: Config stores `stt.confidence_threshold` as `f32` (0.0–1.0). UI shows 0–100 integer (`u8`). Convert on load: `threshold_pct = (config.stt.confidence_threshold * 100.0).round() as u8` and on save: `threshold = threshold_pct as f32 / 100.0`. Round to avoid float drift (e.g. 0.8 → 80 → 0.8, not 0.80000001).
- **Daemon-not-running Save behavior**: `send_command()` returns `Err` when daemon is absent. The Save button should still write `config.yaml` to disk (so changes persist for next restart) but show a status message like "Saved to disk — daemon not running, runtime changes skipped".
- **`serde_yaml_ng` re-serialize**: Load the full `Config`, apply UI field changes to it, serialize back. Do not build a new `Config` from scratch — it has many fields with `#[serde(default)]` that would be dropped if not round-tripped correctly.
- **`DaemonHandle` not available in `vibe-attack-config`**: The config binary communicates with the daemon via the Unix socket (`send_command`), not via a `DaemonHandle`. There is no in-process handle. All runtime control goes through `control::client::send_command()`.

## Open Risks

- PTT key capture widget: `PttCaptureState` in `wizard.rs` already implements a background evdev listener pattern. If the planner decides to reuse it for S02, verifying the egui repaint loop works correctly during capture is needed. If deferred to read-only display, this risk disappears.
- `serde_yaml_ng` round-trip fidelity: Comments in `config.yaml` will be dropped on save (serde_yaml_ng does not preserve comments). This is acceptable per the milestone context — document it in a status message ("Comments in config.yaml are not preserved on Save").

## Skills Discovered

No specialized external skills required. All technology (egui/eframe, serde_yaml_ng, CPAL, Unix socket) is already in the codebase.
