# S03: Egui editor panel

**Goal:** Add an egui pack editor panel inside vibe-attack-config that lets the user browse categories and macros in the active pack, edit them through a form, save back to pack.yaml on disk, and trigger a daemon SwitchProfile reload — wrapping the PackEditor CRUD API delivered in S02 with a real GUI surface.
**Demo:** vibe-attack-config opens editor panel; user adds a new macro via the form; clicks Save; pack.yaml updated on disk; daemon picks up change via SwitchProfile (already shipping)

## Must-Haves

- ## Must-Haves
- `src/ui/pack_editor.rs` exists, follows the `#[cfg(feature = "gui")] pub use inner::*;` pattern from `src/ui/wizard.rs`, and contains a `PackEditorState` type plus a `show_pack_editor` egui entry function.
- `src/ui/mod.rs` declares `pub mod pack_editor;` (gated behind `#[cfg(feature = "gui")]` like `tray`).
- `VibeAttackConfigApp` in `src/bin/vibe-attack-config.rs` owns an `Option<PackEditorState>` that is `None` until the user clicks a profile name in the profiles list, at which point the named pack is loaded via `Pack::load_from_dir(get_profiles_dir()?.join(&name))` and wrapped in `PackEditor::new`.
- The editor panel in `show_pack_editor` exposes, at minimum: category list (selectable), macro list within the selected category (selectable), an edit form bound to the selected macro (name, phrase, if_flag, set_flag, comma-separated key sequence, optional per-key dwell_ms/gap_ms via DragValue), Add Macro / Remove Macro buttons, Add Category / Remove Category / Rename Category controls, and a Save button.
- Save click path: `PackEditorState::save(&profile_dir)` calls `editor.pack().save_to_dir(profile_dir)`, then `send_command(ControlRequest::SwitchProfile { name: pack.name.clone() })` (best-effort; ignored if the daemon is not running, mirroring `handle_save`'s pattern).
- Rename Category surfaces a visible warning in the panel that macro `if_flag` / `set_flag` references are NOT cascaded (per MEM027 / S02 decision), so the user can manually fix references if needed.
- Pure-logic key-sequence parser (`parse_key_sequence(&str) -> Result<Vec<KeyAction>>`) and form-to-MacroUpdates assembly are isolated from egui code and unit-tested under `src/ui/pack_editor.rs::tests`.
- New integration test `tests/pack_editor_state_roundtrip.rs` exercises the state-machine layer end-to-end: build a `PackEditorState` from a fixture pack, drive it through CRUD ops via the same calls the UI would make, save to a tempdir, reload, assert equivalence using a private `assert_packs_equal` helper (mirror the S02 pattern from `tests/pack_editor_roundtrip.rs`).
- `cargo build` (no features) and `cargo build --features gui` both compile clean.
- `RUSTFLAGS="-D warnings" cargo clippy --all-targets --features gui` is clean.
- `cargo test` (full suite) passes; the 78 prior tests remain green and the new tests for `PackEditorState` and the state-roundtrip pass.
- ## Threat Surface
- **Abuse**: Editor writes only inside `$XDG_CONFIG_HOME/vibe-attack/profiles/<name>/pack.yaml` — paths derived from the active profile name, not from user-typed paths. No path traversal surface added.
- **Data exposure**: pack.yaml has no secrets; the editor reads/writes the same file the user already owns.
- **Input trust**: User-entered key-name strings (`KEY_*`) flow into pack.yaml. The daemon already validates evdev key names at startup; the editor surfaces a non-blocking warning on parse failure but does not block save (consistent with research doc `Constraints` and `Common Pitfalls`).
- ## Requirement Impact
- **Requirements touched**: None. REQUIREMENTS.md shows 0 active, 6 validated; the slice does not modify the contract of any validated requirement (`SwitchProfile` semantics are unchanged).
- **Re-verify**: None — no shipped requirement is affected.
- **Decisions revisited**: None.
- ## Proof Level
- This slice proves: integration (real disk write + real control-socket dispatch wired into the binary).
- Real runtime required: no for the automated suite (state-roundtrip uses a tempdir; `SwitchProfile` is fire-and-forget and skipped when the daemon is absent). Yes for manual UAT in S06.
- Human/UAT required: deferred to S06.
- ## Verification
- `cargo build` (no features) — exit 0, zero warnings.
- `cargo build --features gui` — exit 0, zero warnings.
- `RUSTFLAGS="-D warnings" cargo clippy --all-targets --features gui -- -D warnings` — exit 0.
- `cargo test --lib ui::pack_editor -- --test-threads=1` — all new pack_editor unit tests pass.
- `cargo test --test pack_editor_state_roundtrip -- --test-threads=1` — state-machine roundtrip integration test passes.
- `cargo test -- --test-threads=1` — full suite green (78 pre-existing tests + new tests; 1 ignored KWS test remains ignored).
- ## Observability / Diagnostics
- Runtime signals: `tracing::info!` on successful save (path + macro count), `tracing::warn!` when `SwitchProfile` send fails (daemon not running is logged at info, not warn).
- Inspection surfaces: the existing config-app log scroll already drains the `tracing` channel — the editor's save messages flow into it without new wiring. The on-disk `pack.yaml` is the persistent inspection surface.
- Failure visibility: `PackEditorState` exposes a `last_error: Option<String>` populated by failed CRUD attempts; the egui panel renders it as a colored label so the user sees parse/duplicate/validation errors inline.
- Redaction constraints: none — pack.yaml has no secrets.
- ## Integration Closure
- Upstream surfaces consumed: `crate::pack::{Pack, PackEditor, MacroUpdates, get_profiles_dir}`, `crate::config::{MacroConfig, KeyAction}`, `crate::control::client::send_command`, `crate::control::protocol::ControlRequest::SwitchProfile`.
- New wiring introduced in this slice: `PackEditorState` field on `VibeAttackConfigApp`, profile-list click handler that loads the pack, `show_pack_editor(ui, state)` call inside `show_main_config`, save path that pipes through `Pack::save_to_dir` then `send_command(SwitchProfile)`.
- What remains before the milestone is truly usable end-to-end: S04 (Import/Export dialogs), S05 (TriggerMacro Test button), S06 (full UAT).

## Proof Level

- This slice proves: integration

## Integration Closure

Wires the S02 PackEditor API into the binary entry point: profile-list click → load pack → mutate via PackEditorState CRUD → save_to_dir → SwitchProfile dispatch. After this slice, the user can complete a full edit cycle end-to-end through the GUI; only file-picker import/export (S04) and the Test button (S05) remain before the milestone is truly demo-ready.

## Verification

- Editor save logs structured `tracing::info!` events (path, macro_count); daemon notification path warns on send failure but not on daemon-absent. PackEditorState carries a `last_error` field rendered inline in the panel for synchronous failure visibility. No new persistent state files beyond the pack.yaml the user already owns.

## Tasks

- [x] **T01: Scaffold pack_editor module and wire PackEditorState into VibeAttackConfigApp** `est:1h`
  Create `src/ui/pack_editor.rs` following the exact `#[cfg(feature = "gui")] mod inner { ... } pub use inner::*;` pattern used in `src/ui/wizard.rs`. Inside `inner`, define a `PackEditorState` struct that owns: `editor: PackEditor`, `profile_dir: PathBuf`, `selected_category: Option<String>`, `selected_macro: Option<String>`, transient form-state strings (`form_name: String`, `form_phrase: String`, `form_if_flag: String`, `form_set_flag: String`, `form_keys: String`), `show_rename_warning: bool`, and `last_error: Option<String>`. Expose constructors `PackEditorState::new(editor: PackEditor, profile_dir: PathBuf) -> Self` and a stub `show_pack_editor(ui: &mut egui::Ui, state: &mut PackEditorState)` that for now renders only a placeholder heading + the loaded pack name. Implement a pure helper `parse_key_sequence(input: &str) -> anyhow::Result<Vec<KeyAction>>` (comma-separated; trims whitespace; each token becomes `KeyAction { key: token.to_string(), dwell_ms: None, gap_ms: None }`; rejects empty input with a clear error). Add `pub mod pack_editor;` (with `#[cfg(feature = "gui")]` matching `tray`) to `src/ui/mod.rs`. In `src/bin/vibe-attack-config.rs`, add `pack_editor: Option<vibe_attack::ui::pack_editor::PackEditorState>` to `VibeAttackConfigApp` (initialized to `None`); modify the profiles-list loop in `show_main_config` so clicking a profile name loads `Pack::load_from_dir(get_profiles_dir()?.join(name))`, wraps it in `PackEditor::new`, and stores `Some(PackEditorState::new(editor, profile_dir))` in `app.pack_editor`. Below the profiles list, if `app.pack_editor.is_some()`, call `show_pack_editor(ui, app.pack_editor.as_mut().unwrap())`. Add unit tests under `pack_editor::tests`: `parse_key_sequence_single`, `parse_key_sequence_multiple_with_whitespace`, `parse_key_sequence_empty_errors`, `parse_key_sequence_trailing_comma`. CRITICAL: verify `cargo build` (no features) compiles clean BEFORE adding any further egui code — any leaked `eframe::egui` import outside the `#[cfg(feature = "gui")]` gate will break the default build.
  - Files: `src/ui/pack_editor.rs`, `src/ui/mod.rs`, `src/bin/vibe-attack-config.rs`
  - Verify: cargo build && cargo build --features gui && cargo test --lib ui::pack_editor -- --test-threads=1

- [x] **T02: Implement egui editor panel: category/macro lists, edit form, CRUD buttons, Save with SwitchProfile dispatch** `est:2h`
  Flesh out `show_pack_editor` in `src/ui/pack_editor.rs` (inside the `#[cfg(feature = "gui")]` `inner` module) to render the full editor UX. Layout: use `egui::SidePanel` or three `egui::ScrollArea` columns inside a `ui.horizontal` — left column = category list (`selectable_label` per category, click sets `state.selected_category`); middle column = macro list for the selected category (`selectable_label` per macro, click populates the form fields by reading from the underlying `MacroConfig`); right column = the edit form. Form fields: `TextEdit::singleline` for name, phrase, if_flag, set_flag, and the comma-separated keys string; for the selected macro's keys, render each `KeyAction`'s optional `dwell_ms`/`gap_ms` as `DragValue<u64>` rows with a checkbox to enable the override. Buttons in the form area: `Add Macro` (calls `state.editor.add_macro(category, MacroConfig { ... })` built from form state via a private `build_macro_config_from_form` helper that calls `parse_key_sequence`); `Update Macro` (calls `edit_macro` with a `MacroUpdates` built using the `Some(Some(_))` / `Some(None)` semantics from MEM055 — `Some(Some(s))` for non-empty fields, `Some(None)` to explicitly clear); `Remove Macro` (with a small inline confirmation: a follow-up confirm button that appears after first click). Buttons in a toolbar above the category list: `Add Category` (with a singleline TextEdit for the new name), `Remove Category` (refused by `PackEditor` if non-empty — surface the error in `state.last_error`), `Rename Category` (when invoked, set `state.show_rename_warning = true` and render a yellow `Color32::YELLOW` colored_label warning that `if_flag`/`set_flag` macro references are NOT cascaded; user must confirm via a second click before the rename is applied). Bottom of panel: a `Save` button that calls a new method `PackEditorState::save(&self) -> anyhow::Result<()>` which performs `self.editor.pack().save_to_dir(&self.profile_dir)` then `send_command(ControlRequest::SwitchProfile { name: self.editor.pack().name.clone() })` (the SwitchProfile error is logged via `tracing::warn!` if the daemon is absent or returns Err, but the save itself succeeds — return Ok if the disk write succeeded). Render `state.last_error` as a `Color32::RED` colored_label whenever it's `Some`. After every CRUD call, wipe `state.last_error` to `None` on success or set it to the error's `to_string()` on failure. Borrow-checker note: when iterating `state.editor.pack().categories` inside an egui closure, follow the existing `app.device_names.clone()` precedent — clone `Vec<String>` of category and macro names BEFORE the egui closure to avoid simultaneous mutable borrows of `state`. Add unit tests under `pack_editor::tests` for pure-logic helpers: `build_macro_config_from_form_minimal_fields`, `build_macro_config_from_form_clears_optional_when_empty`, `build_macro_config_from_form_propagates_key_parse_error`. Form-state assembly must NOT call any egui function — it must be testable from a pure unit test.
  - Files: `src/ui/pack_editor.rs`
  - Verify: RUSTFLAGS="-D warnings" cargo clippy --all-targets --features gui -- -D warnings && cargo test --lib ui::pack_editor -- --test-threads=1

- [x] **T03: Add state-machine roundtrip integration test and verify full slice acceptance** `est:1h`
  Create `tests/pack_editor_state_roundtrip.rs` that proves the editor state machine performs an end-to-end edit → save → reload → equivalence cycle. Reuse the helper-shape pattern from `tests/pack_editor_roundtrip.rs` (S02): define a private `fn assert_packs_equal(left: &Pack, right: &Pack)` that walks `name`, `author`, `categories.len()`, then for each category walks `name`, `macros.len()`, then for each macro walks every `MacroConfig` field (name, phrase, if_flag, set_flag, sound, keys with KeyAction.key/dwell_ms/gap_ms). Define a `fn fixture_pack() -> Pack` with two categories and two macros each, mirroring the S02 fixture. Tests required:
- `state_roundtrip_after_full_crud_via_state_layer`: build a fixture Pack, wrap in `PackEditor::new`, then `PackEditorState::new(editor, tempdir.path().join("hd2").to_path_buf())`. Drive the SAME mutation calls the UI buttons would issue: `state.editor.add_category("NewCat")`, `state.editor.add_macro("NewCat", ...)`, `state.editor.edit_macro("NewCat", "m1", MacroUpdates { phrase: Some(Some("new phrase".into())), ..Default::default() })`, `state.editor.move_macro("Cat1", "NewCat", "orig_macro")`, `state.editor.rename_category("Cat2", "Cat2Renamed")`, `state.editor.remove_macro("NewCat", "orig_macro")`, `state.editor.remove_category("Cat1")`. Then `std::fs::create_dir_all(&profile_dir)?` and `state.editor.pack().save_to_dir(&profile_dir)?`. Reload via `Pack::load_from_dir(&profile_dir)?` and call `assert_packs_equal(state.editor.pack(), &reloaded)`.
- `state_parse_key_sequence_drives_form_to_save`: use the public `parse_key_sequence` helper to build a Vec<KeyAction> from a comma-separated string, construct a MacroConfig with those keys, add it via the editor, save to a fresh tempdir, reload, and assert the macro's `keys` survive byte-for-byte (key string + None dwell + None gap).
- `state_save_to_dir_writes_pack_yaml`: simple smoke test asserting that after `state.editor.pack().save_to_dir(tempdir)`, `tempdir.join("pack.yaml")` exists and `serde_yaml_ng::from_str::<Pack>(&fs::read_to_string(...))` round-trips.
After writing the test file, run the full slice acceptance suite and confirm all green: `cargo build` (no features), `cargo build --features gui`, `RUSTFLAGS="-D warnings" cargo clippy --all-targets --features gui -- -D warnings`, `cargo test -- --test-threads=1`. Address any clippy lints surfaced by T01/T02 code that escaped earlier verification. The task is complete only when the full `cargo test` reports the prior 78 tests still passing PLUS the new unit and integration tests, with zero warnings under the strict RUSTFLAGS. Do NOT exercise SwitchProfile in the integration test (no daemon is running in CI); the save-to-disk half is the integration-closure proof for this slice.
  - Files: `tests/pack_editor_state_roundtrip.rs`
  - Verify: cargo build && cargo build --features gui && RUSTFLAGS="-D warnings" cargo clippy --all-targets --features gui -- -D warnings && cargo test -- --test-threads=1

## Files Likely Touched

- src/ui/pack_editor.rs
- src/ui/mod.rs
- src/bin/vibe-attack-config.rs
- tests/pack_editor_state_roundtrip.rs
