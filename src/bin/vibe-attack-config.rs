use eframe::egui;
use vibe_attack::ui::config_app::ConfigApp;
use vibe_attack::ui::first_run::FirstRunState;

struct VibeAttackConfigApp {
    first_run: FirstRunState,
    config: ConfigApp,
}

impl VibeAttackConfigApp {
    fn new() -> Self {
        Self {
            first_run: FirstRunState::from_checks(false, false, false, false),
            config: ConfigApp::new(),
        }
    }
}

impl eframe::App for VibeAttackConfigApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Vibe Attack");
            ui.separator();

            if !self.first_run.is_setup_complete() {
                ui.label("First-run setup required:");
                for step in self.first_run.steps_remaining() {
                    ui.label(format!("  • {step:?}"));
                }
            } else {
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
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Vibe Attack Config")
            .with_inner_size([600.0, 400.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Vibe Attack Config",
        options,
        Box::new(|_cc| Ok(Box::new(VibeAttackConfigApp::new()))),
    )
}
