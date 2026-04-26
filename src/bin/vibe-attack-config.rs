use std::sync::{
    atomic::{AtomicU32, Ordering},
    mpsc, Arc,
};

use eframe::egui;
use vibe_attack::ui::config_app::{load_profiles, ConfigApp};
use vibe_attack::ui::first_run::FirstRunState;
use vibe_attack::ui::probe;
use vibe_attack::ui::wizard::{show_wizard, ModelDownloadState, PttCaptureState};

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
    config_example_contents: &'static str,
    hd2_profile_contents: &'static str,
    mic: MicLevelState,
    log_rx: mpsc::Receiver<String>,
    setup_just_completed: bool,
}

impl VibeAttackConfigApp {
    fn new(log_rx: mpsc::Receiver<String>) -> Self {
        let config_example_contents =
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/config.example.yaml"));
        let hd2_profile_contents =
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/profiles/hd2/pack.yaml"));

        let first_run = probe::run();
        let setup_complete = first_run.is_setup_complete();

        let mut config = ConfigApp::new();
        let mic;

        if setup_complete {
            config.profiles = load_profiles();
            mic = spawn_mic_level_thread();
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
            config_example_contents,
            hd2_profile_contents,
            mic,
            log_rx,
            setup_just_completed: false,
        }
    }
}

impl eframe::App for VibeAttackConfigApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

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

        // Drain log channel.
        while let Ok(line) = self.log_rx.try_recv() {
            self.config.add_log_line(line);
        }

        // Update mic level from atomic.
        self.config.mic_level = self.mic.current_level();
        self.config.mic_no_device = self.mic.no_device;

        // Detect wizard completion and load profiles + start mic thread.
        if self.setup_just_completed {
            self.setup_just_completed = false;
            self.config.profiles = load_profiles();
            self.mic = spawn_mic_level_thread();
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
                self.config_example_contents,
                self.hd2_profile_contents,
            );
            // Detect transition to complete
            if was_incomplete && self.first_run.is_setup_complete() {
                self.setup_just_completed = true;
            }
        } else {
            show_main_config(ui, &self.config);
        }
    }
}

fn show_main_config(ui: &mut egui::Ui, config: &ConfigApp) {
    // Mic level
    ui.horizontal(|ui| {
        ui.label("Mic:");
        if config.mic_no_device {
            ui.label("no device");
        } else {
            ui.add(
                egui::ProgressBar::new(config.mic_level.clamp(0.0, 1.0))
                    .desired_width(200.0)
                    .show_percentage(),
            );
        }
    });

    ui.add_space(8.0);

    // Profiles
    ui.label(format!("Profiles ({})", config.profile_count()));
    if config.profiles.is_empty() {
        ui.weak("No profiles found in ~/.config/vibe-attack/profiles/");
    } else {
        for name in &config.profiles {
            let is_active = config.active_profile.as_deref() == Some(name.as_str());
            let _ = ui.selectable_label(is_active, name.as_str());
        }
    }

    ui.separator();
    ui.label("Log:");
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for line in &config.log_lines {
                ui.monospace(line.as_str());
            }
        });
}

fn main() -> eframe::Result<()> {
    use tracing_subscriber::prelude::*;

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
        Box::new(|_cc| Ok(Box::new(VibeAttackConfigApp::new(log_rx)))),
    )
}
