/// Pack editor panel for browsing and editing macro packs.
///
/// All egui code is gated to `gui` — the default build (no eframe) only compiles
/// the pure-logic `parse_key_sequence` helper exposed by the tests below.

#[cfg(feature = "gui")]
pub use inner::*;

#[cfg(feature = "gui")]
mod inner {
    use std::path::PathBuf;

    use eframe::egui;

    use crate::pack::PackEditor;

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
                last_error: None,
            }
        }
    }

    /// Stub panel — renders a placeholder heading and the loaded pack name.
    /// Subsequent tasks fill in the full form UI.
    pub fn show_pack_editor(ui: &mut egui::Ui, state: &mut PackEditorState) {
        ui.heading("Pack Editor");
        ui.label(format!("Pack: {}", state.editor.pack().name));

        if let Some(err) = &state.last_error.clone() {
            ui.colored_label(egui::Color32::RED, format!("Error: {err}"));
        }
    }
}

// ── Key-sequence parser (pure logic — no gui feature required) ────────────────

use crate::config::KeyAction;
use anyhow::{bail, Result};

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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::parse_key_sequence;

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
}
