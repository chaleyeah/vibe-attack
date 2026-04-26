use std::path::PathBuf;

use eframe::egui;
use vibe_attack::ui::config_app::ConfigApp;
use vibe_attack::ui::first_run::FirstRunState;
use vibe_attack::ui::probe;
use vibe_attack::ui::wizard::{show_wizard, PttCaptureState};

struct VibeAttackConfigApp {
    first_run: FirstRunState,
    config: ConfigApp,
    ptt: PttCaptureState,
    config_example_path: PathBuf,
}

impl VibeAttackConfigApp {
    fn new() -> Self {
        // Locate config.example.yaml relative to the binary's directory.
        // Falls back to the current directory if the exe path is unavailable.
        let config_example_path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."))
            .join("config.example.yaml");

        Self {
            first_run: probe::run(),
            config: ConfigApp::new(),
            ptt: PttCaptureState::new(),
            config_example_path,
        }
    }
}

impl eframe::App for VibeAttackConfigApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request continuous repaints while the PTT capture thread is running
        // so we pick up the result promptly without blocking the UI.
        if self.ptt.listening {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Vibe Attack");
            ui.separator();

            if !self.first_run.is_setup_complete() {
                show_wizard(
                    ui,
                    &mut self.first_run,
                    &mut self.ptt,
                    &self.config_example_path,
                );
            } else {
                // Main config app view (populated further in S04)
                ui.label(format!("Profiles: {}", self.config.profile_count()));
                if let Some(active) = &self.config.active_profile {
                    ui.label(format!("Active: {active}"));
                }
                ui.separator();
                ui.label("Log:");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for line in &self.config.log_lines {
                        ui.label(line);
                    }
                });
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Vibe Attack Config")
            .with_inner_size([680.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Vibe Attack Config",
        options,
        Box::new(|_cc| Ok(Box::new(VibeAttackConfigApp::new()))),
    )
}
