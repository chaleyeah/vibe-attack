---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T02: Implement egui editor panel: category/macro lists, edit form, CRUD buttons, Save with SwitchProfile dispatch

Flesh out `show_pack_editor` in `src/ui/pack_editor.rs` (inside the `#[cfg(feature = "gui")]` `inner` module) to render the full editor UX. Layout: use `egui::SidePanel` or three `egui::ScrollArea` columns inside a `ui.horizontal` — left column = category list (`selectable_label` per category, click sets `state.selected_category`); middle column = macro list for the selected category (`selectable_label` per macro, click populates the form fields by reading from the underlying `MacroConfig`); right column = the edit form. Form fields: `TextEdit::singleline` for name, phrase, if_flag, set_flag, and the comma-separated keys string; for the selected macro's keys, render each `KeyAction`'s optional `dwell_ms`/`gap_ms` as `DragValue<u64>` rows with a checkbox to enable the override. Buttons in the form area: `Add Macro` (calls `state.editor.add_macro(category, MacroConfig { ... })` built from form state via a private `build_macro_config_from_form` helper that calls `parse_key_sequence`); `Update Macro` (calls `edit_macro` with a `MacroUpdates` built using the `Some(Some(_))` / `Some(None)` semantics from MEM055 — `Some(Some(s))` for non-empty fields, `Some(None)` to explicitly clear); `Remove Macro` (with a small inline confirmation: a follow-up confirm button that appears after first click). Buttons in a toolbar above the category list: `Add Category` (with a singleline TextEdit for the new name), `Remove Category` (refused by `PackEditor` if non-empty — surface the error in `state.last_error`), `Rename Category` (when invoked, set `state.show_rename_warning = true` and render a yellow `Color32::YELLOW` colored_label warning that `if_flag`/`set_flag` macro references are NOT cascaded; user must confirm via a second click before the rename is applied). Bottom of panel: a `Save` button that calls a new method `PackEditorState::save(&self) -> anyhow::Result<()>` which performs `self.editor.pack().save_to_dir(&self.profile_dir)` then `send_command(ControlRequest::SwitchProfile { name: self.editor.pack().name.clone() })` (the SwitchProfile error is logged via `tracing::warn!` if the daemon is absent or returns Err, but the save itself succeeds — return Ok if the disk write succeeded). Render `state.last_error` as a `Color32::RED` colored_label whenever it's `Some`. After every CRUD call, wipe `state.last_error` to `None` on success or set it to the error's `to_string()` on failure. Borrow-checker note: when iterating `state.editor.pack().categories` inside an egui closure, follow the existing `app.device_names.clone()` precedent — clone `Vec<String>` of category and macro names BEFORE the egui closure to avoid simultaneous mutable borrows of `state`. Add unit tests under `pack_editor::tests` for pure-logic helpers: `build_macro_config_from_form_minimal_fields`, `build_macro_config_from_form_clears_optional_when_empty`, `build_macro_config_from_form_propagates_key_parse_error`. Form-state assembly must NOT call any egui function — it must be testable from a pure unit test.

## Inputs

- ``src/ui/pack_editor.rs``
- ``src/pack/mod.rs``
- ``src/config.rs``
- ``src/control/client.rs``
- ``src/control/protocol.rs``

## Expected Output

- ``src/ui/pack_editor.rs``

## Verification

RUSTFLAGS="-D warnings" cargo clippy --all-targets --features gui -- -D warnings && cargo test --lib ui::pack_editor -- --test-threads=1

## Observability Impact

Save path emits `tracing::info!(path, macro_count, "pack saved")` on success and `tracing::warn!(reason, "SwitchProfile dispatch failed")` when send_command errors. The existing log scroll in vibe-attack-config drains tracing events into the UI, so failures are visible without new plumbing. `state.last_error` provides synchronous in-panel error visibility for CRUD operations.
