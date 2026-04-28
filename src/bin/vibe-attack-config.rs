use std::sync::{
    atomic::{AtomicU32, Ordering},
    mpsc, Arc,
};

use eframe::egui;
use vibe_attack::control::protocol::ActivationMode;
use vibe_attack::ui::config_app::{load_config_into_app, load_profiles, save_app_to_config, ConfigApp};
use vibe_attack::ui::first_run::FirstRunState;
use vibe_attack::ui::pack_editor::{show_pack_editor, PackEditorState};
use vibe_attack::ui::probe;
use vibe_attack::ui::tray::TrayHandle;
use vibe_attack::ui::wizard::{show_wizard, ModelDownloadState, PttCaptureState, UinputSetupState};

// ── Log channel ──────────────────────────────────────────────────────────────

/// A tracing subscriber layer that forwards formatted log records to a channel.
struct ChannelLayer {
    tx: mpsc::SyncSender<String>,
}

impl<S> tracing_subscriber::Layer<S> for ChannelLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = MessageVisitor(String::new());
        event.record(&mut visitor);
        let line = format!("[{}] {}", event.metadata().level(), visitor.0);
        // Non-blocking send — drop if buffer full to avoid blocking the UI thread.
        let _ = self.tx.try_send(line);
    }
}

struct MessageVisitor(String);

impl tracing::field::Visit for MessageVisitor {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.0 = value.to_string();
        }
    }
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{value:?}");
        }
    }
}

// ── Mic level thread ─────────────────────────────────────────────────────────

/// Shared mic level state: f32 RMS stored as bits in an AtomicU32.
struct MicLevelState {
    level: Arc<AtomicU32>,
    no_device: bool,
    _handle: Option<std::thread::JoinHandle<()>>,
}

impl MicLevelState {
    fn current_level(&self) -> f32 {
        f32::from_bits(self.level.load(Ordering::Relaxed))
    }
}

fn spawn_mic_level_thread() -> MicLevelState {
    let level = Arc::new(AtomicU32::new(0u32));
    let level_clone = Arc::clone(&level);

    // no_device is signalled back from the thread via a one-shot channel.
    let (tx, rx) = std::sync::mpsc::sync_channel::<bool>(1);

    // Build and play the stream inside the thread so the !Send stream never
    // crosses a thread boundary.
    let handle = std::thread::spawn(move || {
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

        let host = cpal::default_host();
        let device = match host.default_input_device() {
            Some(d) => d,
            None => {
                tracing::warn!("Mic level: no default input device found");
                let _ = tx.send(true);
                return;
            }
        };

        let config = match device.default_input_config() {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(reason = %e, "Mic level: failed to get default input config");
                let _ = tx.send(true);
                return;
            }
        };

        tracing::info!(
            device = device.name().unwrap_or_default(),
            sample_rate = config.sample_rate().0,
            channels = config.channels(),
            "Mic level thread: using device"
        );

        let err_level = Arc::clone(&level_clone);
        let stream_result = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _| {
                if data.is_empty() {
                    return;
                }
                let rms = (data.iter().map(|s| s * s).sum::<f32>() / data.len() as f32).sqrt();
                level_clone.store(rms.to_bits(), Ordering::Relaxed);
            },
            move |e| {
                tracing::error!(reason = %e, "Mic level: stream error");
                err_level.store(0u32, Ordering::Relaxed);
            },
            None,
        );

        let stream = match stream_result {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(reason = %e, "Mic level: failed to build input stream");
                let _ = tx.send(true);
                return;
            }
        };

        if let Err(e) = stream.play() {
            tracing::error!(reason = %e, "Mic level: stream play failed");
            let _ = tx.send(true);
            return;
        }

        let _ = tx.send(false);
        // Park the thread to keep the stream alive until the process exits.
        std::thread::park();
    });

    let no_device = rx.recv().unwrap_or(true);

    MicLevelState {
        level,
        no_device,
        _handle: Some(handle),
    }
}

// ── App ───────────────────────────────────────────────────────────────────────

struct VibeAttackConfigApp {
    first_run: FirstRunState,
    config: ConfigApp,
    ptt: PttCaptureState,
    dl: ModelDownloadState,
    uinput: UinputSetupState,
    config_example_contents: &'static str,
    hd2_profile_contents: &'static str,
    mic: MicLevelState,
    log_rx: mpsc::Receiver<String>,
    setup_just_completed: bool,
    tray: Option<TrayHandle>,
    /// Cached full Config for round-trip saves; populated at startup or wizard-complete.
    cached_config: Option<vibe_attack::config::Config>,
    /// Input device names enumerated once at startup; empty list on CPAL failure.
    device_names: Vec<String>,
    /// Active pack editor state; `None` until a profile is clicked.
    pack_editor: Option<PackEditorState>,
}

impl VibeAttackConfigApp {
    fn new(log_rx: mpsc::Receiver<String>, skip_wizard: bool) -> Self {
        use cpal::traits::{DeviceTrait, HostTrait};

        let config_example_contents =
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/config.example.yaml"));
        let hd2_profile_contents =
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/profiles/hd2/pack.yaml"));

        let first_run = if skip_wizard {
            tracing::info!(skip_wizard = true, "Wizard bypass via --skip-wizard flag");
            FirstRunState::from_checks(true, true, true, true)
        } else {
            probe::run()
        };
        let setup_complete = first_run.is_setup_complete();

        let mut config = ConfigApp::new();
        let mic;

        // Enumerate input device names once at startup; degrade gracefully on failure.
        let device_names: Vec<String> = cpal::default_host()
            .input_devices()
            .map(|iter| iter.filter_map(|d| d.name().ok()).collect())
            .unwrap_or_default();

        let mut cached_config = None;

        if setup_complete {
            config.profiles = load_profiles();
            mic = spawn_mic_level_thread();

            match load_config_into_app(&mut config, None) {
                Ok(cfg) => cached_config = Some(cfg),
                Err(e) => config.set_status(format!("Could not load config.yaml: {e}")),
            }
        } else {
            // Defer mic/profile load until after wizard completes
            mic = MicLevelState {
                level: Arc::new(AtomicU32::new(0)),
                no_device: false,
                _handle: None,
            };
        }

        Self {
            first_run,
            config,
            ptt: PttCaptureState::new(),
            dl: ModelDownloadState::new(),
            uinput: UinputSetupState::new(),
            config_example_contents,
            hd2_profile_contents,
            mic,
            log_rx,
            setup_just_completed: false,
            tray: TrayHandle::spawn(),
            cached_config,
            device_names,
            pack_editor: None,
        }
    }
}

impl eframe::App for VibeAttackConfigApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Honour "Open Config" requests from the tray.
        if self.tray.as_ref().is_some_and(|t| t.take_open_request()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }

        // Repaint at ~10Hz for mic level updates when in main config view.
        if self.first_run.is_setup_complete() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }

        // Repaint faster while PTT capture thread is listening.
        if self.ptt.listening {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }

        // Repaint while model download is in progress.
        if self.dl.is_running() {
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }

        // Repaint while uinput setup actions are running.
        use vibe_attack::ui::wizard::SetupActionStatus;
        if matches!(self.uinput.modprobe, SetupActionStatus::Running)
            || matches!(self.uinput.usermod, SetupActionStatus::Running)
        {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        }

        // Drain log channel.
        while let Ok(line) = self.log_rx.try_recv() {
            self.config.add_log_line(line);
        }

        // Update mic level from atomic.
        self.config.mic_level = self.mic.current_level();
        self.config.mic_no_device = self.mic.no_device;

        // Refresh daemon-running status each frame (cheap socket stat).
        self.config.daemon_running = vibe_attack::control::client::is_daemon_running();

        // Detect wizard completion and load profiles + start mic thread.
        if self.setup_just_completed {
            self.setup_just_completed = false;
            self.config.profiles = load_profiles();
            self.mic = spawn_mic_level_thread();

            match load_config_into_app(&mut self.config, None) {
                Ok(cfg) => self.cached_config = Some(cfg),
                Err(e) => self.config.set_status(format!("Could not load config.yaml: {e}")),
            }
        }

        ui.heading("Vibe Attack");
        ui.separator();

        let was_incomplete = !self.first_run.is_setup_complete();

        if !self.first_run.is_setup_complete() {
            show_wizard(
                ui,
                &mut self.first_run,
                &mut self.ptt,
                &mut self.dl,
                &mut self.uinput,
                self.config_example_contents,
                self.hd2_profile_contents,
            );
            // Detect transition to complete
            if was_incomplete && self.first_run.is_setup_complete() {
                self.setup_just_completed = true;
            }
        } else {
            show_main_config(ui, self);
        }
    }
}

fn show_main_config(ui: &mut egui::Ui, app: &mut VibeAttackConfigApp) {
    // Daemon status row
    ui.horizontal(|ui| {
        if app.config.daemon_running {
            ui.colored_label(egui::Color32::GREEN, "● Daemon: running");
        } else {
            ui.colored_label(
                egui::Color32::from_rgb(255, 165, 0),
                "● Daemon: not running (changes will save to disk only)",
            );
        }
    });

    ui.add_space(4.0);

    // Mic level row
    ui.horizontal(|ui| {
        ui.label("Mic:");
        if app.config.mic_no_device {
            ui.label("no device");
        } else {
            ui.add(
                egui::ProgressBar::new(app.config.mic_level.clamp(0.0, 1.0))
                    .desired_width(200.0)
                    .show_percentage(),
            );
        }
    });

    ui.add_space(4.0);

    // Activation mode
    ui.horizontal(|ui| {
        ui.label("Mode:");
        ui.radio_value(&mut app.config.mode, ActivationMode::Ptt, "Push-to-talk");
        ui.radio_value(&mut app.config.mode, ActivationMode::Wake, "Wake word");
    });

    // Confidence threshold slider
    ui.add(
        egui::Slider::new(&mut app.config.threshold_pct, 0u8..=100u8)
            .text("Confidence threshold (%)"),
    );

    // Input device combo box
    let current_device_label = app
        .config
        .input_device
        .as_deref()
        .unwrap_or("<system default>")
        .to_string();
    egui::ComboBox::from_label("Input device")
        .selected_text(current_device_label)
        .show_ui(ui, |ui| {
            // Leading option: system default (maps to None)
            let is_default = app.config.input_device.is_none();
            if ui.selectable_label(is_default, "<system default>").clicked() {
                app.config.input_device = None;
            }
            for name in &app.device_names.clone() {
                let is_selected = app.config.input_device.as_deref() == Some(name.as_str());
                if ui.selectable_label(is_selected, name.as_str()).clicked() {
                    app.config.input_device = Some(name.clone());
                }
            }
        });

    // PTT binding (read-only display — live capture deferred to post-S01)
    ui.horizontal(|ui| {
        ui.label(format!("PTT key: {}", app.config.ptt_binding));
    });

    // Save button
    if ui.button("Save").clicked() {
        handle_save(app);
    }

    // Status message
    if let Some(msg) = &app.config.status_message.clone() {
        ui.label(msg);
    }

    ui.add_space(8.0);

    // Profiles list
    ui.label(format!("Profiles ({})", app.config.profile_count()));
    if app.config.profiles.is_empty() {
        ui.weak("No profiles found in ~/.config/vibe-attack/profiles/");
    } else {
        for name in &app.config.profiles.clone() {
            let is_active = app.config.active_profile.as_deref() == Some(name.as_str());
            let is_editing = app
                .pack_editor
                .as_ref()
                .map(|s| s.editor.pack().name == *name)
                .unwrap_or(false);
            if ui.selectable_label(is_active || is_editing, name.as_str()).clicked() {
                match vibe_attack::pack::get_profiles_dir() {
                    Ok(profiles_dir) => {
                        let profile_dir = profiles_dir.join(name);
                        match vibe_attack::pack::Pack::load_from_dir(&profile_dir) {
                            Ok(pack) => {
                                let editor = vibe_attack::pack::PackEditor::new(pack);
                                app.pack_editor =
                                    Some(PackEditorState::new(editor, profile_dir));
                            }
                            Err(e) => {
                                tracing::warn!(profile = %name, reason = %e, "Failed to load pack for editor");
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!(reason = %e, "Failed to resolve profiles dir for pack editor");
                    }
                }
            }
        }
    }

    if let Some(editor_state) = app.pack_editor.as_mut() {
        ui.add_space(8.0);
        ui.separator();
        show_pack_editor(ui, editor_state, app.config.daemon_running);

        // Drain imported_pack_name: refresh the profiles list so the newly imported
        // pack appears immediately. The editor itself was already swapped inside the
        // import handler, so no further editor update is needed here.
        if editor_state.imported_pack_name.take().is_some() {
            app.config.profiles = load_profiles();
        }
    }

    ui.separator();
    ui.label("Log:");
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for line in &app.config.log_lines.clone() {
                ui.monospace(line.as_str());
            }
        });
}

fn handle_save(app: &mut VibeAttackConfigApp) {
    use vibe_attack::control::client::send_command;
    use vibe_attack::control::protocol::ControlRequest;

    let Some(cached) = app.cached_config.as_ref() else {
        app.config.set_status("No config loaded — cannot save.");
        return;
    };

    match save_app_to_config(&app.config, cached, None) {
        Err(e) => {
            app.config.set_status(format!("Save failed: {e}"));
            return;
        }
        Ok(updated) => {
            app.cached_config = Some(updated);
        }
    }

    if !app.config.daemon_running {
        app.config
            .set_status("Saved to disk — daemon not running, runtime changes skipped.");
        return;
    }

    // Dispatch four control commands; collect any failures.
    let commands = [
        ControlRequest::SetMode { mode: app.config.mode.clone() },
        ControlRequest::SetThreshold {
            threshold: app.config.threshold_pct as f32 / 100.0,
        },
        ControlRequest::SetInputDevice {
            device: app.config.input_device.clone(),
        },
        ControlRequest::SetPttBinding {
            key: app.config.ptt_binding.clone(),
        },
    ];

    let mut errors: Vec<String> = Vec::new();
    for cmd in commands {
        let label = format!("{cmd:?}");
        if let Err(e) = send_command(cmd) {
            errors.push(format!("Failed to send {label}: {e}"));
        }
    }

    if errors.is_empty() {
        app.config.set_status("Saved and applied.");
    } else {
        app.config.set_status(errors.join("; "));
    }
}

fn main() -> eframe::Result<()> {
    use tracing_subscriber::prelude::*;

    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("Usage: vibe-attack-config [--skip-wizard] [--help]");
        println!();
        println!("  --skip-wizard   Bypass first-run wizard regardless of probe state");
        println!("  --help, -h      Print this help and exit");
        return Ok(());
    }

    let skip_wizard = args.iter().any(|a| a == "--skip-wizard");

    // Log channel: bounded to 500 entries; oldest are dropped under pressure.
    let (log_tx, log_rx) = mpsc::sync_channel(500);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(ChannelLayer { tx: log_tx })
        .init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Vibe Attack Config")
            .with_inner_size([720.0, 520.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Vibe Attack Config",
        options,
        Box::new(move |_cc| Ok(Box::new(VibeAttackConfigApp::new(log_rx, skip_wizard)))),
    )
}
