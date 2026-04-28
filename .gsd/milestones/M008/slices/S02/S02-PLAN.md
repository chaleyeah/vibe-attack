# S02: ConfigApp state + egui config panel

**Goal:** Extend ConfigApp with mode/threshold/input-device/PTT-binding state, add config-file load/save helpers, and wire an egui main config panel in vibe-attack-config that pushes SetMode/SetThreshold/SetInputDevice/SetPttBinding/ReloadConfig commands to the running daemon over the existing control socket — no daemon restart, graceful "daemon not running" handling.
**Demo:** vibe-attack-config opens a config window; user changes mode toggle → Save → daemon log shows SetMode received; threshold slider moves → Save → daemon log shows SetThreshold received

## Must-Haves

- After this slice: vibe-attack-config opens the main config window post-wizard; user changes mode toggle → Save → daemon log shows runtime_command_applied (SetMode); threshold slider moves → Save → daemon log shows dispatcher threshold updated (SetThreshold); when daemon is absent, the window shows a "daemon not running" status and Save still persists to config.yaml without crashing. cargo test --features gui passes; cargo clippy --features gui -- -D warnings clean; cargo clippy -- -D warnings (default features) clean.

## Proof Level

- This slice proves: contract + operational. Real daemon UAT proof for SetMode/SetThreshold runtime apply is held back to S04; this slice proves the state, I/O, and egui wiring contracts via unit tests plus a manual smoke check that the panel renders without panic in both daemon-running and daemon-absent states.

## Integration Closure

Upstream surfaces consumed: control::client::send_command, control::client::is_daemon_running, control::protocol::{ControlRequest::SetMode, SetThreshold, SetInputDevice, SetPttBinding, ReloadConfig, ActivationMode}, config::{Config, load, default_config_path}, cpal::default_host().input_devices(). New wiring introduced: show_main_config becomes &mut self, calls into save_app_to_config + send_command on Save click. Remaining for milestone: S03 wires tray icon state + Mode submenu (consumes the same control protocol); S04 ships the end-to-end UAT and headless integration test that proves daemon honors changes without restart.

## Verification

- UI-level: status_message field on ConfigApp surfaces success/error feedback ("Saved", "Daemon not running — saved to disk", "Failed to send SetMode: <err>"). daemon_running field drives a colored status indicator. The existing tracing channel layer already drains daemon stderr into the log scroll area, so SetMode/SetThreshold round-trips are observable through that surface without new wiring.

## Tasks

- [x] **T01: Extend ConfigApp with mode/threshold/device/PTT state + apply_from_config** `est:30m`
  Add the pure-logic state fields S02 needs onto `ConfigApp` so the egui panel and I/O helpers can be built against them in T02 and T03. Keep this task strictly to in-memory state and pure functions — no egui imports, no socket calls, no filesystem I/O. The new fields are: `mode: ActivationMode`, `threshold_pct: u8` (0–100 integer slider domain; converted to/from `Config.stt.confidence_threshold: f32` at I/O time per MEM gotcha about float drift), `input_device: Option<String>` (mirrors `Config.audio.device`), `ptt_binding: String` (mirrors `Config.ptt.key`), `status_message: Option<String>` (UI status bar), `daemon_running: bool` (driven each frame from `control::client::is_daemon_running()`). Add an `apply_from_config(&mut self, cfg: &Config)` method that copies the four config-derived fields onto the struct and rounds threshold via `(cfg.stt.confidence_threshold * 100.0).round().clamp(0.0, 100.0) as u8`. Add `set_status(&mut self, msg: impl Into<String>)` that writes to `status_message`. ActivationMode is re-exported from `vibe_attack::control::protocol::ActivationMode`; import it in `config_app.rs` (and add `#[derive(PartialEq)]` upstream on ActivationMode if it is missing — verify before changing). Default values when no config has been loaded: `mode = ActivationMode::Ptt`, `threshold_pct = 80`, `input_device = None`, `ptt_binding = String::new()`, `status_message = None`, `daemon_running = false`. Update `ConfigApp::new()` and the `Default` impl. Write unit tests in the existing `#[cfg(test)] mod tests` block: (a) `apply_from_config` round-trips a Config with `stt.confidence_threshold = 0.8` to `threshold_pct = 80`; (b) `apply_from_config` clamps `confidence_threshold = 1.5` to `threshold_pct = 100` and `-0.2` to `0`; (c) `apply_from_config` rounds 0.835 to 84 (no truncation); (d) ActivationMode round-trips through the field; (e) `set_status` writes the message. Do NOT touch `vibe-attack-config.rs` in this task — it still calls `show_main_config(ui, &self.config)` against the old read-only signature. T03 will rewrite that call site.

Must-haves:
- ConfigApp has all six new public fields with the documented types
- apply_from_config method exists and is unit-tested for clamping, rounding, round-trip
- ActivationMode derives `PartialEq` (verify; add if missing — it is needed for egui radio comparison in T03)
- No egui, cpal, std::fs, or socket calls introduced
- `cargo test --lib` passes; `cargo clippy --all-targets -- -D warnings` clean (default features); `cargo clippy --all-targets --features gui -- -D warnings` clean

Assumption: ActivationMode is currently `#[derive(Debug, Clone, Copy, Serialize, Deserialize)]` per MEM030/S01. If `PartialEq` is already derived, leave it. If not, derive it (it is a 2-variant unit enum so this is safe).
  - Files: `src/ui/config_app.rs`, `src/control/protocol.rs`
  - Verify: cargo test --lib config_app && cargo clippy --all-targets -- -D warnings && cargo clippy --all-targets --features gui -- -D warnings

- [x] **T02: Add config.yaml load/save helpers with full-Config round-trip** `est:1h`
  Add two helpers in `src/ui/config_app.rs` that read and write `~/.config/vibe-attack/config.yaml` while round-tripping the full `Config` struct (all sub-structs use `#[serde(deny_unknown_fields)]` per `src/config.rs:8` — partial writes will fail to deserialize on next load). T01 fields are surfaced; this task wires file I/O for them.

Signatures (do not deviate):
- `pub fn load_config_into_app(app: &mut ConfigApp, path_override: Option<&std::path::Path>) -> anyhow::Result<Config>` — calls `crate::config::load(path_override)`, then `app.apply_from_config(&cfg)`, then returns the loaded `Config` for the caller to retain (T03 caches it on `VibeAttackConfigApp`).
- `pub fn save_app_to_config(app: &ConfigApp, current: &Config, path_override: Option<&std::path::Path>) -> anyhow::Result<Config>` — clones `current`, mutates the four owned fields (`stt.confidence_threshold = app.threshold_pct as f32 / 100.0`, `audio.device = app.input_device.clone()`, `ptt.key = app.ptt_binding.clone()`; mode is NOT in Config — see assumption below), serializes via `serde_yaml_ng::to_string`, atomically writes via temp-file + rename to the resolved path (`path_override` or `default_config_path()`), and returns the mutated `Config`. Use `std::fs::write` to a sibling `.tmp` file then `std::fs::rename` for atomic replace.

Assumption to document inline: `ActivationMode` is a runtime-only mode flag in M008; `Config` does not yet have a `mode` field. The Save path therefore sends `SetMode` over the control socket (T03) but does NOT persist mode to YAML. This is consistent with the milestone scope — adding a `mode` field to YAML would change the schema and is out of scope. Add a code comment in `save_app_to_config` documenting this.

Unit tests (use the existing `XDG_CONFIG_HOME` + `serial_test::serial` pattern from `load_profiles` tests at `src/ui/config_app.rs:99`):
- `load_config_into_app_populates_state`: write a minimal valid config.yaml to a tempdir, call `load_config_into_app(&mut app, Some(path))`, assert `app.threshold_pct`, `app.input_device`, `app.ptt_binding` match expected values.
- `save_app_to_config_round_trips`: load a config, mutate `app.threshold_pct = 50` and `app.input_device = Some("plughw:CARD=Test".to_string())`, call save, then `crate::config::load(Some(path))` and assert the new values landed.
- `save_app_to_config_preserves_unknown_macros`: load a config containing 2 macros, save without touching macros, reload, assert all 2 macros survived round-trip — proves we did not lose adjacent fields.
- `save_app_to_config_atomic`: assert the .tmp file does not remain after a successful save (no leftover sibling files in the directory).

Must-haves:
- Helpers are public and live in `src/ui/config_app.rs` (no new module — keep T03's footprint small)
- Full `Config` round-trip preserves macros, vad, wake, pipeline, timing sub-structs (proven by test)
- Atomic write via tmp+rename — partial saves on crash are not visible to next load
- Tests use tempdir + XDG_CONFIG_HOME override + `serial_test::serial` (per MEM008)
- `cargo test --lib config_app` passes; clippy clean for both default and gui features

For the test fixture YAML, use the minimum required fields per `Config` definition: `ptt.key`, `timing.dwell_ms`, `timing.gap_ms`. Everything else can rely on `#[serde(default)]` impls. Reference `config.example.yaml` at the repo root for a known-good template if needed.
  - Files: `src/ui/config_app.rs`
  - Verify: cargo test --lib config_app && cargo clippy --all-targets -- -D warnings && cargo clippy --all-targets --features gui -- -D warnings

- [x] **T03: Wire egui main config panel + Save dispatch to control socket** `est:2h`
  Replace the current read-only `show_main_config(ui: &mut egui::Ui, config: &ConfigApp)` at `src/bin/vibe-attack-config.rs:281` with a mutable variant that lets the user edit mode/threshold/input-device/PTT-binding and trigger a Save action that (a) writes config.yaml via `save_app_to_config` and (b) sends control commands over the daemon socket via `control::client::send_command`. Touching the existing wizard, mic-level, log-drain, or tray paths is out of scope — leave them unchanged.

Signature change: `fn show_main_config(ui: &mut egui::Ui, app: &mut VibeAttackConfigApp)` (full app handle so Save can read `app.cached_config`, mutate `app.config`, and call socket helpers). Update the caller at `vibe-attack-config.rs:276` from `show_main_config(ui, &self.config)` to `show_main_config(ui, self)`.

New fields on `VibeAttackConfigApp`:
- `cached_config: Option<vibe_attack::config::Config>` — populated on first wizard-complete frame (or immediately at startup if `setup_complete`) by calling `vibe_attack::ui::config_app::load_config_into_app(&mut self.config, None)`. Stored to feed back into `save_app_to_config(&app.config, &cached_config, None)` so we round-trip the full Config.
- `device_names: Vec<String>` — populated once at startup via `cpal::default_host().input_devices().map(|iter| iter.filter_map(|d| d.name().ok()).collect()).unwrap_or_default()`. Do NOT enumerate devices on every frame.

Load-on-startup logic: in `VibeAttackConfigApp::new`, after `setup_complete` branch sets profiles and mic, also call `load_config_into_app` and stash the returned Config in `cached_config`. Wrap in `match` and on `Err` set `config.set_status(format!("Could not load config.yaml: {e}"))` and leave `cached_config = None`. Mirror the same load in the `setup_just_completed` branch at `src/bin/vibe-attack-config.rs:250`.

Daemon-running detection: at the top of each frame (inside `ui()` after the existing log drain, before `show_main_config`), refresh `self.config.daemon_running = vibe_attack::control::client::is_daemon_running();`. This is a stat() on a Unix socket — cheap.

Panel layout in `show_main_config` (top to bottom):
1. Daemon status row — green dot label "Daemon: running" if `app.config.daemon_running`, else amber "Daemon: not running (changes will save to disk only)".
2. Mic level row — preserve the existing widget (lift it from the old function unchanged).
3. Activation mode — `ui.horizontal { ui.label("Mode:"); ui.radio_value(&mut app.config.mode, ActivationMode::Ptt, "Push-to-talk"); ui.radio_value(&mut app.config.mode, ActivationMode::Wake, "Wake word"); }`. Requires `ActivationMode: PartialEq` from T01.
4. Confidence threshold — `ui.add(egui::Slider::new(&mut app.config.threshold_pct, 0..=100).text("Confidence threshold (%)"));`.
5. Input device — `egui::ComboBox::from_label("Input device").selected_text(...).show_ui(...)`: each option is one entry in `app.device_names` plus a leading "<system default>" option that maps to `None`. Selection writes `app.config.input_device`.
6. PTT binding — `ui.horizontal { ui.label(format!("PTT key: {}", app.config.ptt_binding)); }` (read-only display for M008 — do not implement live key capture in this task; first-run wizard already sets this and runtime change requires daemon restart per S01 follow-up notes).
7. Save button — `if ui.button("Save").clicked() { handle_save(app); }`.
8. Status message — `if let Some(msg) = &app.config.status_message { ui.label(msg); }`.
9. Profiles list and log scroll area — preserve the existing widgets (lift from old function unchanged).

Save handler — implement as a free function `fn handle_save(app: &mut VibeAttackConfigApp)` in `vibe-attack-config.rs`:
1. If `app.cached_config.is_none()`: set status to "No config loaded — cannot save." and return.
2. Call `save_app_to_config(&app.config, app.cached_config.as_ref().unwrap(), None)`. On `Err`, set status `format!("Save failed: {e}")` and return.
3. On `Ok(updated)`: replace `app.cached_config = Some(updated)`.
4. If `!app.config.daemon_running`: set status "Saved to disk — daemon not running, runtime changes skipped." and return.
5. Otherwise dispatch four control requests in order: `SetMode { mode: app.config.mode }`, `SetThreshold { threshold: app.config.threshold_pct as f32 / 100.0 }`, `SetInputDevice { device: app.config.input_device.clone() }`, `SetPttBinding { key: app.config.ptt_binding.clone() }`. Each `send_command` failure should append to status (collect into a Vec<String>, join with "; "). On full success, set status to "Saved and applied.".

Slice-level smoke verification: a manual run of `cargo run --features gui --bin vibe-attack-config` after the wizard completes should show the new panel without crash; daemon-not-running case shows the amber status; clicking Save with no daemon writes config.yaml and shows the disk-only status. The verification command below covers compile + clippy; the manual smoke goes in the slice's UAT-style log.

Must-haves:
- `show_main_config` signature is `&mut VibeAttackConfigApp`; old read-only call site updated
- `cached_config` and `device_names` populated once (startup or wizard-complete frame), not per-frame
- `daemon_running` refreshed each frame via `is_daemon_running`
- Save button persists to disk on every click; only fires control commands when daemon is running
- Daemon-absent path never panics and always leaves the user with a usable status message
- `cargo build --features gui --bin vibe-attack-config` succeeds; `cargo clippy --all-targets --features gui -- -D warnings` clean; `cargo clippy --all-targets -- -D warnings` (default features) clean; `cargo test --features gui` passes
  - Files: `src/bin/vibe-attack-config.rs`
  - Verify: cargo build --features gui --bin vibe-attack-config && cargo test --features gui && cargo clippy --all-targets --features gui -- -D warnings && cargo clippy --all-targets -- -D warnings

## Files Likely Touched

- src/ui/config_app.rs
- src/control/protocol.rs
- src/bin/vibe-attack-config.rs
