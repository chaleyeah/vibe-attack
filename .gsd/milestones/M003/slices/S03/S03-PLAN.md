# S03: Wizard UI panels

**Goal:** Replace the stub wizard rendering in vibe-attack-config.rs with real per-step egui panels. Each step panel performs the real action (copy file, re-probe, capture PTT key via evdev thread) and refreshes FirstRunState after each action. When all steps pass the app transitions to the main ConfigApp view.
**Demo:** Launching vibe-attack-config on a system missing prerequisites shows the wizard; clicking through each step performs the real action; after all steps pass, the app transitions to the main config view

## Must-Haves

- Four wizard step panels render without panic; CreateConfig button copies example config and re-probes; InstallModel shows curl command and re-probes on Re-check; SetupUinput shows commands and re-probes; ConfigurePtt captures a keypress from evdev and writes KEY_* name to config ptt.key field; transition to ConfigApp occurs when is_setup_complete() returns true

## Proof Level

- This slice proves: Manual launch of vibe-attack-config --features gui on dev machine with at least one step incomplete; PTT capture verified by pressing a key and checking config file

## Integration Closure

Wizard panels call probe::run() after each action; transition to ConfigApp happens when is_setup_complete() is true; PTT key is written to the config file's ptt.key field

## Verification

- Each panel action logs tracing::info on success or tracing::error on failure; PTT capture thread logs startup, keypress received, and clean exit

## Tasks

- [x] **T01: Add wizard module src/ui/wizard.rs with panel routing** `est:30m`
  Create src/ui/wizard.rs (feature-gated to gui via #[cfg(feature = "gui")] or kept as conditional import). Define a WizardPanel trait or simple show_wizard(ui, state) function that dispatches to the correct panel based on first_incomplete_step(). The dispatcher refreshes state after each action by calling probe::run(). Update vibe-attack-config.rs to call show_wizard instead of the debug label loop.
  - Files: `src/ui/wizard.rs`, `src/ui/mod.rs`, `src/bin/vibe-attack-config.rs`
  - Verify: cargo check --lib and cargo check --bin vibe-attack-config --features gui both pass (no errors from wizard source)

- [x] **T02: Implement CreateConfig and InstallModel panels** `est:40m`
  In wizard.rs, implement show_create_config(ui, state) and show_install_model(ui, state). CreateConfig: heading 'Create config file', shows target path, a 'Copy example config' button that calls std::fs::copy then probe::run() to refresh. InstallModel: heading 'Install whisper model', shows model path, shows the curl command in a monospace code block (ui.monospace), a 'Re-check' button that calls probe::run(). Both panels show a success indicator when their check already passes.
  - Files: `src/ui/wizard.rs`
  - Verify: Panels compile; CreateConfig button copies file when clicked (manual check)

- [x] **T03: Implement SetupUinput panel** `est:20m`
  In wizard.rs, implement show_setup_uinput(ui). Heading 'Set up uinput access'. Shows two code blocks: 'sudo modprobe uinput' and 'sudo usermod -aG input $USER'. Shows the systemd v258+/CachyOS note. Shows a 'Re-check' button that calls probe::run() to refresh state. No sudo commands are run by the app itself — the panel is informational with re-probe.
  - Files: `src/ui/wizard.rs`
  - Verify: Panel renders without panic; Re-check button calls probe::run()

- [x] **T04: Implement ConfigurePtt panel with evdev capture thread** `est:60m`
  In wizard.rs, implement show_configure_ptt(ui, state, ptt_state). PttCaptureState holds: thread handle Option, captured_key Arc<Mutex<Option<String>>>, listening bool. On 'Listen for key' button click: spawn a thread that opens the first available evdev keyboard device, calls fetch_events() in a loop, on first KeyDown event converts the key to its name string (format!("{:?}", key)), stores it in captured_key, and exits. On next update() call, check captured_key and if Some: write it to config file's ptt.key field via a simple text replacement (read config, replace/append ptt.key line), call probe::run() to refresh. Show 'Listening... press any key' label while thread is running. Show captured key name when done.
  - Files: `src/ui/wizard.rs`, `src/ui/mod.rs`
  - Verify: Thread starts on button click; pressing a key stores the name; config file is updated; probe::run() returns ptt_configured=true after write

## Files Likely Touched

- src/ui/wizard.rs
- src/ui/mod.rs
- src/bin/vibe-attack-config.rs
