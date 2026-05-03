/// Pack editor panel for browsing and editing macro packs.
///
/// All egui code is gated to `gui` — the default build (no eframe) only compiles
/// the pure-logic helpers (`parse_key_sequence`, `build_macro_config_from_form`)
/// exposed by the tests below.

#[cfg(feature = "gui")]
pub use inner::*;

#[cfg(feature = "gui")]
mod inner {
    use std::path::PathBuf;
    use std::time::{Duration, Instant};

    use eframe::egui;
    use rfd::FileDialog;
    use tracing;

    use crate::config::MacroConfig;
    use crate::control::client::send_command;
    use crate::control::protocol::{ControlRequest, ControlResponse};
    use crate::pack::{get_profiles_dir, Pack, PackEditor};

    use super::{build_macro_config_from_form, parse_key_sequence};

    /// All mutable UI state for the pack editor panel.
    pub struct PackEditorState {
        pub editor: PackEditor,
        pub profile_dir: PathBuf,
        pub selected_category: Option<String>,
        pub selected_macro: Option<String>,
        /// Form input: macro name.
        pub form_name: String,
        /// Form input: trigger phrase.
        pub form_phrase: String,
        /// Form input: if_flag gate condition.
        pub form_if_flag: String,
        /// Form input: set_flag side effect.
        pub form_set_flag: String,
        /// Form input: comma-separated key sequence.
        pub form_keys: String,
        /// True while the category rename confirmation UI is visible.
        pub show_rename_warning: bool,
        /// Staging field for the new category name in Add Category toolbar.
        pub form_new_category: String,
        /// True after the first "Remove Macro" click — shows confirm button.
        pub pending_remove_macro: bool,
        /// Last synchronous error to surface inline in the panel.
        pub last_error: Option<String>,
        /// Set to the imported pack name after a successful Import Pack; caller drains each frame.
        pub imported_pack_name: Option<String>,
        // 1-second confirmation guards against accidental fire — never use thread::sleep.
        pub pending_test: Option<(String, Instant)>,
        /// Inline result of the most recent Test action (success or error message).
        pub last_test_status: Option<String>,
    }

    impl PackEditorState {
        pub fn new(editor: PackEditor, profile_dir: PathBuf) -> Self {
            Self {
                editor,
                profile_dir,
                selected_category: None,
                selected_macro: None,
                form_name: String::new(),
                form_phrase: String::new(),
                form_if_flag: String::new(),
                form_set_flag: String::new(),
                form_keys: String::new(),
                show_rename_warning: false,
                form_new_category: String::new(),
                pending_remove_macro: false,
                last_error: None,
                imported_pack_name: None,
                pending_test: None,
                last_test_status: None,
            }
        }

        /// Save the pack to disk then notify the daemon via SwitchProfile.
        ///
        /// Returns Ok if the disk write succeeded. Daemon notification failure is
        /// logged as a warning but does not make the overall save fail.
        pub fn save(&self) -> anyhow::Result<()> {
            let pack = self.editor.pack();
            pack.save_to_dir(&self.profile_dir)?;

            let macro_count = pack.categories.iter().map(|c| c.macros.len()).sum::<usize>();
            let path = self.profile_dir.display().to_string();
            tracing::info!(path, macro_count, "pack saved");

            let name = pack.name.clone();
            if let Err(e) = send_command(ControlRequest::SwitchProfile { name }) {
                let reason = e.to_string();
                tracing::warn!(reason, "SwitchProfile dispatch failed");
            }

            Ok(())
        }
    }

    /// Render the full pack editor panel.
    pub fn show_pack_editor(ui: &mut egui::Ui, state: &mut PackEditorState, daemon_running: bool) {
        use crate::ui::theme::Palette;
        use crate::ui::widgets::{banner, primary_button, section_header, BannerKind};

        // ── Pack identity subheader ───────────────────────────────────────────
        let pack_name = state.editor.pack().name.clone();
        let macro_count: usize = state.editor.pack().categories.iter().map(|c| c.macros.len()).sum();

        egui::Frame::new()
            .fill(Palette::BG_PANEL)
            .stroke(egui::Stroke::new(1.0, Palette::STROKE_FAINT))
            .inner_margin(egui::Margin::symmetric(16, 10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("EDITING PACK").color(Palette::FG_FAINT).size(10.0));
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(&pack_name).color(Palette::FG_STRONG).size(13.0).strong());
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(format!("{macro_count} macros")).color(Palette::FG_MUTED).size(11.0));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if primary_button(ui, "Save Pack").clicked() {
                            match state.save() {
                                Ok(()) => state.last_error = None,
                                Err(e) => state.last_error = Some(e.to_string()),
                            }
                        }
                        ui.add_space(8.0);
                        if ui.button("Export").clicked() {
                            let default_filename = format!("{pack_name}.hdpack");
                            if let Some(dest_path) = FileDialog::new()
                                .add_filter("Pack", &["hdpack"])
                                .set_file_name(&default_filename)
                                .save_file()
                            {
                                match state.editor.pack().export(&state.profile_dir, &dest_path) {
                                    Ok(()) => state.last_error = None,
                                    Err(e) => state.last_error = Some(e.to_string()),
                                }
                            }
                        }
                        ui.add_space(8.0);
                        if ui.button("Import").clicked() {
                            if let Some(path) = FileDialog::new().add_filter("Pack", &["hdpack"]).pick_file() {
                                match get_profiles_dir() {
                                    Ok(profiles_dir) => match Pack::import_to(&path, &profiles_dir) {
                                        Ok(pack) => {
                                            let pack_name = pack.name.clone();
                                            let new_profile_dir = profiles_dir.join(&pack_name);
                                            state.editor = PackEditor::new(pack);
                                            state.profile_dir = new_profile_dir;
                                            state.selected_category = None;
                                            state.selected_macro = None;
                                            state.last_error = None;
                                            state.imported_pack_name = Some(pack_name);
                                        }
                                        Err(e) => state.last_error = Some(e.to_string()),
                                    },
                                    Err(e) => state.last_error = Some(e.to_string()),
                                }
                            }
                        }
                    });
                });
            });

        // Error / status banners
        if let Some(err) = state.last_error.clone() {
            banner(ui, BannerKind::Error, "ERROR", &err, &[]);
        }
        if let Some(ref status) = state.last_test_status.clone() {
            let kind = if status.starts_with("Fired:") { BannerKind::Ok } else { BannerKind::Error };
            banner(ui, kind, "TEST RESULT", status, &[]);
        }

        // ── Three-column layout ───────────────────────────────────────────────
        let category_names: Vec<String> =
            state.editor.pack().categories.iter().map(|c| c.name.clone()).collect();

        let macro_names: Vec<String> = state
            .selected_category.as_deref()
            .and_then(|cat| state.editor.pack().categories.iter().find(|c| c.name == cat))
            .map(|cat| cat.macros.iter().map(|m| m.name.clone()).collect())
            .unwrap_or_default();

        let selected_macro_config: Option<MacroConfig> = state
            .selected_category.as_deref()
            .zip(state.selected_macro.as_deref())
            .and_then(|(cat, mac)| {
                state.editor.pack().categories.iter()
                    .find(|c| c.name == cat)
                    .and_then(|c| c.macros.iter().find(|m| m.name == mac))
                    .cloned()
            });

        let total_w = ui.available_width();
        let col1_w: f32 = 180.0;
        let col2_w: f32 = 240.0;
        let col3_w: f32 = (total_w - col1_w - col2_w - 4.0).max(200.0);

        ui.horizontal(|ui| {
            // ── Column 1: Categories ──────────────────────────────────────────
            ui.vertical(|ui| {
                ui.set_width(col1_w);

                ui.horizontal(|ui| {
                    section_header(ui, "CATEGORIES", None);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("+").clicked() {
                            let name = state.form_new_category.trim().to_string();
                            if !name.is_empty() {
                                match state.editor.add_category(&name) {
                                    Ok(()) => { state.last_error = None; state.form_new_category.clear(); }
                                    Err(e) => state.last_error = Some(e.to_string()),
                                }
                            }
                        }
                    });
                });

                ui.add_space(4.0);
                ui.add(egui::TextEdit::singleline(&mut state.form_new_category)
                    .hint_text("New category…")
                    .desired_width(col1_w - 8.0));

                ui.add_space(4.0);

                egui::ScrollArea::vertical().id_salt("cat_scroll").max_height(320.0).show(ui, |ui| {
                    for cat_name in &category_names {
                        let selected = state.selected_category.as_deref() == Some(cat_name.as_str());
                        let row_bg = if selected { Palette::accent_faint() } else { egui::Color32::TRANSPARENT };
                        let row_bdr = if selected { Palette::accent_line() } else { egui::Color32::TRANSPARENT };

                        egui::Frame::new().fill(row_bg).stroke(egui::Stroke::new(1.0, row_bdr))
                            .corner_radius(egui::CornerRadius::same(3))
                            .inner_margin(egui::Margin::symmetric(8, 4))
                            .show(ui, |ui| {
                                let resp = ui.selectable_label(selected, egui::RichText::new(cat_name.as_str()).color(if selected { Palette::ACCENT } else { Palette::FG }).size(12.0));
                                if resp.clicked() && state.selected_category.as_deref() != Some(cat_name.as_str()) {
                                    state.selected_category = Some(cat_name.clone());
                                    state.selected_macro = None;
                                    state.pending_remove_macro = false;
                                }
                            });
                    }
                });

                // Category actions when one is selected
                if let Some(cat_name) = state.selected_category.clone() {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if ui.small_button("Rename").clicked() { state.show_rename_warning = true; }
                        if ui.small_button("Remove").clicked() {
                            match state.editor.remove_category(&cat_name) {
                                Ok(()) => { state.last_error = None; state.selected_category = None; state.selected_macro = None; }
                                Err(e) => state.last_error = Some(e.to_string()),
                            }
                        }
                    });
                    if state.show_rename_warning {
                        ui.label(egui::RichText::new("⚠ flag references NOT updated").color(Palette::WARN).size(11.0));
                        let new_name = state.form_new_category.trim().to_string();
                        ui.horizontal(|ui| {
                            if ui.small_button("Confirm").clicked() && !new_name.is_empty() {
                                match state.editor.rename_category(&cat_name, &new_name) {
                                    Ok(()) => { state.last_error = None; state.selected_category = Some(new_name); state.form_new_category.clear(); state.show_rename_warning = false; }
                                    Err(e) => { state.last_error = Some(e.to_string()); state.show_rename_warning = false; }
                                }
                            }
                            if ui.small_button("Cancel").clicked() { state.show_rename_warning = false; }
                        });
                    }
                }
            });

            // col divider
            ui.add(egui::Separator::default().vertical());

            // ── Column 2: Macros ──────────────────────────────────────────────
            ui.vertical(|ui| {
                ui.set_width(col2_w);

                section_header(ui, "MACROS", state.selected_category.as_deref());
                ui.add_space(4.0);

                egui::ScrollArea::vertical().id_salt("macro_scroll").max_height(320.0).show(ui, |ui| {
                    for mac_name in &macro_names {
                        let selected = state.selected_macro.as_deref() == Some(mac_name.as_str());
                        let row_bg = if selected { Palette::accent_faint() } else { egui::Color32::TRANSPARENT };

                        egui::Frame::new().fill(row_bg)
                            .corner_radius(egui::CornerRadius::same(3))
                            .inner_margin(egui::Margin::symmetric(8, 4))
                            .show(ui, |ui| {
                                let resp = ui.selectable_label(selected, egui::RichText::new(mac_name.as_str()).color(if selected { Palette::ACCENT } else { Palette::FG }).size(12.0));
                                if resp.clicked() && state.selected_macro.as_deref() != Some(mac_name.as_str()) {
                                    state.selected_macro = Some(mac_name.clone());
                                    state.pending_remove_macro = false;
                                    if let Some(cat) = state.selected_category.as_deref() {
                                        if let Some(m) = state.editor.pack().categories.iter()
                                            .find(|c| c.name == cat)
                                            .and_then(|c| c.macros.iter().find(|m| m.name == *mac_name))
                                        {
                                            state.form_name   = m.name.clone();
                                            state.form_phrase = m.phrase.clone().unwrap_or_default();
                                            state.form_if_flag  = m.if_flag.clone().unwrap_or_default();
                                            state.form_set_flag = m.set_flag.clone().unwrap_or_default();
                                            state.form_keys = m.keys.iter().map(|k| k.key.as_str()).collect::<Vec<_>>().join(", ");
                                        }
                                    }
                                }
                            });
                    }
                });

                ui.add_space(4.0);
                if ui.button(egui::RichText::new("+ Add Macro").color(Palette::ACCENT).size(11.0)).clicked() {
                    if let Some(cat) = state.selected_category.clone() {
                        match build_macro_config_from_form(&state.form_name, &state.form_phrase, &state.form_if_flag, &state.form_set_flag, &state.form_keys) {
                            Ok(mc) => match state.editor.add_macro(&cat, mc) {
                                Ok(()) => state.last_error = None,
                                Err(e) => state.last_error = Some(e.to_string()),
                            },
                            Err(e) => state.last_error = Some(e.to_string()),
                        }
                    } else {
                        state.last_error = Some("Select a category first.".into());
                    }
                }
            });

            // col divider
            ui.add(egui::Separator::default().vertical());

            // ── Column 3: Detail panel ────────────────────────────────────────
            ui.vertical(|ui| {
                ui.set_width(col3_w);

                if state.selected_macro.is_none() {
                    ui.add_space(40.0);
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new("Select a macro to edit").color(Palette::FG_FAINT).size(13.0));
                    });
                    return;
                }

                section_header(ui, "MACRO", None);
                ui.add_space(8.0);

                egui::Grid::new("macro_form_grid").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                    ui.label(egui::RichText::new("NAME").color(Palette::FG_MUTED).size(11.0));
                    ui.text_edit_singleline(&mut state.form_name);
                    ui.end_row();

                    ui.label(egui::RichText::new("PHRASE").color(Palette::FG_MUTED).size(11.0));
                    ui.text_edit_singleline(&mut state.form_phrase);
                    ui.end_row();

                    ui.label(egui::RichText::new("IF FLAG").color(Palette::FG_MUTED).size(11.0));
                    ui.text_edit_singleline(&mut state.form_if_flag);
                    ui.end_row();

                    ui.label(egui::RichText::new("SET FLAG").color(Palette::FG_MUTED).size(11.0));
                    ui.text_edit_singleline(&mut state.form_set_flag);
                    ui.end_row();

                    ui.label(egui::RichText::new("KEYS").color(Palette::FG_MUTED).size(11.0));
                    ui.text_edit_singleline(&mut state.form_keys);
                    ui.end_row();
                });

                // Per-key timing display
                if let Some(ref mc) = selected_macro_config {
                    if !mc.keys.is_empty() {
                        ui.add_space(8.0);
                        section_header(ui, "KEY SEQUENCE", None);
                        ui.add_space(4.0);
                        egui::Frame::new().fill(Palette::BG_EXTREME).stroke(egui::Stroke::new(1.0, Palette::STROKE))
                            .corner_radius(egui::CornerRadius::same(3))
                            .inner_margin(egui::Margin::symmetric(8, 6))
                            .show(ui, |ui| {
                                ui.horizontal_wrapped(|ui| {
                                    for (i, ka) in mc.keys.iter().enumerate() {
                                        if i > 0 { ui.label(egui::RichText::new("→").color(Palette::FG_FAINT).size(11.0)); }
                                        ui.label(egui::RichText::new(&ka.key).color(Palette::ACCENT).size(11.0).monospace());
                                    }
                                });
                            });
                    }
                }

                ui.add_space(12.0);

                // Each frame: fire the pending test macro once the 1-second window elapses.
                if let Some((name, started)) = state.pending_test.take() {
                    if started.elapsed() >= Duration::from_secs(1) {
                        match send_command(ControlRequest::TestMacro { name: name.clone() }) {
                            Ok(ControlResponse::Ok) => state.last_test_status = Some(format!("Fired: {name}")),
                            Ok(ControlResponse::Error { message }) => state.last_test_status = Some(format!("Test failed: {message}")),
                            Ok(other) => state.last_test_status = Some(format!("Unexpected response: {other:?}")),
                            Err(e) => state.last_test_status = Some(format!("Daemon error: {e}")),
                        }
                    } else {
                        state.pending_test = Some((name, started));
                        ui.ctx().request_repaint_after(Duration::from_millis(50));
                    }
                }

                ui.horizontal(|ui| {
                    if primary_button(ui, "Update Macro").clicked() {
                        match (state.selected_category.clone(), state.selected_macro.clone()) {
                            (Some(cat), Some(mac)) => {
                                match parse_key_sequence(&state.form_keys) {
                                    Ok(keys) => {
                                        let phrase = opt_from_str(&state.form_phrase);
                                        let if_flag = opt_from_str(&state.form_if_flag);
                                        let set_flag = opt_from_str(&state.form_set_flag);
                                        let updates = crate::pack::MacroUpdates { phrase: Some(phrase), if_flag: Some(if_flag), set_flag: Some(set_flag), sound: None, keys: Some(keys) };
                                        match state.editor.edit_macro(&cat, &mac, updates) {
                                            Ok(()) => state.last_error = None,
                                            Err(e) => state.last_error = Some(e.to_string()),
                                        }
                                    }
                                    Err(e) => state.last_error = Some(e.to_string()),
                                }
                            }
                            _ => state.last_error = Some("Select a category and macro first.".into()),
                        }
                    }

                    ui.add_space(8.0);

                    // Test button
                    if let Some((ref _name, started)) = state.pending_test.clone() {
                        let remaining = (1.0_f32 - started.elapsed().as_secs_f32()).max(0.0);
                        ui.label(egui::RichText::new(format!("Firing in {remaining:.1}s…")).color(Palette::FG_MUTED).size(12.0));
                        if ui.small_button("Cancel").clicked() {
                            state.pending_test = None;
                        }
                    } else {
                        let test_btn = egui::Button::new(egui::RichText::new("Test").size(12.0));
                        if ui.add_enabled(daemon_running, test_btn).clicked() {
                            let name = state.selected_macro.clone().unwrap_or_default();
                            state.pending_test = Some((name, Instant::now()));
                            state.last_test_status = None;
                        }
                    }
                });

                ui.add_space(8.0);

                // Remove Macro — two-click confirmation
                if state.pending_remove_macro {
                    ui.horizontal(|ui| {
                        let confirm_btn = egui::Button::new(egui::RichText::new("Confirm Remove").color(Palette::ERR).size(12.0))
                            .fill(Palette::err_faint())
                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0xe8, 0x5d, 0x3c, 77)));
                        if ui.add(confirm_btn).clicked() {
                            if let (Some(cat), Some(mac)) = (state.selected_category.clone(), state.selected_macro.clone()) {
                                match state.editor.remove_macro(&cat, &mac) {
                                    Ok(()) => { state.last_error = None; state.selected_macro = None; state.pending_remove_macro = false; state.form_name.clear(); state.form_phrase.clear(); state.form_if_flag.clear(); state.form_set_flag.clear(); state.form_keys.clear(); }
                                    Err(e) => { state.last_error = Some(e.to_string()); state.pending_remove_macro = false; }
                                }
                            }
                        }
                        if ui.small_button("Cancel").clicked() { state.pending_remove_macro = false; }
                    });
                } else if ui.small_button("Remove Macro").clicked() {
                    state.pending_remove_macro = true;
                }
            });
        });
    }

    /// Convert a form string to `Some(s)` when non-empty, `None` to clear the field.
    /// Used by Update Macro to produce `Some(None)` (clear) vs `Some(Some(s))` (set).
    fn opt_from_str(s: &str) -> Option<String> {
        let trimmed = s.trim();
        if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
    }
}

// ── Pure-logic helpers (no gui feature required) ───────────────────────────────

use crate::config::{KeyAction, MacroConfig};
use anyhow::{bail, Result};

/// Errors returned by [`build_macro_config_from_form`].
#[derive(Debug)]
pub enum FormBuildError {
    EmptyName,
    KeyParseError(anyhow::Error),
}

impl std::fmt::Display for FormBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormBuildError::EmptyName => write!(f, "macro name must not be empty"),
            FormBuildError::KeyParseError(e) => write!(f, "key parse error: {e}"),
        }
    }
}

impl std::error::Error for FormBuildError {}

/// Build a [`MacroConfig`] from raw form-field strings.
///
/// - `name` must be non-empty (after trim).
/// - `phrase`, `if_flag`, `set_flag` become `None` when empty.
/// - `keys` is parsed via [`parse_key_sequence`].
///
/// This is a pure function with no egui dependency — it is unit-tested directly.
pub fn build_macro_config_from_form(
    name: &str,
    phrase: &str,
    if_flag: &str,
    set_flag: &str,
    keys: &str,
) -> Result<MacroConfig, FormBuildError> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(FormBuildError::EmptyName);
    }
    let phrase = {
        let s = phrase.trim();
        if s.is_empty() { None } else { Some(s.to_string()) }
    };
    let if_flag = {
        let s = if_flag.trim();
        if s.is_empty() { None } else { Some(s.to_string()) }
    };
    let set_flag = {
        let s = set_flag.trim();
        if s.is_empty() { None } else { Some(s.to_string()) }
    };
    let keys = parse_key_sequence(keys).map_err(FormBuildError::KeyParseError)?;

    Ok(MacroConfig { name, phrase, if_flag, set_flag, sound: None, keys })
}

/// Parse a comma-separated key sequence string into a `Vec<KeyAction>`.
///
/// Each token is trimmed of whitespace and becomes a `KeyAction` with `dwell_ms`
/// and `gap_ms` left as `None` (inheriting global timing defaults).
/// Returns an error if the input is empty or any token is empty after trimming.
pub fn parse_key_sequence(input: &str) -> Result<Vec<KeyAction>> {
    if input.trim().is_empty() {
        bail!("key sequence must not be empty");
    }

    let mut actions = Vec::new();
    for token in input.split(',') {
        let key = token.trim().to_string();
        if key.is_empty() {
            bail!("key sequence contains an empty token (check for trailing commas)");
        }
        actions.push(KeyAction { key, dwell_ms: None, gap_ms: None });
    }

    Ok(actions)
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{build_macro_config_from_form, parse_key_sequence};

    // ── parse_key_sequence ────────────────────────────────────────────────────

    #[test]
    fn parse_key_sequence_single() {
        let result = parse_key_sequence("KEY_W").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].key, "KEY_W");
        assert!(result[0].dwell_ms.is_none());
        assert!(result[0].gap_ms.is_none());
    }

    #[test]
    fn parse_key_sequence_multiple_with_whitespace() {
        let result = parse_key_sequence("KEY_W , KEY_A , KEY_S").unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].key, "KEY_W");
        assert_eq!(result[1].key, "KEY_A");
        assert_eq!(result[2].key, "KEY_S");
    }

    #[test]
    fn parse_key_sequence_empty_errors() {
        let err = parse_key_sequence("").unwrap_err();
        assert!(err.to_string().contains("empty"), "error must mention empty input");
    }

    #[test]
    fn parse_key_sequence_trailing_comma() {
        let err = parse_key_sequence("KEY_W,").unwrap_err();
        assert!(
            err.to_string().contains("empty token"),
            "error must mention empty token for trailing comma"
        );
    }

    // ── build_macro_config_from_form ──────────────────────────────────────────

    #[test]
    fn build_macro_config_from_form_minimal_fields() {
        let mc = build_macro_config_from_form("Reinforce", "", "", "", "KEY_UP").unwrap();
        assert_eq!(mc.name, "Reinforce");
        assert_eq!(mc.phrase, None);
        assert_eq!(mc.if_flag, None);
        assert_eq!(mc.set_flag, None);
        assert_eq!(mc.sound, None);
        assert_eq!(mc.keys.len(), 1);
        assert_eq!(mc.keys[0].key, "KEY_UP");
    }

    #[test]
    fn build_macro_config_from_form_clears_optional_when_empty() {
        // Empty phrase/if_flag/set_flag must produce None, not Some("").
        let mc =
            build_macro_config_from_form("Test", "  ", "  ", "  ", "KEY_DOWN, KEY_UP").unwrap();
        assert_eq!(mc.phrase, None, "whitespace-only phrase must become None");
        assert_eq!(mc.if_flag, None, "whitespace-only if_flag must become None");
        assert_eq!(mc.set_flag, None, "whitespace-only set_flag must become None");
        assert_eq!(mc.keys.len(), 2);
    }

    #[test]
    fn build_macro_config_from_form_propagates_key_parse_error() {
        let err = build_macro_config_from_form("Test", "", "", "", "KEY_UP,").unwrap_err();
        assert!(
            err.to_string().contains("empty token"),
            "FormBuildError must surface key parse error; got: {err}"
        );
    }

    #[test]
    fn build_macro_config_from_form_empty_name_errors() {
        let err = build_macro_config_from_form("  ", "", "", "", "KEY_UP").unwrap_err();
        assert!(
            err.to_string().contains("name must not be empty"),
            "FormBuildError must report empty name; got: {err}"
        );
    }

    #[test]
    fn build_macro_config_from_form_populates_optional_fields() {
        let mc =
            build_macro_config_from_form("Eagle", "eagle airstrike", "combat", "fired", "KEY_UP")
                .unwrap();
        assert_eq!(mc.phrase, Some("eagle airstrike".into()));
        assert_eq!(mc.if_flag, Some("combat".into()));
        assert_eq!(mc.set_flag, Some("fired".into()));
    }
}
