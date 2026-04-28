# S04: Import / Export dialogs

**Goal:** Wire Import / Export pack dialogs into the egui pack editor: rfd-driven file pickers for both directions, refactor Pack::import to support a destination override (Pack::import_to) so a hermetic round-trip integration test can be written without XDG_CONFIG_HOME mutation, and refresh the in-app profiles list on successful import.
**Demo:** User clicks Import → picks a .hdpack → pack appears in profile list; user clicks Export → picks a destination → .hdpack written; round-trip import of exported file produces byte-identical macros

## Must-Haves

- cargo build --features gui passes with zero warnings; cargo test --test pack_lifecycle passes; cargo test -- --test-threads=1 passes (full suite stays green); show_pack_editor renders Import Pack and Export Pack buttons; successful Import refreshes the profile list and selects the imported pack; successful Export writes a .hdpack file at the user-chosen path; round-trip Import-of-Exported-pack produces field-equivalent macros.

## Proof Level

- This slice proves: integration — slice proves backend round-trip via integration test plus UI wiring compiles under the gui feature; rfd dialogs cannot be driven headlessly so the dialog-trigger behaviour itself is verified by manual smoke (documented in S06 UAT), not automated.

## Integration Closure

Upstream surfaces consumed: Pack::import / Pack::export / Pack::load_from_dir (src/pack/mod.rs); PackEditorState (src/ui/pack_editor.rs); load_profiles + show_main_config profiles list (src/bin/vibe-attack-config.rs).
New wiring introduced in this slice: rfd added under gui feature; show_pack_editor returns a PackEditorOutcome enum (or sets a flag on PackEditorState) so vibe-attack-config.rs can call load_profiles() after a successful import; import_to() backend variant.
What remains before the milestone is truly usable end-to-end: S05 (TriggerMacro Test button) and S06 (manual UAT covering full pack lifecycle).

## Verification

- Runtime signals: tracing::info! on successful export (path, macro_count); tracing::info! on successful import (source_zip, dest_dir, macro_count); tracing::warn! on import/export failure with reason. Inspection surfaces: PackEditorState.last_error renders the failure reason inline as Color32::RED (existing pattern from S03/MEM063); the on-disk .hdpack file is the export inspection surface; the imported profile dir under XDG_CONFIG_HOME/vibe-attack/profiles/&lt;name&gt; is the import inspection surface. Failure visibility: last_error set with anyhow::Error::to_string() for synchronous failures; rfd cancel returns None and is treated as a no-op (no error). Redaction constraints: none — pack.yaml contains user-authored macros, no secrets.

## Tasks

- [x] **T01: Add rfd 0.17 dependency under gui feature** `est:10m`
  Add the rfd (Rusty File Dialog) crate to Cargo.toml under the `gui` feature so the editor panel can open native file pickers. Confirm both default and gui builds compile cleanly. This is a dependency-only change — no code changes yet.

Why: rfd is the standard egui-ecosystem file picker. It calls the native portal on Wayland and zenity/kdialog on X11. Without it, the next two tasks cannot reference rfd::FileDialog. Adding it as optional + listed under the `gui` feature keeps the default (no-features) build untouched.

Key constraints:
- rfd MUST be added with `optional = true` and listed under the `gui` feature dep array. Do not add it to default deps — that would force the file-dialog backend into the headless daemon binary.
- Use rfd version `0.17` (research-confirmed current stable; 0.17.2 is fine).
- Do not import rfd anywhere yet — that happens in T04. This task only touches Cargo.toml.
- Distribution targets are Debian, Red Hat, and Arch (per project memory); rfd's xdg-portal backend covers all three. No extra system deps required at build time.
  - Files: `Cargo.toml`
  - Verify: cargo build && cargo build --features gui — both must exit 0 with zero rustc warnings; cargo metadata --format-version 1 | grep -q '"name":"rfd"' to confirm the crate is in the dependency graph.

- [x] **T02: Add Pack::import_to(zip_path, dest_dir) and make Pack::import delegate** `est:30m`
  Refactor `Pack::import` so the destination directory is injectable. Add a new public `Pack::import_to(zip_path: &Path, dest_dir: &Path) -> Result<Pack>` that contains the actual extraction logic, then make the existing `Pack::import(zip_path)` a thin wrapper that resolves `get_profiles_dir()?.join(&pack.name)` and delegates to `import_to`. Also add a unit test for `import_to` against a tempdir to lock the behaviour in.

Why: the existing `Pack::import` writes unconditionally to `get_profiles_dir()` (XDG_CONFIG_HOME). MEM005 records that the existing parallel test for export/import is already flaky under shared XDG mutation. To write a reliable round-trip integration test in T03, the import side must accept a destination path so the test can pass a tempdir. Refactoring `import` to delegate keeps backwards compatibility — all existing call sites (and existing tests using `XDG_CONFIG_HOME` env override) remain green.

Key constraints:
- Public API: `pub fn import_to(zip_path: &Path, dest_dir: &Path) -> Result<Pack>`. Signature must accept `&Path` for both args (matches existing `import` and `export` style).
- `import_to` MUST extract into `dest_dir.join(&pack.name)` — i.e. the caller passes the *parent* profiles directory, and the function appends the pack name itself. This matches the existing `import()` semantics (extracts to `get_profiles_dir()?.join(&pack.name)`).
- Preserve the path-traversal protection (`file.enclosed_name()`).
- Preserve the existing collision behaviour: if `dest_dir.join(&pack.name)` already exists, `remove_dir_all` it first. This is what callers (UI and tests) expect.
- Existing `pub fn import(zip_path: &Path) -> Result<Self>` becomes a 3-line wrapper: `let profiles_dir = get_profiles_dir()?; Self::import_to(zip_path, &profiles_dir)`.
- Add tracing::info! at the start of `import_to` with `zip_path` and `dest_dir` fields; add tracing::info! at the end with `macro_count` after successful extraction.
- Add one inline unit test in `src/pack/mod.rs` `#[cfg(test)] mod tests` (or a fresh test module) that builds a small pack, exports to tempdir, calls `import_to(&zip, &tempdir2)`, and asserts the imported `Pack` name and one macro name match. This ensures `import_to` is exercised even before T03 lands.

Failure modes (Q5): zip not found → wraps existing io::Error path; missing pack.yaml inside zip → existing context message; dest_dir not writable → propagated io::Error.
Negative tests (Q7): the unit test above covers the happy path; negative cases (malformed zip, missing pack.yaml) are already covered by existing tests that exercise `import` and will exercise the same code path through delegation.
  - Files: `src/pack/mod.rs`
  - Verify: cargo test --lib pack:: -- --test-threads=1 — the new import_to unit test plus all existing pack tests must pass; cargo test --test pack_hd2_bundle -- --test-threads=1 — existing serial export/import tests must stay green (they exercise import() via the wrapper path).

- [x] **T03: Write tests/pack_lifecycle.rs round-trip integration test** `est:45m`
  Add a new integration test file `tests/pack_lifecycle.rs` that proves the export → import round-trip is byte/field-equivalent for a non-trivial fixture pack. Use `Pack::import_to()` so the test does not mutate `XDG_CONFIG_HOME` (and is therefore not subject to MEM005-style parallel flake).

Why: the milestone success criterion includes `Export → import round-trip produces byte-identical macro content — hermetic test in tests/pack_lifecycle.rs`. This is the slice's primary objective stopping condition for the backend — once it passes, the UI wiring in T04 only adds dialog plumbing on top of a verified contract.

Key constraints:
- File path: `tests/pack_lifecycle.rs` (the milestone roadmap names this exact path; do not pick a different name).
- Build at least 2 tests:
  1. `pack_export_then_import_to_round_trips_macros` — build a fixture Pack with multiple categories and macros that exercise every MacroConfig field (phrase, if_flag, set_flag, sound: None, keys with and without dwell_ms/gap_ms overrides). Save it to a source tempdir. Call `pack.export(source_dir, &zip_path)`. Call `Pack::import_to(&zip_path, &dest_tempdir)`. Reload the imported pack from `dest_tempdir.join(&pack.name)` via `Pack::load_from_dir`. Assert the reloaded pack has identical: name, author, category count, category names in order, macro counts, macro names in order, every MacroConfig field of every macro.
  2. `pack_export_imports_sounds_subdirectory` — write a small dummy file under `source_dir/sounds/test.wav`, export, import_to, then assert `dest_dir/<pack_name>/sounds/test.wav` exists and has the same byte content. This locks in the sounds-bundling behaviour (already in `Pack::export`'s add_dir_to_zip) at the integration level.
- Use `tempfile::tempdir()` for both source and dest dirs — no shared state.
- Do NOT mutate XDG_CONFIG_HOME and do NOT use `#[serial]`. The whole point of using `import_to` is that the test stays hermetic and parallel-safe.
- Use the helper style of `tests/pack_hd2_bundle.rs` for fixture builders (`fn key`, `fn macro_simple`) — copy them inline if needed (these are tiny). Do NOT add a shared test util crate for two functions.
- The fixture Pack name MUST NOT collide with any name used by a test that runs in the same suite under env-var mutation — pick something obviously test-local like `LifecycleRoundTripFixture`.
- Since the test reads ONLY paths created inside its own tempdirs, it does not depend on any tracked fixture file. (Reminder from auto-mode: planned tests must only read from tracked or test-created paths — this test creates its own.)

Failure modes (Q5): zip write failure → io::Error propagated, test panics with clear message; field mismatch → assert_eq! prints expected vs actual.
Negative tests (Q7): not required for this slice — malformed zip handling is already covered by existing pack_hd2_bundle.rs tests.
  - Files: `tests/pack_lifecycle.rs`
  - Verify: cargo test --test pack_lifecycle -- --test-threads=1 — both tests must pass; cargo test -- --test-threads=1 — full suite stays green.

- [ ] **T04: Wire Import / Export buttons into show_pack_editor and refresh profiles on import** `est:1h`
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
  - Files: `src/ui/pack_editor.rs`, `src/bin/vibe-attack-config.rs`
  - Verify: cargo build --features gui — must compile clean with zero warnings; cargo build (no features) — must still compile clean (proves rfd is gated correctly); cargo test -- --test-threads=1 — full suite stays green; grep for 'rfd::FileDialog' in src/ui/pack_editor.rs returns >=2 matches (one Import, one Export).

## Files Likely Touched

- Cargo.toml
- src/pack/mod.rs
- tests/pack_lifecycle.rs
- src/ui/pack_editor.rs
- src/bin/vibe-attack-config.rs
