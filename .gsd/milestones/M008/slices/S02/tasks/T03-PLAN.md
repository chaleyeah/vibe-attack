---
estimated_steps: 31
estimated_files: 1
skills_used: []
---

# T03: Wire egui main config panel + Save dispatch to control socket

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

## Inputs

- ``src/bin/vibe-attack-config.rs``
- ``src/ui/config_app.rs``
- ``src/control/client.rs``
- ``src/control/protocol.rs``

## Expected Output

- ``src/bin/vibe-attack-config.rs``

## Verification

cargo build --features gui --bin vibe-attack-config && cargo test --features gui && cargo clippy --all-targets --features gui -- -D warnings && cargo clippy --all-targets -- -D warnings

## Observability Impact

Crosses three runtime boundaries: filesystem (config.yaml save), Unix socket (control commands), and CPAL host enumeration. All three failure paths must surface through ConfigApp.status_message — never a panic. CPAL device enumeration is run once at startup; failures degrade to an empty device list, not a crash.
