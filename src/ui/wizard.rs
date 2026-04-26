//! First-run wizard panels.
//!
//! show_wizard() is the entry point. It dispatches to the correct step panel
//! based on first_incomplete_step(), re-probes after each action, and signals
//! completion when is_setup_complete() is true.
//!
//! All panels are feature-gated to `gui` — only compiled when eframe is present.

#[cfg(feature = "gui")]
pub use inner::*;

#[cfg(feature = "gui")]
mod inner {
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    use eframe::egui;
    use evdev::{Device, EventSummary};
    use tracing::{error, info};

    use crate::ui::first_run::{FirstRunState, SetupStep};
    use crate::ui::probe;

    // ── PTT capture state ────────────────────────────────────────────────────

    /// Shared state for the PTT key capture background thread.
    pub struct PttCaptureState {
        pub listening: bool,
        pub captured_key: Arc<Mutex<Option<String>>>,
        pub handle: Option<std::thread::JoinHandle<()>>,
        pub error: Option<String>,
    }

    impl PttCaptureState {
        pub fn new() -> Self {
            Self {
                listening: false,
                captured_key: Arc::new(Mutex::new(None)),
                handle: None,
                error: None,
            }
        }

        /// True when the thread has finished and deposited a key name.
        pub fn has_result(&self) -> bool {
            self.captured_key.lock().map(|g| g.is_some()).unwrap_or(false)
        }

        /// Take the captured key name, leaving None in its place.
        pub fn take_result(&self) -> Option<String> {
            self.captured_key.lock().ok()?.take()
        }
    }

    impl Default for PttCaptureState {
        fn default() -> Self {
            Self::new()
        }
    }

    // ── Wizard entry point ───────────────────────────────────────────────────

    /// Render the wizard panel for the current incomplete step.
    ///
    /// Returns an updated `FirstRunState` — call probe::run() after actions.
    /// If setup is complete, returns the current state unchanged.
    pub fn show_wizard(
        ui: &mut egui::Ui,
        state: &mut FirstRunState,
        ptt: &mut PttCaptureState,
        config_example_path: &PathBuf,
    ) {
        // Harvest PTT result from background thread if ready
        if ptt.listening && ptt.has_result() {
            if let Some(key_name) = ptt.take_result() {
                ptt.listening = false;
                match write_ptt_key_to_config(&key_name) {
                    Ok(()) => {
                        info!(key = %key_name, "PTT key written to config");
                        *state = probe::run();
                    }
                    Err(e) => {
                        error!(key = %key_name, reason = %e, "Failed to write PTT key to config");
                        ptt.error = Some(format!("Failed to save key: {e}"));
                    }
                }
            }
        }

        // Reap finished thread handle to avoid leaking it
        if let Some(h) = &ptt.handle {
            if h.is_finished() {
                if let Some(handle) = ptt.handle.take() {
                    let _ = handle.join();
                }
            }
        }

        match state.first_incomplete_step() {
            None => {
                ui.heading("Setup complete");
                ui.label("All prerequisites satisfied. Loading config app…");
            }
            Some(SetupStep::CreateConfig) => {
                show_create_config(ui, state, config_example_path);
            }
            Some(SetupStep::InstallModel) => {
                show_install_model(ui, state);
            }
            Some(SetupStep::SetupUinput) => {
                show_setup_uinput(ui, state);
            }
            Some(SetupStep::ConfigurePtt) => {
                show_configure_ptt(ui, state, ptt);
            }
        }
    }

    // ── Step: CreateConfig ───────────────────────────────────────────────────

    fn show_create_config(
        ui: &mut egui::Ui,
        state: &mut FirstRunState,
        config_example_path: &PathBuf,
    ) {
        let target = crate::ui::probe::config_path_for_display();
        ui.heading("Step 1 of 4: Create config file");
        ui.add_space(8.0);
        ui.label(format!("Target: {target}"));
        ui.add_space(12.0);

        if ui.button("Copy example config").clicked() {
            match std::fs::create_dir_all(
                PathBuf::from(&target).parent().unwrap_or(&PathBuf::from(".")),
            )
            .and_then(|_| std::fs::copy(config_example_path, &target).map(|_| ()))
            {
                Ok(()) => {
                    info!(path = %target, "Config file created");
                    *state = probe::run();
                }
                Err(e) => {
                    error!(reason = %e, "Failed to copy config file");
                    ui.colored_label(egui::Color32::RED, format!("Error: {e}"));
                }
            }
        }

        ui.add_space(8.0);
        ui.label("After copying, edit the file to set your audio device and PTT key.");
    }

    // ── Step: InstallModel ───────────────────────────────────────────────────

    fn show_install_model(ui: &mut egui::Ui, state: &mut FirstRunState) {
        let model_path = crate::ui::probe::model_path_for_display();
        let curl_cmd = format!(
            "mkdir -p \"{}\"\ncurl -L -o \"{}\" \\\n  {}",
            PathBuf::from(&model_path)
                .parent()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            model_path,
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin",
        );

        ui.heading("Step 2 of 4: Install whisper model");
        ui.add_space(8.0);
        ui.label(format!("Target: {model_path}"));
        ui.add_space(8.0);
        ui.label("Run this command in your terminal:");
        ui.add_space(4.0);

        egui::ScrollArea::horizontal().show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut curl_cmd.as_str())
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .interactive(false),
            );
        });

        ui.add_space(12.0);
        if ui.button("Re-check").clicked() {
            *state = probe::run();
        }
    }

    // ── Step: SetupUinput ────────────────────────────────────────────────────

    fn show_setup_uinput(ui: &mut egui::Ui, state: &mut FirstRunState) {
        ui.heading("Step 3 of 4: Set up uinput access");
        ui.add_space(8.0);
        ui.label("Vibe Attack needs /dev/uinput to inject key events into your game.");
        ui.add_space(8.0);

        ui.label("1. Load the uinput kernel module:");
        egui::Frame::NONE
            .fill(egui::Color32::from_gray(30))
            .inner_margin(egui::Margin::same(6))
            .show(ui, |ui| {
                ui.monospace("sudo modprobe uinput");
            });
        ui.add_space(4.0);
        ui.label("Optional — persist across reboots:");
        egui::Frame::NONE
            .fill(egui::Color32::from_gray(30))
            .inner_margin(egui::Margin::same(6))
            .show(ui, |ui| {
                ui.monospace("echo \"uinput\" | sudo tee /etc/modules-load.d/uinput.conf");
            });
        ui.add_space(8.0);

        ui.label("2. Add yourself to the input group:");
        egui::Frame::NONE
            .fill(egui::Color32::from_gray(30))
            .inner_margin(egui::Margin::same(6))
            .show(ui, |ui| {
                ui.monospace("sudo usermod -aG input $USER");
            });
        ui.add_space(4.0);
        ui.label("3. Apply without logging out:");
        egui::Frame::NONE
            .fill(egui::Color32::from_gray(30))
            .inner_margin(egui::Margin::same(6))
            .show(ui, |ui| {
                ui.monospace("newgrp input");
            });
        ui.add_space(8.0);

        ui.colored_label(
            egui::Color32::YELLOW,
            "Note (systemd v258+ / CachyOS 2025+): use the 'input' group, not 'uinput'.",
        );

        ui.add_space(12.0);
        if ui.button("Re-check").clicked() {
            *state = probe::run();
        }
    }

    // ── Step: ConfigurePtt ───────────────────────────────────────────────────

    fn show_configure_ptt(
        ui: &mut egui::Ui,
        state: &mut FirstRunState,
        ptt: &mut PttCaptureState,
    ) {
        ui.heading("Step 4 of 4: Configure PTT key");
        ui.add_space(8.0);
        ui.label("Click 'Listen for key', then press the key you want to use as Push-to-Talk.");
        ui.add_space(8.0);

        if ptt.listening {
            ui.spinner();
            ui.label("Listening… press any key now.");
        } else if !ptt.listening && ptt.handle.is_none() {
            if ui.button("Listen for key").clicked() {
                ptt.error = None;
                let captured = Arc::clone(&ptt.captured_key);
                let handle = std::thread::spawn(move || {
                    capture_first_keypress(captured);
                });
                ptt.handle = Some(handle);
                ptt.listening = true;
            }
        }

        if let Some(err) = &ptt.error {
            ui.add_space(8.0);
            ui.colored_label(egui::Color32::RED, err.as_str());
        }

        ui.add_space(8.0);
        ui.label("Or enter a key name manually (e.g. KEY_GRAVE, KEY_F13):");
        let mut manual_key = String::new();
        let resp = ui.text_edit_singleline(&mut manual_key);
        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            let key_name = manual_key.trim().to_string();
            if !key_name.is_empty() {
                match write_ptt_key_to_config(&key_name) {
                    Ok(()) => {
                        info!(key = %key_name, "PTT key written to config (manual entry)");
                        *state = probe::run();
                    }
                    Err(e) => {
                        ptt.error = Some(format!("Failed to save key: {e}"));
                    }
                }
            }
        }
    }

    // ── PTT capture thread ───────────────────────────────────────────────────

    /// Open the first readable keyboard evdev device and wait for the first KeyDown event.
    /// Stores the key name in `captured` and returns.
    fn capture_first_keypress(captured: Arc<Mutex<Option<String>>>) {
        info!("PTT capture thread started");

        let device = match find_keyboard_device() {
            Some(d) => d,
            None => {
                error!("PTT capture: no readable keyboard device found in /dev/input/event*");
                return;
            }
        };

        info!(
            device = device.name().unwrap_or("unknown"),
            "PTT capture: using device"
        );

        let mut device = device;
        loop {
            match device.fetch_events() {
                Ok(events) => {
                    for event in events {
                        if let EventSummary::Key(_, key, 1) = event.destructure() {
                            let name = format!("{key:?}");
                            info!(key = %name, "PTT capture: key pressed");
                            if let Ok(mut guard) = captured.lock() {
                                *guard = Some(name);
                            }
                            return;
                        }
                    }
                }
                Err(e) => {
                    error!(reason = %e, "PTT capture: device read error");
                    return;
                }
            }
        }
    }

    fn find_keyboard_device() -> Option<Device> {
        // Prefer devices that report standard keyboard keys (KEY_A is a reliable indicator)
        for (_path, device) in evdev::enumerate() {
            if device
                .supported_keys()
                .map_or(false, |keys| keys.contains(evdev::KeyCode::KEY_A))
            {
                return Some(device);
            }
        }
        None
    }

    // ── Config file write ────────────────────────────────────────────────────

    /// Write `key: <key_name>` to the ptt section of the config file.
    ///
    /// Reads the existing config, replaces the ptt.key line if present,
    /// or appends a ptt section if the file has no ptt.key line.
    fn write_ptt_key_to_config(key_name: &str) -> std::io::Result<()> {
        let path = crate::ui::probe::config_path_for_display();
        let path = PathBuf::from(path);

        let contents = std::fs::read_to_string(&path).unwrap_or_default();
        let new_contents = rewrite_ptt_key(&contents, key_name);
        std::fs::write(&path, new_contents)
    }

    /// Pure function: rewrite config text to set ptt.key = key_name.
    ///
    /// If a `key: KEY_*` line already exists, replaces it.
    /// If a `# key:` commented line exists, replaces that.
    /// Otherwise appends a ptt block at the end.
    pub(crate) fn rewrite_ptt_key(config: &str, key_name: &str) -> String {
        let mut found = false;
        let mut lines: Vec<String> = config
            .lines()
            .map(|line| {
                let trimmed = line.trim();
                // Match existing key: line (active or commented)
                if !found
                    && (trimmed.starts_with("key:") || trimmed.starts_with("# key:"))
                    && trimmed.contains("KEY_")
                {
                    found = true;
                    // Preserve indentation
                    let indent = line.len() - line.trim_start().len();
                    format!("{}key: {key_name}", " ".repeat(indent))
                } else {
                    line.to_string()
                }
            })
            .collect();

        if !found {
            lines.push(String::new());
            lines.push("ptt:".to_string());
            lines.push(format!("  key: {key_name}"));
        }

        let mut result = lines.join("\n");
        if !result.ends_with('\n') {
            result.push('\n');
        }
        result
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "gui")]
    use super::inner::rewrite_ptt_key;

    #[test]
    #[cfg(feature = "gui")]
    fn rewrite_ptt_key_replaces_existing_active_key() {
        let cfg = "ptt:\n  key: KEY_LEFTCTRL\n";
        let result = rewrite_ptt_key(cfg, "KEY_GRAVE");
        assert!(result.contains("key: KEY_GRAVE"), "should replace key");
        assert!(!result.contains("KEY_LEFTCTRL"), "should remove old key");
    }

    #[test]
    #[cfg(feature = "gui")]
    fn rewrite_ptt_key_replaces_commented_key() {
        let cfg = "ptt:\n  # key: KEY_LEFTCTRL\n";
        let result = rewrite_ptt_key(cfg, "KEY_F13");
        assert!(result.contains("key: KEY_F13"), "should replace commented key");
    }

    #[test]
    #[cfg(feature = "gui")]
    fn rewrite_ptt_key_appends_when_no_ptt_section() {
        let cfg = "stt:\n  enabled: false\n";
        let result = rewrite_ptt_key(cfg, "KEY_GRAVE");
        assert!(result.contains("ptt:"), "should append ptt section");
        assert!(result.contains("key: KEY_GRAVE"), "should contain new key");
    }
}
