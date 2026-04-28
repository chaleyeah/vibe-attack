---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T01: Scaffold pack_editor module and wire PackEditorState into VibeAttackConfigApp

Create `src/ui/pack_editor.rs` following the exact `#[cfg(feature = "gui")] mod inner { ... } pub use inner::*;` pattern used in `src/ui/wizard.rs`. Inside `inner`, define a `PackEditorState` struct that owns: `editor: PackEditor`, `profile_dir: PathBuf`, `selected_category: Option<String>`, `selected_macro: Option<String>`, transient form-state strings (`form_name: String`, `form_phrase: String`, `form_if_flag: String`, `form_set_flag: String`, `form_keys: String`), `show_rename_warning: bool`, and `last_error: Option<String>`. Expose constructors `PackEditorState::new(editor: PackEditor, profile_dir: PathBuf) -> Self` and a stub `show_pack_editor(ui: &mut egui::Ui, state: &mut PackEditorState)` that for now renders only a placeholder heading + the loaded pack name. Implement a pure helper `parse_key_sequence(input: &str) -> anyhow::Result<Vec<KeyAction>>` (comma-separated; trims whitespace; each token becomes `KeyAction { key: token.to_string(), dwell_ms: None, gap_ms: None }`; rejects empty input with a clear error). Add `pub mod pack_editor;` (with `#[cfg(feature = "gui")]` matching `tray`) to `src/ui/mod.rs`. In `src/bin/vibe-attack-config.rs`, add `pack_editor: Option<vibe_attack::ui::pack_editor::PackEditorState>` to `VibeAttackConfigApp` (initialized to `None`); modify the profiles-list loop in `show_main_config` so clicking a profile name loads `Pack::load_from_dir(get_profiles_dir()?.join(name))`, wraps it in `PackEditor::new`, and stores `Some(PackEditorState::new(editor, profile_dir))` in `app.pack_editor`. Below the profiles list, if `app.pack_editor.is_some()`, call `show_pack_editor(ui, app.pack_editor.as_mut().unwrap())`. Add unit tests under `pack_editor::tests`: `parse_key_sequence_single`, `parse_key_sequence_multiple_with_whitespace`, `parse_key_sequence_empty_errors`, `parse_key_sequence_trailing_comma`. CRITICAL: verify `cargo build` (no features) compiles clean BEFORE adding any further egui code — any leaked `eframe::egui` import outside the `#[cfg(feature = "gui")]` gate will break the default build.

## Inputs

- ``src/ui/wizard.rs``
- ``src/ui/mod.rs``
- ``src/bin/vibe-attack-config.rs``
- ``src/pack/mod.rs``
- ``src/config.rs``

## Expected Output

- ``src/ui/pack_editor.rs``
- ``src/ui/mod.rs``
- ``src/bin/vibe-attack-config.rs``

## Verification

cargo build && cargo build --features gui && cargo test --lib ui::pack_editor -- --test-threads=1

## Observability Impact

Adds a `last_error: Option<String>` field on `PackEditorState` that downstream tasks will render in the panel; no runtime side effects yet (T01 only scaffolds the load path).
