---
estimated_steps: 15
estimated_files: 2
skills_used: []
---

# T04: Wire Import / Export buttons into show_pack_editor and refresh profiles on import

Add `Import Pack` and `Export Pack` buttons to the egui pack editor toolbar inside `show_pack_editor`. On click, open `rfd::FileDialog` with a `.hdpack` filter. Handle cancel as a no-op, success by calling the relevant Pack method, and surface failures via `state.last_error` (the existing inline pattern from S03/MEM063). After a successful import, signal the outer `show_main_config` so it can call `load_profiles()` and replace `app.pack_editor` with one pointing at the just-imported pack.

Why: this is the user-visible surface of the slice. Without it, the backend round-trip in T03 has no UI exposure and the slice goal/demo are not met.

Key constraints:
- All rfd imports MUST be inside the `#[cfg(feature = "gui")] mod inner { ... }` block in `src/ui/pack_editor.rs`. Default-feature build must remain untouched (this is verified by `cargo build` with no features still succeeding).
- Add a public enum at module scope (NOT inside the inner mod) named `PackEditorOutcome` with variants `None`, `Imported(String /* pack name */)`. Or, equivalently, add `pub imported_pack_name: Option<String>` to `PackEditorState` and have the caller drain it each frame. Pick whichever is simpler given the existing borrow patterns in the file — but document the choice in the SUMMARY at slice complete time.
- Buttons go in the existing category toolbar `ui.horizontal` block (just after the Add Category / Remove Category / Rename Category cluster) so they are always visible regardless of selection state. Order: `[Import Pack]  [Export Pack]`.
- Import Pack flow: `rfd::FileDialog::new().add_filter("Pack", &["hdpack"]).pick_file()` → `Some(path)` → resolve `profiles_dir = get_profiles_dir()?` → call `Pack::import_to(&path, &profiles_dir)` → on Ok(pack), set `state.last_error = None`, signal the outcome (set the outcome flag), and update `state.editor` and `state.profile_dir` to point at the new pack so the editor immediately reflects the imported content; on Err(e), set `state.last_error = Some(e.to_string())`.
- Export Pack flow: `rfd::FileDialog::new().add_filter("Pack", &["hdpack"]).set_file_name("<pack_name>.hdpack").save_file()` → `Some(path)` → call `state.editor.pack().export(&state.profile_dir, &path)` → on Ok, log and clear last_error; on Err(e), set last_error.
- DO NOT block or warn on existing-profile collision in this slice — `import_to` already handles collisions by removing the existing dir, matching the established `Pack::import` contract. (Surfacing a confirmation prompt is a nice-to-have but increases scope; the milestone roadmap does not require it for S04, and S06 UAT will cover the user-visible behaviour.) If the executor finds the collision-destruction surprise blocking, document it in the SUMMARY and file a follow-up.
- After T04, in `src/bin/vibe-attack-config.rs` `show_main_config`, after the `show_pack_editor(ui, editor_state)` call, check the outcome flag. On `Imported(name)`: call `app.config.profiles = load_profiles()` (existing helper) so the profile list refreshes; the editor itself was already swapped to the new pack inside T04's import handler, so no extra work is needed there.
- Add tracing::info! on successful import (zip_path, pack_name, macro_count) and successful export (dest_path, pack_name).
- rfd dialogs block the calling thread. This is acceptable on the egui frame thread (research-confirmed) and matches all egui-rfd integrations. Do NOT spawn a thread or use rfd's async API — synchronous is correct here.

Threat surface (Q3): The dialog returns a user-chosen path. `Pack::import_to` already extracts via `enclosed_name()` which prevents zip path traversal; do not bypass that. No auth boundary touched. Input trust: a malicious .hdpack could in principle ship a giant pack.yaml; `serde_yaml_ng::from_str` will fail cleanly on a malformed doc and the failure surfaces in `last_error`. No new attack surface beyond what `Pack::import` already exposed.
Failure modes (Q5): user cancels (None) → no-op; rfd portal unavailable on Linux → returns None silently (treated as cancel); zip parse error → error surfaces in last_error; XDG dir unwritable → error surfaces in last_error.
Negative tests (Q7): not added at the unit-test layer (rfd cannot be driven headlessly); negative paths are exercised through Pack::import_to's existing error contract proven by T02 and T03.

## Inputs

- ``Cargo.toml``
- ``src/ui/pack_editor.rs``
- ``src/bin/vibe-attack-config.rs``
- ``src/pack/mod.rs``

## Expected Output

- ``src/ui/pack_editor.rs``
- ``src/bin/vibe-attack-config.rs``

## Verification

cargo build --features gui — must compile clean with zero warnings; cargo build (no features) — must still compile clean (proves rfd is gated correctly); cargo test -- --test-threads=1 — full suite stays green; grep for 'rfd::FileDialog' in src/ui/pack_editor.rs returns >=2 matches (one Import, one Export).
