# S03: Egui editor panel — Research

**Date:** 2026-04-27

## Summary

S03 adds an egui panel inside the existing `vibe-attack-config` binary that lets the user browse categories and macros in the active pack, add/edit/remove macros through a form, and save the result back to `pack.yaml` on disk. The PackEditor logic is fully built (S02); this slice is purely the egui surface wired to that API.

The entry point for all GUI work is `src/bin/vibe-attack-config.rs`. It owns `VibeAttackConfigApp`, which has a `show_main_config()` free function for the post-wizard main view. That function already renders daemon status, mic level, mode selection, threshold, device picker, profiles list, and log scroll. The editor panel will be added into or adjacent to that function — either as an expandable section inside `show_main_config` or as a separate `show_editor_panel()` helper called from it. The existing pattern (wizard uses `show_wizard()`, main config uses `show_main_config()`) strongly favors a new `show_pack_editor()` free function.

`rfd` (Rusty File Dialogs) is **not** in `Cargo.toml`. The M009 roadmap references it for S04 import/export file pickers; S03 does not need file pickers at all — pack saves go directly to the XDG pack directory using `Pack::save_to_dir`. The `eframe` version is `0.34` with `x11` + `wayland` features. No new egui/eframe deps are required for S03. The feature gate for the binary is already `required-features = ["gui"]`, and `wizard.rs` establishes the pattern: `#[cfg(feature = "gui")]` wrapping an `inner` module with the egui code re-exported via `pub use inner::*`. The same pattern must be applied to any new `src/ui/pack_editor.rs` module.

## Recommendation

Create `src/ui/pack_editor.rs` behind `#[cfg(feature = "gui")]` that holds:
1. `PackEditorState` — pure-logic struct holding the loaded `PackEditor`, an optional `(category_name, macro_name)` selection, and transient form strings for the add/edit macro form.
2. `show_pack_editor(ui, state)` — egui function called from `show_main_config`.

Wire `PackEditorState` into `VibeAttackConfigApp` (field `pack_editor: Option<PackEditorState>`) and populate it when a profile is selected in the profiles list. On "Save" in the editor panel, call `ed.into_pack().save_to_dir(profile_dir)` and then send `ControlRequest::SwitchProfile { name }` to the daemon so it reloads the changed pack without restart. This matches the existing `handle_save` pattern (compute → disk → socket) and uses the already-wired `send_command` from `src/control/client.rs`.

The `Option<PackEditorState>` approach (rather than always-initialized) avoids loading any pack until the user selects one, keeps startup cost zero, and mirrors the existing `cached_config: Option<Config>` pattern.

## Implementation Landscape

### Key Files

- `src/bin/vibe-attack-config.rs` — owns `VibeAttackConfigApp` and the egui update loop. `show_main_config()` is the injection point; add `pack_editor: Option<PackEditorState>` to `VibeAttackConfigApp` and call `show_pack_editor(ui, state)` from `show_main_config`. File is ~496 lines; the profiles list (lines 396–404) is the natural trigger for loading a pack into the editor.

- `src/ui/config_app.rs` — pure-logic `ConfigApp` struct. No egui in this file; it is not feature-gated. This file should NOT be modified for S03; pack editor state is GUI-only and belongs in the binary or a gui-feature module.

- `src/ui/mod.rs` — declares all UI submodules. A new `pub mod pack_editor;` with `#[cfg(feature = "gui")]` must be added here so `vibe-attack-config.rs` can import from it.

- `src/ui/wizard.rs` — establishes the `#[cfg(feature = "gui")] mod inner { ... } pub use inner::*;` pattern. The new `src/ui/pack_editor.rs` must follow this exact pattern so a `cargo build` (no gui feature) stays clean.

- `src/pack/mod.rs` — `PackEditor` and `MacroUpdates` are the full CRUD API S03 consumes. Key methods: `add_macro`, `edit_macro`, `remove_macro`, `move_macro`, `rename_category`, `add_category`, `remove_category`. All return `Result<()>`. `pack()` returns `&Pack`, `into_pack()` consumes the editor. `Pack::save_to_dir(dir)` is the write path. Already pub and accessible under the default feature set.

- `src/control/client.rs` — `send_command(ControlRequest)` sends to the daemon socket. `ControlRequest::SwitchProfile { name }` reloads the pack in the daemon without restart. This is the correct post-save notification.

- `src/control/protocol.rs` — `ControlRequest` enum. No variant needs adding for S03; `SwitchProfile` covers the post-save daemon reload.

- `src/config.rs` — `MacroConfig` and `KeyAction` struct definitions (the data shapes the form must produce). `MacroConfig` has: `name: String`, `phrase: Option<String>`, `if_flag: Option<String>`, `set_flag: Option<String>`, `sound: Option<PathBuf>`, `keys: Vec<KeyAction>`. `KeyAction` has: `key: String`, `dwell_ms: Option<u64>`, `gap_ms: Option<u64>`. Note `#[serde(deny_unknown_fields)]` — the struct must be constructed fully (no shortcut).

- `Cargo.toml` — `eframe = "0.34"` optional under `gui` feature. `rfd` is absent; do NOT add it for S03 (it belongs to S04). No new deps needed.

- `profiles/hd2/pack.yaml` — the HD2 bundled pack (75 stratagems). The editor panel will be exercised against this pack in UAT.

### Build Order

1. **Scaffold `src/ui/pack_editor.rs`** with the `#[cfg(feature = "gui")]` wrapper and an empty `PackEditorState::new(pack: PackEditor)` and `show_pack_editor()` stub. Verify `cargo build --features gui` and `cargo build` (no feature) both pass before adding any real egui code.

2. **Wire `PackEditorState` into `VibeAttackConfigApp`**: add `pack_editor: Option<PackEditorState>` field and initialize to `None`. In the profiles list interaction, load the selected pack via `Pack::load_from_dir` and wrap it in `PackEditor::new`. This proves the load path before any UI rendering.

3. **Implement read-only panel first**: render the category list (left column) and macro list (right column) for the selected category without any edit form. This confirms layout compiles and egui `ScrollArea` + `selectable_label` pattern works as expected.

4. **Add the macro form** (add/edit with `TextEdit` for name/phrase, `DragValue` for timing fields, key sequence as a comma-separated string entry). Bind the Save button: call `editor.edit_macro` or `editor.add_macro`, on success call `pack.save_to_dir` then `send_command(SwitchProfile)`.

5. **Add remove/move/category CRUD** in the toolbar row above the category list.

6. **Write unit tests** for `PackEditorState` logic (parse key sequence string, form validation). Integration test: load a pack, add a macro via the state machine (not the UI), save, reload, verify.

### Verification Approach

- `cargo build` (default, no features) — must compile clean; no gui-gated code must leak.
- `cargo build --features gui` — must compile clean.
- `cargo clippy --features gui -- -D warnings` — must be clean.
- `cargo test` — all 22+ existing tests must still pass; new unit tests for `PackEditorState`.
- Manual smoke: `cargo run --bin vibe-attack-config --features gui`, select hd2 profile, add a macro, save, verify `~/.config/vibe-attack/profiles/hd2/pack.yaml` updated.
- If daemon is running: verify `SwitchProfile` is received and the daemon reloads (check log output).

## Constraints

- The `vibe-attack-config` binary is `required-features = ["gui"]`; all egui code is already behind that gate. Any new module must use `#[cfg(feature = "gui")]` to keep `cargo build` (no features) clean.
- `MacroConfig` uses `#[serde(deny_unknown_fields)]`; it must be constructed with all fields present (no Default derive exists on it) — use explicit struct literal.
- `Pack::save_to_dir` is NOT atomic (uses `File::create` directly, unlike the `.tmp` rename in `save_app_to_config`). For S03 an atomic write is a quality improvement but not a milestone gate; note it and ship non-atomic for now, or copy the `.tmp` rename pattern.
- `Pack::import` calls `get_profiles_dir()` internally (XDG-coupled); keep it out of S03. The save path for S03 is always `Pack::save_to_dir(profile_dir)` with an explicit path derived from the active profile name.
- `rfd` is absent from `Cargo.toml`. Do not add it for S03. File pickers are S04 scope.
- `rename_category` does NOT cascade to macro `if_flag`/`set_flag` references (documented S02 decision MEM027). The UI should display a warning or note if the user renames a category, but must not attempt to cascade.
- Key sequence entry in the form must handle parse errors gracefully (unknown key name). The daemon validates key names at startup; the editor should warn but not block save.

## Common Pitfalls

- **Feature guard omission** — if a new `use` import or type reference from `eframe::egui` appears outside `#[cfg(feature = "gui")]`, the default-feature build breaks with an unresolved import. Always verify both `cargo build` and `cargo build --features gui`.
- **Borrowing `VibeAttackConfigApp` in egui closures** — egui's `show_ui` closures take `&mut Ui` and close over outer `&mut self` fields. Rust borrow checker rejects simultaneous mutable borrows. The fix is the same pattern already in `show_main_config`: clone the `Vec` before iterating (e.g. `app.device_names.clone()`), or split the borrow by extracting the field reference before the closure.
- **Non-atomic pack save** — `Pack::save_to_dir` truncates and rewrites; a crash mid-write corrupts the file. This is an existing limitation of the method. For S03, document the gap; do not add a separate write path that diverges from the library function.
- **ProfileDir path** — the profile directory for save must be `get_profiles_dir()?.join(&profile_name)`, not a relative path. Use `vibe_attack::pack::get_profiles_dir()` which is already pub.
- **`SwitchProfile` with wrong name** — the daemon matches profile names to subdirectory names. The name passed to `SwitchProfile` must exactly match the `pack.name` field (which equals the subdirectory name by convention). Verify that `PackEditor.pack().name` equals the subdirectory name before sending.
- **egui `TextEdit` retains focus across frames** — if the add-macro form is shown in a collapsible section, egui may retain focus state when it is hidden and re-shown. Use `ui.ctx().memory_mut(|m| m.request_focus(...))` only when opening the form, not every frame.
