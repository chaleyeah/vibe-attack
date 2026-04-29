//! First-run wizard panels.
//!
//! show_wizard() is the entry point. It dispatches to the correct step panel
//! based on first_incomplete_step(), re-probes after each action, and signals
//! completion when is_setup_complete() is true.
//!
//! All panels are feature-gated to `gui` — only compiled when eframe is present.

/// Re-export all wizard types and the `show_wizard` entry point when the `gui` feature is enabled.
#[cfg(feature = "gui")]
pub use inner::*;

#[cfg(feature = "gui")]
mod inner {
    use std::io::Read;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    use eframe::egui;
    use evdev::{Device, EventSummary};
    use tracing::{error, info};

    use crate::ui::first_run::{FirstRunState, SetupStep};
    use crate::ui::probe;

    const MODEL_URL: &str =
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin";

    // ── Download state ───────────────────────────────────────────────────────

    /// Progress state of the background model download.
    #[derive(Debug, Clone, PartialEq)]
    pub enum DownloadStatus {
        /// No download in progress.
        Idle,
        /// Download underway; `total` is `None` when the server omits Content-Length.
        Downloading { received: u64, total: Option<u64> },
        /// Download finished and the file has been moved into place.
        Done,
        /// Download failed; the inner string is a human-readable error message.
        Failed(String),
    }

    /// Shared state for the model download background thread.
    pub struct ModelDownloadState {
        /// Current download progress, shared with the download thread via `Arc<Mutex<_>>`.
        pub status: Arc<Mutex<DownloadStatus>>,
        /// Join handle for the download thread; `None` when no download is running.
        pub handle: Option<std::thread::JoinHandle<()>>,
    }

    impl ModelDownloadState {
        /// Construct initial state with `Idle` status and no thread handle.
        pub fn new() -> Self {
            Self {
                status: Arc::new(Mutex::new(DownloadStatus::Idle)),
                handle: None,
            }
        }

        /// Return a clone of the current download status without blocking on failure.
        pub fn current(&self) -> DownloadStatus {
            self.status.lock().map(|g| g.clone()).unwrap_or(DownloadStatus::Idle)
        }

        /// True while the download thread is actively transferring bytes.
        pub fn is_running(&self) -> bool {
            matches!(self.current(), DownloadStatus::Downloading { .. })
        }
    }

    impl Default for ModelDownloadState {
        fn default() -> Self {
            Self::new()
        }
    }

    // ── Uinput setup state ───────────────────────────────────────────────────

    /// Lifecycle state of a single privileged uinput setup action (modprobe or usermod).
    #[derive(Debug, Clone, PartialEq)]
    pub enum SetupActionStatus {
        /// Action has not been initiated yet.
        Idle,
        /// pkexec subprocess is running; poll the thread handle to detect completion.
        Running,
        /// Action completed successfully.
        Done,
        /// Action failed; the inner string is a human-readable error message.
        Failed(String),
    }

    /// Per-action status for the two privileged uinput setup steps.
    pub struct UinputSetupState {
        /// Status of the `modprobe uinput` step.
        pub modprobe: SetupActionStatus,
        /// Status of the `usermod -aG input` step.
        pub usermod: SetupActionStatus,
        /// Join handle for the modprobe pkexec thread; `None` when not running.
        pub modprobe_handle: Option<std::thread::JoinHandle<Result<(), String>>>,
        /// Join handle for the usermod pkexec thread; `None` when not running.
        pub usermod_handle: Option<std::thread::JoinHandle<Result<(), String>>>,
    }

    impl UinputSetupState {
        /// Construct with all actions in `Idle` state and no thread handles.
        pub fn new() -> Self {
            Self {
                modprobe: SetupActionStatus::Idle,
                usermod: SetupActionStatus::Idle,
                modprobe_handle: None,
                usermod_handle: None,
            }
        }
    }

    impl Default for UinputSetupState {
        fn default() -> Self {
            Self::new()
        }
    }

    // ── PTT capture state ────────────────────────────────────────────────────

    /// Shared state for the PTT key capture background thread.
    pub struct PttCaptureState {
        /// True while the capture thread is waiting for a keypress.
        pub listening: bool,
        /// Receives the evdev key name from the capture thread when a key is pressed.
        pub captured_key: Arc<Mutex<Option<String>>>,
        /// Join handle for the capture thread; `None` when not running.
        pub handle: Option<std::thread::JoinHandle<()>>,
        /// Last capture error message, shown in the wizard UI.
        pub error: Option<String>,
        /// Manual key name typed by the user; persisted across frames so edits are not lost.
        pub manual_key: String,
    }

    impl PttCaptureState {
        /// Construct with `listening = false` and no captured key or thread.
        pub fn new() -> Self {
            Self {
                listening: false,
                captured_key: Arc::new(Mutex::new(None)),
                handle: None,
                error: None,
                manual_key: String::new(),
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
        dl: &mut ModelDownloadState,
        uinput: &mut UinputSetupState,
        config_example_contents: &str,
        hd2_profile_contents: &str,
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

        // Re-probe after download completes, then reap the handle.
        if let Some(h) = &dl.handle {
            if h.is_finished() {
                if let Ok(DownloadStatus::Done) = dl.status.lock().map(|g| g.clone()) {
                    *state = probe::run();
                }
                if let Some(handle) = dl.handle.take() {
                    let _ = handle.join();
                }
            }
        }

        // Harvest modprobe and usermod thread results.
        if let Some(h) = &uinput.modprobe_handle {
            if h.is_finished() {
                if let Some(handle) = uinput.modprobe_handle.take() {
                    uinput.modprobe = match handle.join() {
                        Ok(Ok(())) => SetupActionStatus::Done,
                        Ok(Err(e)) => SetupActionStatus::Failed(e),
                        Err(_) => SetupActionStatus::Failed("thread panicked".to_string()),
                    };
                    *state = probe::run();
                }
            }
        }
        if let Some(h) = &uinput.usermod_handle {
            if h.is_finished() {
                if let Some(handle) = uinput.usermod_handle.take() {
                    uinput.usermod = match handle.join() {
                        Ok(Ok(())) => SetupActionStatus::Done,
                        Ok(Err(e)) => SetupActionStatus::Failed(e),
                        Err(_) => SetupActionStatus::Failed("thread panicked".to_string()),
                    };
                    *state = probe::run();
                }
            }
        }

        match state.first_incomplete_step() {
            None => {
                ui.heading("Setup complete");
                ui.label("All prerequisites satisfied. Loading config app…");
            }
            Some(SetupStep::CreateConfig) => {
                show_create_config(ui, state, config_example_contents, hd2_profile_contents);
            }
            Some(SetupStep::InstallModel) => {
                show_install_model(ui, state, dl);
            }
            Some(SetupStep::SetupUinput) => {
                show_setup_uinput(ui, state, uinput);
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
        config_example_contents: &str,
        hd2_profile_contents: &str,
    ) {
        let target = crate::ui::probe::config_path_for_display();
        ui.heading("Step 1 of 4: Create config file");
        ui.add_space(8.0);
        ui.label(format!("Target: {target}"));
        ui.add_space(12.0);

        if ui.button("Copy example config").clicked() {
            let target_path = PathBuf::from(&target);
            let parent = target_path.parent().unwrap_or(&target_path);
            match std::fs::create_dir_all(parent)
                .and_then(|_| std::fs::write(&target_path, config_example_contents))
            {
                Ok(()) => {
                    info!(path = %target, "Config file created");
                    install_default_profile(hd2_profile_contents);
                    *state = probe::run();
                }
                Err(e) => {
                    error!(reason = %e, "Failed to write config file");
                    ui.colored_label(egui::Color32::RED, format!("Error: {e}"));
                }
            }
        }

        ui.add_space(8.0);
        ui.label("After copying, edit the file to set your audio device and PTT key.");
    }

    /// Write the bundled hd2 profile to the XDG profiles directory if not already present.
    fn install_default_profile(hd2_profile_contents: &str) {
        let profiles_dir = xdg::BaseDirectories::with_prefix("vibe-attack")
            .get_config_home()
            .map(|p| p.join("profiles"));

        let Some(dir) = profiles_dir else {
            error!("Could not resolve XDG config path for profiles");
            return;
        };

        let dest = dir.join("hd2.yaml");
        if dest.exists() {
            info!(path = %dest.display(), "hd2 profile already present — skipping");
            return;
        }

        if let Err(e) = std::fs::create_dir_all(&dir) {
            error!(reason = %e, "Failed to create profiles directory");
            return;
        }

        match std::fs::write(&dest, hd2_profile_contents) {
            Ok(()) => info!(path = %dest.display(), "Default hd2 profile installed"),
            Err(e) => error!(reason = %e, path = %dest.display(), "Failed to write hd2 profile"),
        }
    }

    // ── Step: InstallModel ───────────────────────────────────────────────────

    fn show_install_model(
        ui: &mut egui::Ui,
        state: &mut FirstRunState,
        dl: &mut ModelDownloadState,
    ) {
        let model_path = crate::ui::probe::model_path_for_display();

        ui.heading("Step 2 of 4: Install whisper model");
        ui.add_space(8.0);
        ui.label(format!("Target: {model_path}"));
        ui.add_space(12.0);

        match dl.current() {
            DownloadStatus::Idle => {
                ui.label("~75 MB download from HuggingFace.");
                ui.add_space(8.0);
                if ui.button("Download model").clicked() {
                    let status = Arc::clone(&dl.status);
                    let dest = model_path.clone();
                    let handle = std::thread::spawn(move || {
                        download_model(status, &dest);
                    });
                    dl.handle = Some(handle);
                    // Immediately set to downloading so next frame shows progress
                    if let Ok(mut g) = dl.status.lock() {
                        *g = DownloadStatus::Downloading { received: 0, total: None };
                    }
                }
            }
            DownloadStatus::Downloading { received, total } => {
                ui.spinner();
                match total {
                    Some(total) if total > 0 => {
                        let frac = received as f32 / total as f32;
                        ui.add(
                            egui::ProgressBar::new(frac)
                                .desired_width(300.0)
                                .text(format!(
                                    "{:.1} / {:.1} MB",
                                    received as f64 / 1_048_576.0,
                                    total as f64 / 1_048_576.0
                                )),
                        );
                    }
                    _ => {
                        ui.label(format!(
                            "Downloading… {:.1} MB received",
                            received as f64 / 1_048_576.0
                        ));
                    }
                }
                // Fast repaint while downloading
                ui.ctx().request_repaint_after(std::time::Duration::from_millis(250));
            }
            DownloadStatus::Done => {
                // Re-probe on every frame while Done so the wizard advances automatically
                // when the user re-enters this step after the download handle was already reaped.
                *state = probe::run();
                ui.colored_label(egui::Color32::GREEN, "Download complete.");
                ui.add_space(4.0);
                if ui.button("Re-check").clicked() {
                    *state = probe::run();
                }
            }
            DownloadStatus::Failed(msg) => {
                ui.colored_label(egui::Color32::RED, format!("Download failed: {msg}"));
                ui.add_space(8.0);
                if ui.button("Retry").clicked() {
                    if let Ok(mut g) = dl.status.lock() {
                        *g = DownloadStatus::Idle;
                    }
                }
            }
        }
    }

    /// Download the model file to `dest`, streaming progress into `status`.
    fn download_model(status: Arc<Mutex<DownloadStatus>>, dest: &str) {
        info!(url = MODEL_URL, dest, "starting model download");

        let dest_path = PathBuf::from(dest);
        if let Some(parent) = dest_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                error!(reason = %e, "failed to create model directory");
                if let Ok(mut g) = status.lock() {
                    *g = DownloadStatus::Failed(format!("mkdir failed: {e}"));
                }
                return;
            }
        }

        let response = match ureq::get(MODEL_URL).call() {
            Ok(r) => r,
            Err(e) => {
                error!(reason = %e, "model download request failed");
                if let Ok(mut g) = status.lock() {
                    *g = DownloadStatus::Failed(format!(
                        "HuggingFace serves a 302 redirect to a CDN — if your network blocks the CDN this will fail.\n{}",
                        e
                    ));
                }
                return;
            }
        };

        let total = response
            .header("content-length")
            .and_then(|v| v.parse::<u64>().ok());

        // Write to a temp file alongside the destination, then rename atomically.
        let tmp_path = dest_path.with_extension("tmp");
        let mut file = match std::fs::File::create(&tmp_path) {
            Ok(f) => f,
            Err(e) => {
                error!(reason = %e, "failed to create temp file");
                if let Ok(mut g) = status.lock() {
                    *g = DownloadStatus::Failed(format!("file create failed: {e}"));
                }
                return;
            }
        };

        let mut reader = response.into_reader();
        let mut buf = vec![0u8; 65536];
        let mut received: u64 = 0;

        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    use std::io::Write;
                    if let Err(e) = file.write_all(&buf[..n]) {
                        error!(reason = %e, "write error during download");
                        let _ = std::fs::remove_file(&tmp_path);
                        if let Ok(mut g) = status.lock() {
                            *g = DownloadStatus::Failed(format!("write error: {e}"));
                        }
                        return;
                    }
                    received += n as u64;
                    if let Ok(mut g) = status.lock() {
                        *g = DownloadStatus::Downloading { received, total };
                    }
                }
                Err(e) => {
                    error!(reason = %e, "read error during download");
                    let _ = std::fs::remove_file(&tmp_path);
                    if let Ok(mut g) = status.lock() {
                        *g = DownloadStatus::Failed(format!("read error: {e}"));
                    }
                    return;
                }
            }
        }

        if let Err(e) = std::fs::rename(&tmp_path, &dest_path) {
            error!(reason = %e, "failed to rename tmp file");
            let _ = std::fs::remove_file(&tmp_path);
            if let Ok(mut g) = status.lock() {
                *g = DownloadStatus::Failed(format!("rename failed: {e}"));
            }
            return;
        }

        info!(dest, bytes = received, "model download complete");
        if let Ok(mut g) = status.lock() {
            *g = DownloadStatus::Done;
        }
    }

    // ── Step: SetupUinput ────────────────────────────────────────────────────

    fn show_setup_uinput(
        ui: &mut egui::Ui,
        state: &mut FirstRunState,
        uinput: &mut UinputSetupState,
    ) {
        // Request repaint while an action is running.
        if matches!(uinput.modprobe, SetupActionStatus::Running)
            || matches!(uinput.usermod, SetupActionStatus::Running)
        {
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(200));
        }

        ui.heading("Step 3 of 4: Set up uinput access");
        ui.add_space(8.0);
        ui.label("Vibe Attack needs /dev/uinput to inject key events into your game.");
        ui.add_space(12.0);

        // --- Action 1: load module ---
        ui.label("1. Load the uinput kernel module:");
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let button_text = match &uinput.modprobe {
                SetupActionStatus::Idle => "Load module",
                SetupActionStatus::Running => "Loading…",
                SetupActionStatus::Done => "✓ Loaded",
                SetupActionStatus::Failed(_) => "Retry",
            };
            let enabled = !matches!(uinput.modprobe, SetupActionStatus::Running);
            if ui.add_enabled(enabled, egui::Button::new(button_text)).clicked() {
                uinput.modprobe = SetupActionStatus::Running;
                let handle = std::thread::spawn(|| run_pkexec(&["modprobe", "uinput"]));
                uinput.modprobe_handle = Some(handle);
            }
            if let SetupActionStatus::Failed(ref msg) = uinput.modprobe {
                ui.colored_label(egui::Color32::RED, msg.as_str());
            }
        });
        ui.add_space(4.0);
        ui.label("Optional — persist across reboots:");
        copy_command_row(
            ui,
            "echo \"uinput\" | sudo tee /etc/modules-load.d/uinput.conf",
        );
        ui.add_space(12.0);

        // --- Action 2: add to input group ---
        ui.label("2. Add yourself to the input group:");
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let button_text = match &uinput.usermod {
                SetupActionStatus::Idle => "Add to input group",
                SetupActionStatus::Running => "Running…",
                SetupActionStatus::Done => "✓ Added",
                SetupActionStatus::Failed(_) => "Retry",
            };
            let enabled = !matches!(uinput.usermod, SetupActionStatus::Running);
            if ui.add_enabled(enabled, egui::Button::new(button_text)).clicked() {
                let username = std::env::var("USER").unwrap_or_default();
                uinput.usermod = SetupActionStatus::Running;
                let handle = std::thread::spawn(move || {
                    run_pkexec(&["usermod", "-aG", "input", &username])
                });
                uinput.usermod_handle = Some(handle);
            }
            if let SetupActionStatus::Failed(ref msg) = uinput.usermod {
                ui.colored_label(egui::Color32::RED, msg.as_str());
            }
        });
        ui.add_space(8.0);

        ui.label("3. Apply the group change without logging out (run in your terminal):");
        copy_command_row(ui, "newgrp input");
        ui.add_space(8.0);

        egui::Frame::NONE
            .fill(egui::Color32::from_rgb(64, 50, 0))
            .inner_margin(egui::Margin::same(6))
            .show(ui, |ui| {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 200, 60),
                    "Note (systemd v258+ / CachyOS 2025+): use the 'input' group, not 'uinput'.",
                );
            });

        ui.add_space(12.0);
        if ui.button("Re-check").clicked() {
            *state = probe::run();
        }
    }

    /// Run a command via pkexec (polkit) and return Ok(()) on exit 0, Err(msg) otherwise.
    fn run_pkexec(args: &[&str]) -> Result<(), String> {
        let mut cmd_args = vec!["pkexec"];
        cmd_args.extend_from_slice(args);

        let status = std::process::Command::new(cmd_args[0])
            .args(&cmd_args[1..])
            .status()
            .map_err(|e| format!("failed to run pkexec: {e}"))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!(
                "command exited with code {}",
                status.code().unwrap_or(-1)
            ))
        }
    }

    /// Render a dark code block with a "Copy" button on the right.
    fn copy_command_row(ui: &mut egui::Ui, cmd: &str) {
        ui.horizontal(|ui| {
            egui::Frame::NONE
                .fill(egui::Color32::from_gray(30))
                .inner_margin(egui::Margin::same(6))
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut cmd.to_string().as_str())
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .interactive(false),
                    );
                });
            if ui.small_button("Copy").clicked() {
                ui.ctx().copy_text(cmd.to_string());
            }
        });
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
        } else if !ptt.listening && ptt.handle.is_none() && ui.button("Listen for key").clicked() {
            ptt.error = None;
            let captured = Arc::clone(&ptt.captured_key);
            let handle = std::thread::spawn(move || {
                capture_first_keypress(captured);
            });
            ptt.handle = Some(handle);
            ptt.listening = true;
        }

        if let Some(err) = &ptt.error {
            ui.add_space(8.0);
            ui.colored_label(egui::Color32::RED, err.as_str());
        }

        ui.add_space(8.0);
        ui.label("Or enter a key name manually (e.g. KEY_GRAVE, KEY_F13):");
        let resp = ui.text_edit_singleline(&mut ptt.manual_key);
        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            let key_name = ptt.manual_key.trim().to_string();
            if !key_name.is_empty() {
                match write_ptt_key_to_config(&key_name) {
                    Ok(()) => {
                        info!(key = %key_name, "PTT key written to config (manual entry)");
                        ptt.manual_key.clear();
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
                .is_some_and(|keys| keys.contains(evdev::KeyCode::KEY_A))
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
    use super::inner::{rewrite_ptt_key, PttCaptureState};

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

    #[test]
    #[cfg(feature = "gui")]
    fn manual_key_persists_in_state() {
        let mut ptt = PttCaptureState::new();
        ptt.manual_key.push_str("KEY_F13");
        assert_eq!(ptt.manual_key, "KEY_F13");
    }

    #[test]
    #[cfg(feature = "gui")]
    fn manual_key_default_empty() {
        assert!(PttCaptureState::default().manual_key.is_empty());
    }
}
