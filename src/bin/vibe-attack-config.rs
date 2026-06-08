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
use vibe_attack::ui::widgets::{DaemonStatus, NavId, NAV_ITEMS};
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

/// Tracks the state of a daemon start attempt so the UI can show progress and errors.
enum DaemonStartState {
    /// No start attempt in progress.
    Idle,
    /// Daemon was spawned; waiting for socket to appear. Holds the deadline and stderr reader.
    Starting {
        deadline: std::time::Instant,
        stderr_thread: Option<std::thread::JoinHandle<String>>,
    },
    /// Start attempt failed; holds the error message for display.
    Failed(String),
}

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
    /// Tracks daemon start attempts so the UI can report progress/failure.
    daemon_start: DaemonStartState,
    /// Active navigation pane in the main config window.
    active_nav: NavId,
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
            daemon_start: DaemonStartState::Idle,
            active_nav: NavId::Devices,
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

        // Honour "Quit" requests from the tray.
        if self.tray.as_ref().is_some_and(|t| t.take_quit_request()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
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

fn daemon_status(app: &VibeAttackConfigApp) -> DaemonStatus {
    if app.config.daemon_running {
        DaemonStatus::Running
    } else {
        match &app.daemon_start {
            DaemonStartState::Starting { .. } => DaemonStatus::Disconnected,
            DaemonStartState::Failed(_)       => DaemonStatus::Error,
            DaemonStartState::Idle            => DaemonStatus::Disconnected,
        }
    }
}

fn show_main_config(ui: &mut egui::Ui, app: &mut VibeAttackConfigApp) {
    use vibe_attack::ui::widgets::{
        app_header, banner, side_nav, status_footer, BannerKind,
    };
    use vibe_attack::ui::theme::Palette;

    // Poll daemon start state machine.
    if let DaemonStartState::Starting { deadline, .. } = &app.daemon_start {
        if app.config.daemon_running {
            app.daemon_start = DaemonStartState::Idle;
            app.config.set_status("Daemon started.");
        } else if std::time::Instant::now() > *deadline {
            let stderr_output = if let DaemonStartState::Starting { stderr_thread, .. } = &mut app.daemon_start {
                stderr_thread.take().and_then(|h| h.join().ok()).unwrap_or_default()
            } else {
                String::new()
            };
            let msg = if stderr_output.trim().is_empty() {
                "Daemon failed to start (no output — check permissions and config)".to_string()
            } else {
                let trimmed = stderr_output.trim();
                let last_lines: String = trimmed.lines().rev().take(3).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join(" | ");
                format!("Daemon failed to start: {last_lines}")
            };
            app.daemon_start = DaemonStartState::Failed(msg);
        }
    }

    let status = daemon_status(app);
    let time   = ui.ctx().input(|i| i.time);
    let version = env!("CARGO_PKG_VERSION");

    // ── Layout: header / (rail + body) / footer ───────────────────────────────
    let total_rect = ui.available_rect_before_wrap();
    let footer_h: f32 = 30.0;
    let header_h: f32 = 44.0;
    let body_h = (total_rect.height() - header_h - footer_h).max(0.0);

    // Header
    let header_rect = egui::Rect::from_min_size(total_rect.min, egui::vec2(total_rect.width(), header_h));
    let mut header_ui = ui.new_child(egui::UiBuilder::new().max_rect(header_rect));
    app_header(&mut header_ui, version, status, time);

    // Body: side nav + pane
    let body_top = total_rect.min.y + header_h;
    let body_rect = egui::Rect::from_min_size(
        egui::pos2(total_rect.min.x, body_top),
        egui::vec2(total_rect.width(), body_h),
    );

    let rail_w: f32 = 52.0;
    let rail_rect = egui::Rect::from_min_size(body_rect.min, egui::vec2(rail_w, body_h));
    let pane_rect = egui::Rect::from_min_size(
        egui::pos2(body_rect.min.x + rail_w, body_top),
        egui::vec2(body_rect.width() - rail_w, body_h),
    );

    // Side nav
    let mut rail_ui = ui.new_child(egui::UiBuilder::new().max_rect(rail_rect));
    side_nav(&mut rail_ui, NAV_ITEMS, &mut app.active_nav);

    // Pane content
    let mut pane_ui = ui.new_child(egui::UiBuilder::new().max_rect(pane_rect));
    pane_ui.painter().rect_filled(pane_rect, 0.0, Palette::BG_WINDOW);

    egui::ScrollArea::vertical()
        .id_salt("pane_scroll")
        .show(&mut pane_ui, |ui| {
            ui.set_min_width(pane_rect.width());

            // Daemon error banner at top of pane
            if matches!(status, DaemonStatus::Error | DaemonStatus::Disconnected) {
                let (banner_title, banner_body) = match &app.daemon_start {
                    DaemonStartState::Failed(msg) => ("DAEMON ERROR".to_string(), msg.clone()),
                    DaemonStartState::Starting { .. } => ("DAEMON STARTING".to_string(), "Waiting for the daemon to become ready…".to_string()),
                    DaemonStartState::Idle => ("DAEMON OFFLINE".to_string(), "The Vibe Attack daemon is not running. Start it to apply changes at runtime.".to_string()),
                };

                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    ui.add_space(24.0);
                    ui.vertical(|ui| {
                        let action = banner(
                            ui,
                            BannerKind::Error,
                            &banner_title,
                            &banner_body,
                            &[("RECONNECT", true), ("START DAEMON", false)],
                        );
                        match action {
                            Some(0) => {
                                // Reconnect: just re-check (daemon_running is polled each frame)
                            }
                            Some(1) => {
                                match start_daemon() {
                                    Ok(stderr_thread) => {
                                        app.daemon_start = DaemonStartState::Starting {
                                            deadline: std::time::Instant::now() + std::time::Duration::from_secs(15),
                                            stderr_thread: Some(stderr_thread),
                                        };
                                    }
                                    Err(e) => {
                                        app.daemon_start = DaemonStartState::Failed(format!("Failed to start daemon: {e}"));
                                    }
                                }
                            }
                            _ => {}
                        }
                    });
                    ui.add_space(24.0);
                });
            }

            // Running: show stop-daemon button in a subtle row
            if app.config.daemon_running {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.add_space(24.0);
                    if ui.small_button("Stop Daemon").clicked() {
                        match vibe_attack::control::client::send_command(
                            vibe_attack::control::protocol::ControlRequest::Shutdown,
                        ) {
                            Ok(_) => app.config.set_status("Daemon stopping…"),
                            Err(e) => app.config.set_status(format!("Failed to stop daemon: {e}")),
                        }
                    }
                });
            }

            ui.add_space(20.0);

            match app.active_nav {
                NavId::Devices  => pane_devices(ui, app),
                NavId::Voice    => pane_voice(ui, app),
                NavId::Packs    => pane_packs(ui, app),
                NavId::Hotkeys  => pane_hotkeys(ui, app),
                NavId::Advanced => pane_advanced(ui, app),
            }

            // Drain imported pack name
            if let Some(editor_state) = app.pack_editor.as_mut() {
                if editor_state.imported_pack_name.take().is_some() {
                    app.config.profiles = load_profiles();
                }
            }
        });

    // Footer
    let footer_rect = egui::Rect::from_min_size(
        egui::pos2(total_rect.min.x, total_rect.max.y - footer_h),
        egui::vec2(total_rect.width(), footer_h),
    );
    let mut footer_ui = ui.new_child(egui::UiBuilder::new().max_rect(footer_rect));
    let model_name = "tiny.en";
    status_footer(&mut footer_ui, status, app.config.mic_level, model_name, None);
}

// ── Devices pane ──────────────────────────────────────────────────────────────

fn pane_devices(ui: &mut egui::Ui, app: &mut VibeAttackConfigApp) {
    use vibe_attack::ui::widgets::{field_row, led_meter, section_header};
    use vibe_attack::ui::theme::Palette;

    ui.horizontal(|ui| { ui.add_space(24.0); ui.vertical(|ui| {

    section_header(ui, "INPUT DEVICE", None);
    ui.add_space(12.0);

    // Device select
    field_row(ui, "Input Device", false, |ui| {
        let current = app.config.input_device.as_deref().unwrap_or("<system default>").to_string();
        egui::ComboBox::from_id_salt("device_combo")
            .selected_text(&current)
            .width(260.0)
            .show_ui(ui, |ui| {
                let is_default = app.config.input_device.is_none();
                if ui.selectable_label(is_default, "<system default>").clicked() {
                    app.config.input_device = None;
                }
                for name in &app.device_names.clone() {
                    let selected = app.config.input_device.as_deref() == Some(name.as_str());
                    if ui.selectable_label(selected, name.as_str()).clicked() {
                        app.config.input_device = Some(name.clone());
                    }
                }
            });
    });

    ui.add_space(8.0);

    // Live mic monitor
    field_row(ui, "Mic Level", false, |ui| {
        if app.config.mic_no_device {
            ui.label(egui::RichText::new("No device").color(Palette::FG_MUTED).size(12.0));
        } else {
            led_meter(ui, app.config.mic_level, 40);
            ui.add_space(8.0);
            let db = if app.config.mic_level > 0.0 {
                format!("{:.0} dB", 20.0 * app.config.mic_level.log10())
            } else {
                "—".to_string()
            };
            ui.label(egui::RichText::new(db).color(Palette::FG_MUTED).size(11.0));
        }
    });

    ui.add_space(20.0);
    section_header(ui, "VAD SETTINGS", None);
    ui.add_space(12.0);

    field_row(ui, "Sensitivity", false, |ui| {
        ui.add(egui::Slider::new(&mut app.config.threshold_pct, 0u8..=100u8).suffix("%"));
    });

    ui.add_space(20.0);

    // Save
    ui.horizontal(|ui| {
        use vibe_attack::ui::widgets::primary_button;
        if primary_button(ui, "Save Changes").clicked() {
            handle_save(app);
        }
        if let Some(msg) = &app.config.status_message.clone() {
            ui.add_space(12.0);
            ui.label(egui::RichText::new(msg).color(Palette::FG_MUTED).size(12.0));
        }
    });

    }); });
}

// ── Voice pane ────────────────────────────────────────────────────────────────

fn pane_voice(ui: &mut egui::Ui, app: &mut VibeAttackConfigApp) {
    use vibe_attack::ui::widgets::{field_row, kbd, section_header};
    use vibe_attack::ui::theme::Palette;

    ui.horizontal(|ui| { ui.add_space(24.0); ui.vertical(|ui| {

    section_header(ui, "TRIGGER MODE", None);
    ui.add_space(12.0);

    field_row(ui, "Mode", true, |ui| {
        ui.radio_value(&mut app.config.mode, ActivationMode::Ptt, "Push-to-talk");
        ui.add_space(16.0);
        ui.radio_value(&mut app.config.mode, ActivationMode::Wake, "Wake word");
    });

    if app.config.mode == ActivationMode::Ptt {
        ui.add_space(8.0);
        field_row(ui, "PTT Key", true, |ui| {
            kbd(ui, &app.config.ptt_binding);
            ui.add_space(8.0);
            ui.label(egui::RichText::new("(set in wizard)").color(Palette::FG_FAINT).size(11.0));
        });
    }

    ui.add_space(20.0);
    section_header(ui, "STT MODEL", None);
    ui.add_space(12.0);

    field_row(ui, "Confidence", false, |ui| {
        ui.add(egui::Slider::new(&mut app.config.threshold_pct, 0u8..=100u8).suffix("%"));
    });

    ui.add_space(20.0);

    ui.horizontal(|ui| {
        use vibe_attack::ui::widgets::primary_button;
        if primary_button(ui, "Save Changes").clicked() {
            handle_save(app);
        }
        if let Some(msg) = &app.config.status_message.clone() {
            ui.add_space(12.0);
            ui.label(egui::RichText::new(msg).color(Palette::FG_MUTED).size(12.0));
        }
    });

    }); });
}

// ── Packs pane ────────────────────────────────────────────────────────────────

fn pane_packs(ui: &mut egui::Ui, app: &mut VibeAttackConfigApp) {
    use vibe_attack::ui::widgets::section_header;
    use vibe_attack::ui::theme::Palette;

    ui.horizontal(|ui| { ui.add_space(24.0); ui.vertical(|ui| {

    section_header(ui, "PROFILES", Some(&format!("{} loaded", app.config.profile_count())));
    ui.add_space(12.0);

    if app.config.profiles.is_empty() {
        ui.label(
            egui::RichText::new("No profiles found in ~/.config/vibe-attack/profiles/")
                .color(Palette::FG_MUTED)
                .size(12.0),
        );
    } else {
        // Profile cards in a 2-column grid
        let card_w = 200.0;
        egui::Grid::new("profile_cards")
            .spacing(egui::vec2(12.0, 12.0))
            .show(ui, |ui| {
                for (i, name) in app.config.profiles.clone().iter().enumerate() {
                    let is_active = app.config.active_profile.as_deref() == Some(name.as_str());
                    let is_editing = app.pack_editor.as_ref()
                        .map(|s| s.editor.pack().name == *name)
                        .unwrap_or(false);

                    let (card_rect, resp) = ui.allocate_exact_size(
                        egui::vec2(card_w, 72.0),
                        egui::Sense::click(),
                    );

                    let bg = if is_active || is_editing { Palette::accent_faint() } else { Palette::BG_RAISED };
                    let bdr = if is_active || is_editing { Palette::accent_line() } else { Palette::STROKE };
                    let p = ui.painter();
                    p.rect_filled(card_rect, egui::CornerRadius::same(4), bg);
                    p.rect_stroke(card_rect, egui::CornerRadius::same(4), egui::Stroke::new(1.0, bdr), egui::StrokeKind::Inside);

                    // Profile icon
                    let icon_rect = egui::Rect::from_min_size(
                        egui::pos2(card_rect.left() + 12.0, card_rect.top() + 12.0),
                        egui::vec2(24.0, 24.0),
                    );
                    p.rect_filled(icon_rect, egui::CornerRadius::same(3), Palette::BG_WINDOW);
                    p.text(icon_rect.center(), egui::Align2::CENTER_CENTER, "📦", egui::FontId::proportional(14.0), Palette::ACCENT);

                    // Name
                    p.text(
                        egui::pos2(card_rect.left() + 44.0, card_rect.top() + 18.0),
                        egui::Align2::LEFT_CENTER,
                        name.as_str(),
                        egui::FontId::proportional(13.0),
                        Palette::FG_STRONG,
                    );

                    if is_active {
                        // ACTIVE tag
                        let tag_rect = egui::Rect::from_min_size(
                            egui::pos2(card_rect.right() - 52.0, card_rect.top() + 8.0),
                            egui::vec2(44.0, 16.0),
                        );
                        p.rect_filled(tag_rect, egui::CornerRadius::same(2), Palette::accent_faint());
                        p.rect_stroke(tag_rect, egui::CornerRadius::same(2), egui::Stroke::new(1.0, Palette::accent_line()), egui::StrokeKind::Inside);
                        p.text(tag_rect.center(), egui::Align2::CENTER_CENTER, "ACTIVE", egui::FontId::proportional(9.0), Palette::ACCENT);
                    }

                    // Edit button area
                    let edit_y = card_rect.bottom() - 20.0;
                    let mut edit_ui = ui.new_child(egui::UiBuilder::new().max_rect(
                        egui::Rect::from_min_size(
                            egui::pos2(card_rect.left() + 12.0, edit_y - 4.0),
                            egui::vec2(card_w - 24.0, 20.0),
                        )
                    ));
                    if edit_ui.small_button("Open Editor").clicked() || resp.clicked() {
                        open_pack_editor(app, name);
                    }

                    if (i + 1) % 2 == 0 {
                        ui.end_row();
                    }
                }
            });
    }

    // Show pack editor inline below the cards if one is open
    if let Some(editor_state) = app.pack_editor.as_mut() {
        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);
        show_pack_editor(ui, editor_state, app.config.daemon_running);
    }

    }); });
}

fn open_pack_editor(app: &mut VibeAttackConfigApp, name: &str) {
    match vibe_attack::pack::get_profiles_dir() {
        Ok(profiles_dir) => {
            let profile_dir = profiles_dir.join(name);
            match vibe_attack::pack::Pack::load_from_dir(&profile_dir) {
                Ok(pack) => {
                    let editor = vibe_attack::pack::PackEditor::new(pack);
                    app.pack_editor = Some(PackEditorState::new(editor, profile_dir));
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

// ── Hotkeys pane ──────────────────────────────────────────────────────────────

fn pane_hotkeys(ui: &mut egui::Ui, app: &mut VibeAttackConfigApp) {
    use vibe_attack::ui::widgets::{field_row, kbd, section_header};
    use vibe_attack::ui::theme::Palette;

    ui.horizontal(|ui| { ui.add_space(24.0); ui.vertical(|ui| {

    section_header(ui, "KEY BINDINGS", None);
    ui.add_space(12.0);

    let bindings: &[(&str, &str)] = &[
        ("PTT Key",         &app.config.ptt_binding.clone()),
        ("Mute",            "—"),
        ("Cycle Profile",   "—"),
        ("Pause Daemon",    "—"),
        ("Open Config",     "—"),
    ];

    for (label, key) in bindings {
        field_row(ui, label, false, |ui| {
            if *key == "—" {
                ui.label(egui::RichText::new("—").color(Palette::FG_FAINT).size(12.0));
            } else {
                kbd(ui, key);
            }
            ui.add_space(8.0);
            let _ = ui.small_button("Rebind");
        });
        ui.add_space(4.0);
    }

    }); });
}

// ── Advanced pane ─────────────────────────────────────────────────────────────

fn pane_advanced(ui: &mut egui::Ui, _app: &mut VibeAttackConfigApp) {
    use vibe_attack::ui::widgets::{banner, field_row, section_header, BannerKind};
    use vibe_attack::ui::theme::Palette;

    ui.horizontal(|ui| { ui.add_space(24.0); ui.vertical(|ui| {

    section_header(ui, "STARTUP", None);
    ui.add_space(12.0);

    field_row(ui, "Autostart", false, |ui| {
        ui.label(egui::RichText::new("(via systemd unit — configure manually)").color(Palette::FG_FAINT).size(11.0));
    });

    field_row(ui, "Socket Path", false, |ui| {
        let sock_path = std::env::var("XDG_RUNTIME_DIR")
            .map(|r| format!("{r}/vibe-attack.sock"))
            .unwrap_or_else(|_| "/run/user/<UID>/vibe-attack.sock".to_string());
        ui.label(egui::RichText::new(sock_path).color(Palette::FG_MUTED).size(12.0));
    });

    ui.add_space(20.0);
    section_header(ui, "DANGER ZONE", None);
    ui.add_space(12.0);

    banner(
        ui,
        BannerKind::Warn,
        "DESTRUCTIVE ACTIONS",
        "These actions cannot be undone. Wipe resets all configuration to factory defaults.",
        &[],
    );
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        let reset_btn = egui::Button::new(
            egui::RichText::new("Reset Config").color(Palette::ERR).size(12.0),
        )
        .fill(Palette::err_faint())
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0xe8, 0x5d, 0x3c, 77)));
        ui.add(reset_btn);
    });

    }); });
}

/// Locate and spawn the daemon (`vibe-attack`) detached from this process.
///
/// Captures stderr on a background thread so startup failures can be surfaced
/// in the UI. Returns a join handle that collects stderr output.
fn start_daemon() -> Result<std::thread::JoinHandle<String>, String> {
    let daemon_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("vibe-attack")))
        .filter(|p| p.exists());

    let daemon_path = match daemon_path {
        Some(p) => p,
        None => {
            which_vibe_attack().ok_or("vibe-attack binary not found next to config or in PATH")?
        }
    };

    let mut child = std::process::Command::new(&daemon_path)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start daemon {}: {e}", daemon_path.display()))?;

    let stderr = child.stderr.take();
    let handle = std::thread::spawn(move || {
        let mut output = String::new();
        if let Some(mut pipe) = stderr {
            use std::io::Read;
            let _ = pipe.read_to_string(&mut output);
        }
        output
    });

    Ok(handle)
}

fn which_vibe_attack() -> Option<std::path::PathBuf> {
    std::env::var_os("PATH").and_then(|path| {
        std::env::split_paths(&path)
            .map(|d| d.join("vibe-attack"))
            .find(|p| p.exists())
    })
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

fn load_icon() -> Option<egui::IconData> {
    let icon_bytes = include_bytes!("../../assets/vibe-attack.png");
    let image = image::load_from_memory(icon_bytes).ok()?;
    let image = image.into_rgba8();
    let (width, height) = image.dimensions();
    Some(egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    })
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

    let stderr_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));

    let channel_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(stderr_filter))
        .with(ChannelLayer { tx: log_tx }.with_filter(channel_filter))
        .init();

    let icon = load_icon();
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Vibe Attack Config")
        .with_inner_size([720.0, 520.0]);
    if let Some(icon) = icon {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Vibe Attack Config",
        options,
        Box::new(move |cc| {
            vibe_attack::ui::theme::apply_theme(&cc.egui_ctx);
            Ok(Box::new(VibeAttackConfigApp::new(log_rx, skip_wizard)))
        }),
    )
}

