/// Pure-logic state and profile loading helpers for the config window.
pub mod config_app;
/// Setup wizard state machine and step enum (feature-gate-free).
pub mod first_run;
/// Environment probe helpers that check for config file, model, and uinput readiness.
pub mod probe;
/// System-tray icon and menu via D-Bus (ksni); spawns its own tokio thread.
#[cfg(feature = "gui")]
pub mod tray;
/// First-run wizard panels rendered via egui (gui feature only).
pub mod wizard;
/// Egui pack editor panel and pure-logic key-sequence parser.
pub mod pack_editor;
