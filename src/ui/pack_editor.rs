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

    use eframe::egui;
    use tracing;

    use crate::config::MacroConfig;
    use crate::control::client::send_command;
    use crate::control::protocol::ControlRequest;
    use crate::pack::PackEditor;

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
    pub fn show_pack_editor(ui: &mut egui::Ui, state: &mut PackEditorState) {
        ui.heading(format!("Pack Editor — {}", state.editor.pack().name));

        // ── Category toolbar ─────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("New category:");
            ui.text_edit_singleline(&mut state.form_new_category);

            if ui.button("Add Category").clicked() {
                let name = state.form_new_category.trim().to_string();
                match state.editor.add_category(&name) {
                    Ok(()) => {
                        state.last_error = None;
                        state.form_new_category.clear();
                    }
                    Err(e) => state.last_error = Some(e.to_string()),
                }
            }

            // Remove Category: only offered when a category is selected.
            if let Some(cat_name) = state.selected_category.clone() {
                if ui.button("Remove Category").clicked() {
                    match state.editor.remove_category(&cat_name) {
                        Ok(()) => {
                            state.last_error = None;
                            state.selected_category = None;
                            state.selected_macro = None;
                        }
                        Err(e) => state.last_error = Some(e.to_string()),
                    }
                }

                // Rename Category (with two-click confirmation).
                if ui.button("Rename Category").clicked() {
                    state.show_rename_warning = true;
                }

                if state.show_rename_warning {
                    ui.colored_label(
                        egui::Color32::YELLOW,
                        "⚠ if_flag/set_flag references are NOT updated — confirm rename?",
                    );
                    let new_name = state.form_new_category.trim().to_string();
                    if ui.button("Confirm Rename").clicked() && !new_name.is_empty() {
                        match state.editor.rename_category(&cat_name, &new_name) {
                            Ok(()) => {
                                state.last_error = None;
                                state.selected_category = Some(new_name.clone());
                                state.form_new_category.clear();
                                state.show_rename_warning = false;
                            }
                            Err(e) => {
                                state.last_error = Some(e.to_string());
                                state.show_rename_warning = false;
                            }
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        state.show_rename_warning = false;
                    }
                }
            }
        });

        ui.add_space(4.0);

        // ── Three-column layout ───────────────────────────────────────────────
        // Clone lists before the egui closures to avoid simultaneous mut borrows.
        let category_names: Vec<String> =
            state.editor.pack().categories.iter().map(|c| c.name.clone()).collect();

        let macro_names: Vec<String> = state
            .selected_category
            .as_deref()
            .and_then(|cat| state.editor.pack().categories.iter().find(|c| c.name == cat))
            .map(|cat| cat.macros.iter().map(|m| m.name.clone()).collect())
            .unwrap_or_default();

        // Snapshot current macro's MacroConfig for form pre-population (read-only borrow ends here).
        let selected_macro_config: Option<MacroConfig> = state
            .selected_category
            .as_deref()
            .zip(state.selected_macro.as_deref())
            .and_then(|(cat, mac)| {
                state
                    .editor
                    .pack()
                    .categories
                    .iter()
                    .find(|c| c.name == cat)
                    .and_then(|c| c.macros.iter().find(|m| m.name == mac))
                    .cloned()
            });

        ui.horizontal(|ui| {
            // ── Left: category list ─────────────────────────────────────────
            ui.vertical(|ui| {
                ui.set_min_width(140.0);
                ui.label("Categories");
                egui::ScrollArea::vertical()
                    .id_salt("cat_scroll")
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for cat_name in &category_names {
                            let selected =
                                state.selected_category.as_deref() == Some(cat_name.as_str());
                            if ui.selectable_label(selected, cat_name.as_str()).clicked() {
                                if state.selected_category.as_deref() != Some(cat_name.as_str()) {
                                    state.selected_category = Some(cat_name.clone());
                                    state.selected_macro = None;
                                    state.pending_remove_macro = false;
                                }
                            }
                        }
                    });
            });

            ui.separator();

            // ── Middle: macro list ──────────────────────────────────────────
            ui.vertical(|ui| {
                ui.set_min_width(160.0);
                ui.label("Macros");
                egui::ScrollArea::vertical()
                    .id_salt("macro_scroll")
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for mac_name in &macro_names {
                            let selected =
                                state.selected_macro.as_deref() == Some(mac_name.as_str());
                            if ui.selectable_label(selected, mac_name.as_str()).clicked() {
                                if state.selected_macro.as_deref() != Some(mac_name.as_str()) {
                                    state.selected_macro = Some(mac_name.clone());
                                    state.pending_remove_macro = false;
                                    // Populate form from the just-selected macro.
                                    if let Some(cat) = state.selected_category.as_deref() {
                                        if let Some(m) = state
                                            .editor
                                            .pack()
                                            .categories
                                            .iter()
                                            .find(|c| c.name == cat)
                                            .and_then(|c| {
                                                c.macros.iter().find(|m| m.name == *mac_name)
                                            })
                                        {
                                            state.form_name = m.name.clone();
                                            state.form_phrase =
                                                m.phrase.clone().unwrap_or_default();
                                            state.form_if_flag =
                                                m.if_flag.clone().unwrap_or_default();
                                            state.form_set_flag =
                                                m.set_flag.clone().unwrap_or_default();
                                            state.form_keys = m
                                                .keys
                                                .iter()
                                                .map(|k| k.key.as_str())
                                                .collect::<Vec<_>>()
                                                .join(", ");
                                        }
                                    }
                                }
                            }
                        }
                    });
            });

            ui.separator();

            // ── Right: edit form ────────────────────────────────────────────
            ui.vertical(|ui| {
                ui.set_min_width(220.0);
                ui.label("Edit Macro");

                egui::Grid::new("macro_form_grid")
                    .num_columns(2)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut state.form_name);
                        ui.end_row();

                        ui.label("Phrase:");
                        ui.text_edit_singleline(&mut state.form_phrase);
                        ui.end_row();

                        ui.label("If flag:");
                        ui.text_edit_singleline(&mut state.form_if_flag);
                        ui.end_row();

                        ui.label("Set flag:");
                        ui.text_edit_singleline(&mut state.form_set_flag);
                        ui.end_row();

                        ui.label("Keys:");
                        ui.text_edit_singleline(&mut state.form_keys);
                        ui.end_row();
                    });

                // Per-key dwell/gap overrides (rendered for selected macro only).
                if let Some(ref mc) = selected_macro_config {
                    if !mc.keys.is_empty() {
                        ui.add_space(4.0);
                        ui.label("Key timing overrides (edit macro to apply):");
                        for ka in &mc.keys {
                            ui.horizontal(|ui| {
                                ui.monospace(ka.key.as_str());
                                ui.label(format!(
                                    "dwell: {} ms   gap: {} ms",
                                    ka.dwell_ms.map_or("default".into(), |v| v.to_string()),
                                    ka.gap_ms.map_or("default".into(), |v| v.to_string()),
                                ));
                            });
                        }
                    }
                }

                ui.add_space(6.0);

                // CRUD buttons.
                ui.horizontal(|ui| {
                    // Add Macro.
                    if ui.button("Add Macro").clicked() {
                        if let Some(cat) = state.selected_category.clone() {
                            match build_macro_config_from_form(
                                &state.form_name,
                                &state.form_phrase,
                                &state.form_if_flag,
                                &state.form_set_flag,
                                &state.form_keys,
                            ) {
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

                    // Update Macro.
                    if ui.button("Update Macro").clicked() {
                        match (state.selected_category.clone(), state.selected_macro.clone()) {
                            (Some(cat), Some(mac)) => {
                                match parse_key_sequence(&state.form_keys) {
                                    Ok(keys) => {
                                        let phrase = opt_from_str(&state.form_phrase);
                                        let if_flag = opt_from_str(&state.form_if_flag);
                                        let set_flag = opt_from_str(&state.form_set_flag);
                                        let updates = crate::pack::MacroUpdates {
                                            phrase: Some(phrase),
                                            if_flag: Some(if_flag),
                                            set_flag: Some(set_flag),
                                            sound: None,
                                            keys: Some(keys),
                                        };
                                        match state.editor.edit_macro(&cat, &mac, updates) {
                                            Ok(()) => state.last_error = None,
                                            Err(e) => {
                                                state.last_error = Some(e.to_string())
                                            }
                                        }
                                    }
                                    Err(e) => state.last_error = Some(e.to_string()),
                                }
                            }
                            _ => {
                                state.last_error =
                                    Some("Select a category and macro first.".into())
                            }
                        }
                    }
                });

                // Remove Macro — two-click confirmation.
                ui.horizontal(|ui| {
                    if state.selected_macro.is_some() {
                        if state.pending_remove_macro {
                            if ui
                                .button(egui::RichText::new("Confirm Remove").color(egui::Color32::RED))
                                .clicked()
                            {
                                if let (Some(cat), Some(mac)) = (
                                    state.selected_category.clone(),
                                    state.selected_macro.clone(),
                                ) {
                                    match state.editor.remove_macro(&cat, &mac) {
                                        Ok(()) => {
                                            state.last_error = None;
                                            state.selected_macro = None;
                                            state.pending_remove_macro = false;
                                            state.form_name.clear();
                                            state.form_phrase.clear();
                                            state.form_if_flag.clear();
                                            state.form_set_flag.clear();
                                            state.form_keys.clear();
                                        }
                                        Err(e) => {
                                            state.last_error = Some(e.to_string());
                                            state.pending_remove_macro = false;
                                        }
                                    }
                                }
                            }
                            if ui.button("Cancel").clicked() {
                                state.pending_remove_macro = false;
                            }
                        } else if ui.button("Remove Macro").clicked() {
                            state.pending_remove_macro = true;
                        }
                    }
                });
            });
        });

        ui.add_space(6.0);

        // ── Save button ───────────────────────────────────────────────────────
        if ui.button("💾 Save Pack").clicked() {
            match state.save() {
                Ok(()) => state.last_error = None,
                Err(e) => state.last_error = Some(e.to_string()),
            }
        }

        // ── Inline error display ─────────────────────────────────────────────
        if let Some(err) = &state.last_error.clone() {
            ui.colored_label(egui::Color32::RED, format!("⚠ {err}"));
        }
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
