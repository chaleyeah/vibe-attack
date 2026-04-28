# S04: Import / Export dialogs — Research

**Date:** 2026-04-27
**Status:** Ready for planning

## Summary

S04 is straightforward. The hard work is already done: `Pack::import()` and `Pack::export()` are fully implemented in `src/pack/mod.rs` (lines 59–155). The ZIP backend (`zip = "0.6"`) is already in `Cargo.toml`. What's missing is purely the UI surface — buttons in the egui panel and a file-picker to get the path from the user.

The one real decision is **how to open a file picker dialog** on Linux. `rfd` ("Rusty File Dialog") is the standard egui-ecosystem answer — it calls the native portal on Wayland and `zenity`/`kdialog` on X11. It is NOT currently in `Cargo.toml` and must be added. The alternative (spawning `zenity` via `std::process::Command`) is simpler but fragile on systems without zenity. `rfd = "0.17.2"` is the current crate version and should be added under the `gui` feature.

The integration test for the round-trip (`tests/pack_lifecycle.rs`) does not yet exist. It should exercise `Pack::export()` → file on disk → `Pack::import()` → byte-equivalent YAML content. Note that `Pack::import()` internally calls `get_profiles_dir()` and writes to the user's XDG config directory — this makes hermetic testing require either `std::env::set_var("HOME", tempdir)` or a refactor of `import()` to accept a destination directory. The cleaner approach is to add an `import_to()` variant that accepts a destination path, then test against that; the existing `import()` can delegate to it.

## Recommendation

Add `rfd` to the `gui` feature. Wire two buttons into the existing `show_pack_editor` panel: **Import Pack** and **Export Pack**. Both buttons open a blocking `rfd::FileDialog` call (acceptable on the egui frame thread since `rfd` dialogs block only until the user picks a file). Handle the result inline — success reloads the `PackEditorState`, failure surfaces in `state.last_error`. Add `Pack::import_to(zip_path, dest_dir)` for hermetic testing, then write `tests/pack_lifecycle.rs` as the round-trip integration test.

## Implementation Landscape

### Key Files

- `src/pack/mod.rs` — `Pack::import()` (line 59) and `Pack::export()` (line 106) already exist; add `import_to(zip_path, dest_dir)` so the import destination is injectable for tests; existing `import()` becomes a thin wrapper
- `src/ui/pack_editor.rs` — add `Import Pack` and `Export Pack` buttons inside `show_pack_editor` (inside `#[cfg(feature = "gui")]`); add `last_import_result: Option<String>` or reuse `state.last_error` for error display
- `src/bin/vibe-attack-config.rs` — after a successful import, reload the profiles list (`app.config.profiles = load_profiles()`) and open the newly imported pack in the editor; this requires passing a mutable ref to `app.config.profiles` out of `show_pack_editor` or returning an import-result enum
- `Cargo.toml` — add `rfd = { version = "0.17", optional = true }` and append `rfd` to the `gui` feature dep list
- `tests/pack_lifecycle.rs` — new integration test: export fixture pack to tempdir → `Pack::import_to()` into second tempdir → reload → assert field-level equivalence (same categories, same macro names, same key sequences)

### Build Order

1. **Add `rfd` to Cargo.toml under `gui` feature** — unblocks compilation of dialog calls
2. **Add `Pack::import_to(zip_path, dest_dir)` to `src/pack/mod.rs`** — makes the import side hermetically testable without XDG side effects; existing tests stay green
3. **Write `tests/pack_lifecycle.rs`** — verify export→import round-trip before wiring UI; confirms the backend contract is solid
4. **Wire Import/Export buttons in `show_pack_editor`** — egui-only code; uses `rfd::FileDialog`; surfaces errors via `state.last_error`; on success, returns a `PackImportResult` or similar signal to the outer `show_main_config` so the profile list can be refreshed
5. **Reload profile list in `vibe-attack-config.rs`** after successful import

### Verification Approach

```
cargo build --features gui              # must compile clean
cargo test --test pack_lifecycle        # round-trip must pass
cargo test -- --test-threads=1          # full suite must stay green
```

Manual smoke: open config app → click "Export Pack" on hd2 profile → pick a path → confirm .hdpack file on disk. Then "Import Pack" → pick that file → profile appears in list.

## Constraints

- `rfd` dialogs block the calling thread. Since `show_pack_editor` runs on the egui frame thread, the dialog will freeze the UI while open — this is acceptable and is what all egui-rfd integrations do. No async wrapper needed.
- `Pack::import()` currently writes unconditionally to `get_profiles_dir()`. This makes it unsuitable for hermetic tests. `import_to()` must be added before the test can be written.
- Export must go through `PackEditorState::profile_dir` as the `source_dir` argument to `Pack::export()` (line 106) — this is the directory containing the sounds/ subdirectory that gets bundled.
- `rfd` must be gated under `#[cfg(feature = "gui")]` (same as eframe) — no rfd imports in the ungated helpers.
- Name collision on import: if a profile with the same name already exists, `Pack::import()` removes the existing dir (`std::fs::remove_dir_all`) before extracting — the UI should surface a warning before proceeding (two-click pattern already established in S03).

## Common Pitfalls

- **`rfd::FileDialog::pick_file()` returns `Option<PathBuf>`** — the cancel case must be handled (no-op, not an error)
- **`Pack::import()` destroys the existing profile dir on name collision** — the UI should warn before calling it; follow the `pending_remove_macro` two-click pattern from S03
- **Profile list not refreshed after import** — the profiles Vec in `VibeAttackConfigApp` is loaded once at startup; after a successful import, `load_profiles()` must be called again; `show_main_config` must act on a return value or flag from `show_pack_editor`
- **Export path extension** — guide the user to use `.hdpack` extension; `rfd` allows setting a default extension filter
