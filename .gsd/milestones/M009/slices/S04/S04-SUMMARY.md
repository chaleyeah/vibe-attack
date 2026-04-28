---
id: S04
parent: M009
milestone: M009
provides:
  - ["Pack::import_to(zip_path, dest_dir) hermetic backend API", "tests/pack_lifecycle.rs round-trip integration tests", "Import Pack / Export Pack buttons in egui editor toolbar", "Profile list refresh on successful import"]
requires:
  []
affects:
  []
key_files:
  - (none)
key_decisions:
  - ["rfd 0.17 added as optional dep under gui feature only — keeps default/headless build free of file-dialog backend", "Pack::import_to accepts parent profiles dir (not pack subdir) — function appends pack.name internally, matching original import() contract", "Import outcome signaled via PackEditorState.imported_pack_name: Option<String> drained each frame — avoids borrow-pattern complexity from enum return through egui call stack", "rfd dialogs called synchronously on egui frame thread — confirmed acceptable by ecosystem convention; async API not used", "No collision confirmation prompt for Import — import_to already removes existing dir matching established contract; prompt deferred to future enhancement"]
patterns_established:
  - ["Hermetic pack round-trip tests use Pack::import_to + tempfile::tempdir() — never XDG_CONFIG_HOME mutation, no #[serial] needed", "rfd imports must live inside #[cfg(feature = \"gui\")] mod inner block in src/ui/pack_editor.rs", "Editor outcome signaling via Option<String> state field drained with .take() each frame (established alongside S03's last_error pattern)"]
observability_surfaces:
  - ["tracing::info! on successful Import Pack (zip_path, pack_name, macro_count)", "tracing::info! on successful Export Pack (dest_path, pack_name)", "tracing::warn! on Import Pack failure (reason)", "tracing::warn! on Export Pack failure (reason)", "PackEditorState.last_error renders failure reason inline as Color32::RED in the editor toolbar"]
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-28T03:09:51.084Z
blocker_discovered: false
---

# S04: Import / Export dialogs

**Wired rfd-driven Import Pack and Export Pack buttons into the egui editor; refactored Pack::import into import_to for hermetic testing; two round-trip integration tests in tests/pack_lifecycle.rs pass cleanly.**

## What Happened

S04 delivered the full import/export surface of the pack editor in four tasks.

**T01** added `rfd = { version = "0.17", optional = true }` to Cargo.toml under the `gui` feature. Cargo resolved rfd 0.17.2 with transitive dep pollster 0.4.0. Both the default build (no features) and the gui build compiled with zero warnings, confirming rfd is correctly gated.

**T02** extracted `Pack::import`'s extraction logic into a new public `Pack::import_to(zip_path: &Path, dest_dir: &Path) -> Result<Pack>`. The function accepts the parent profiles directory and appends `pack.name` internally, preserving the original semantics: path-traversal protection via `enclosed_name()`, collision handling via `remove_dir_all`, and `create_dir_all` for directory entries. Tracing events were added at entry (zip_path, dest_dir) and on success (macro_count). The original `Pack::import` became a 3-line wrapper resolving `get_profiles_dir()` and delegating. A new inline unit test `test_import_to_extracts_into_dest_dir` verified the happy path without any XDG mutation. All 28 lib pack tests and 22 pack_hd2_bundle tests passed.

**T03** added `tests/pack_lifecycle.rs` with two hermetic integration tests using `tempfile::tempdir()` throughout — no XDG_CONFIG_HOME mutation, no `#[serial]`. `pack_export_then_import_to_round_trips_macros` builds a fixture Pack (`LifecycleRoundTripFixture`) with multiple categories and macros exercising all MacroConfig fields, exports to a tempdir zip, imports via `import_to` into a second tempdir, reloads via `Pack::load_from_dir`, and performs deep per-field assertions on every macro. `pack_export_imports_sounds_subdirectory` writes a dummy wav, exports, imports, and asserts the file is present with identical byte content in the destination. Both tests passed in 0.00s.

**T04** wired `Import Pack` and `Export Pack` buttons into `show_pack_editor`'s category toolbar (after Add/Remove/Rename Category). All rfd imports are inside the `#[cfg(feature = "gui")] mod inner` block. Import flow: `rfd::FileDialog::new().add_filter("Pack", &["hdpack"]).pick_file()` → `Pack::import_to` → on Ok, swaps `state.editor` and `state.profile_dir` to the new pack and sets `state.imported_pack_name = Some(pack_name)`. Export flow: `rfd::FileDialog::new().add_filter("Pack", &["hdpack"]).set_file_name(...).save_file()` → `pack.export(...)`. Failures surface via `state.last_error` (existing inline red-text pattern from S03). In `vibe-attack-config.rs`, `show_main_config` drains `editor_state.imported_pack_name.take()` each frame and calls `load_profiles()` on Some, refreshing the profile list. Tracing info events log success for both directions. Cancel (None from rfd) is a no-op. rfd dialogs run synchronously on the egui frame thread — confirmed acceptable by ecosystem convention; async API not used.

## Verification

1. `cargo build` (default, no features) → exit 0, zero warnings — rfd gate confirmed.
2. `cargo build --features gui` → exit 0, zero warnings — gui feature compiles cleanly.
3. `cargo test --test pack_lifecycle -- --test-threads=1` → 2/2 tests pass (pack_export_then_import_to_round_trips_macros, pack_export_imports_sounds_subdirectory).
4. `cargo test -- --test-threads=1` → 202 tests pass, 0 failures, 3 ignored (privileged/hardware-only).
5. `grep -c 'rfd::FileDialog' src/ui/pack_editor.rs` → 1 match (inside cfg(feature="gui") block; both Import and Export share the module import).
6. `grep -n 'imported_pack_name\|load_profiles' src/bin/vibe-attack-config.rs` → confirms drain+refresh wiring at line 442-443.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

rfd dialog behavior cannot be driven headlessly — Import/Export dialog trigger paths are excluded from automated tests. TC-01 through TC-07 require manual smoke before S06 milestone UAT. No collision confirmation prompt: import silently replaces an existing profile with the same name (import_to removes the existing dir). This matches the established Pack::import contract and is documented; a confirmation prompt is a future enhancement.

## Follow-ups

None.

## Files Created/Modified

- `Cargo.toml` — 
- `src/pack/mod.rs` — 
- `tests/pack_lifecycle.rs` — 
- `src/ui/pack_editor.rs` — 
- `src/bin/vibe-attack-config.rs` — 
