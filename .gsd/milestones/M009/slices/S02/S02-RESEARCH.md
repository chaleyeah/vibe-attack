# S02: PackEditor pure-logic state + CRUD — Research

**Date:** 2026-04-27

## Summary

The codebase already has a solid Pack/Category/Macro type hierarchy serialized via serde_yaml_ng and exercised by 22 integration tests. The MacroEditor widget in `src/tui/editor.rs` is currently read-only (renders a bordered pane with phrase, condition flag, and key sequence). S02 requires building a pure-logic `PackEditor` struct in `src/pack/mod.rs` with typed CRUD operations (AddMacro, EditMacro, RemoveMacro, MoveMacro, RenameCategory, AddCategory, RemoveCategory), along with unit tests for each operation plus a round-trip test that verifies byte-equivalence after edit → save → load → serialize.

The existing test infrastructure (`tests/pack_hd2_bundle.rs`) shows that tests use tempdir, redirect XDG_CONFIG_HOME to stay hermetic, and use `#[serial]` for tests that modify environment. The existing Pack type is immutable — all mutations must flow through the PackEditor API. No changes to serde format are needed; the YAML structure remains `name/author/categories[]/name/macros[]`.

## Recommendation

Structure PackEditor as a mutable wrapper around Pack that exposes typed command methods, each returning Result<()>. Store internal state as `pack: Pack` plus optional tracking fields (e.g., for undo, or selection state for future UI binding). Implement each CRUD op as a method that mutates the Pack in place and can be unit tested in isolation. Add a comprehensive test module in `src/pack/mod.rs` (or a sibling `tests/pack_editor_crud.rs`) that covers:

- **AddMacro** — add a new MacroConfig to a named Category; error on missing category
- **EditMacro** — update fields (name, phrase, if_flag, set_flag, keys) of a macro in place; error on missing macro
- **RemoveMacro** — remove a macro from a category by name; error on missing macro
- **MoveMacro** — move a macro between categories; error on missing macro or target category
- **RenameCategory** — rename a category; preserve order and macro contents; error on duplicate name
- **AddCategory** — insert a new empty Category at the end; error on duplicate name
- **RemoveCategory** — remove a category if empty; error if not empty or missing

For round-trip testing, serialize a modified Pack to YAML, deserialize it, and compare the byte-for-byte result of re-serializing. Use the `hd2_pack()` fixture from `pack_hd2_bundle.rs` as a base, apply a series of CRUD ops, save-load, and assert byte equivalence and structural integrity.

## Implementation Landscape

### Key Files

- **`src/pack/mod.rs`** — Currently defines `Pack`, `Category`, `MacroConfig` (via serde from `config.rs`), plus `load_from_dir()`, `save_to_dir()`, `import()` (ZIP), `export()` (ZIP), and `flatten()`. Also has 2 basic tests (`test_pack_save_load`, `test_pack_export_import_with_sounds`). This is where PackEditor will live.

- **`src/pack/manager.rs`** — Manages `ProfileManager` (active profile name persistence in manager.yaml). Not touched by PackEditor itself, but ProfileManager::get_active_pack() will load a pack and pass it to the editor for editing.

- **`src/config.rs`** — Defines `MacroConfig { name, phrase, if_flag, set_flag, sound, keys }` and `KeyAction { key, dwell_ms, gap_ms }`. These are immutable once deserialized, so all mutations happen through PackEditor.

- **`tests/pack_hd2_bundle.rs`** — 22 integration tests covering Pack serialization, Category/Macro preservation, flatten, export/import (ZIP), ProfileManager persistence, and full lifecycle. Tests use `tempfile::tempdir()`, redirect `XDG_CONFIG_HOME` via `std::env::set_var()`, and use `#[serial]` to avoid concurrency issues.

- **`src/tui/editor.rs`** — Currently a read-only `MacroEditor` widget that renders a macro's phrase, condition flag, and key sequence using ratatui. This UI widget will consume the PackEditor pure-logic state in S03 (Egui editor panel), not in this slice.

### Build Order

1. **Define PackEditor struct** in `src/pack/mod.rs` with `pack: Pack` field and optional `selection` field for future UI binding. Implement `new(pack: Pack) -> Self`.

2. **Implement AddMacro** — `pub fn add_macro(&mut self, category: &str, macro_config: MacroConfig) -> Result<()>`. Validate category exists, add to `pack.categories[..].macros`, return Ok or descriptive error.

3. **Implement EditMacro** — `pub fn edit_macro(&mut self, category: &str, macro_name: &str, updates: MacroUpdates) -> Result<()>` where `MacroUpdates` is a struct holding only the fields to change (e.g., phrase, keys). Locate macro by name within the category, apply updates in place, return error on not found.

4. **Implement RemoveMacro** — `pub fn remove_macro(&mut self, category: &str, macro_name: &str) -> Result<()>`. Find macro, remove from vec, return Ok or error if not found.

5. **Implement MoveMacro** — `pub fn move_macro(&mut self, from_category: &str, to_category: &str, macro_name: &str) -> Result<()>`. Remove from source, add to dest, validate both exist.

6. **Implement RenameCategory** — `pub fn rename_category(&mut self, old_name: &str, new_name: &str) -> Result<()>`. Find category by old_name, check new_name not in use, update in place.

7. **Implement AddCategory** — `pub fn add_category(&mut self, name: &str) -> Result<()>`. Check not in use, push empty Category.

8. **Implement RemoveCategory** — `pub fn remove_category(&mut self, name: &str) -> Result<()>`. Check exists and is empty, remove.

9. **Implement get_pack()** — `pub fn get_pack(&self) -> &Pack` to extract the Pack for saving via Pack::save_to_dir().

10. **Add comprehensive unit tests** in `src/pack/mod.rs::tests` (or in a new `tests/pack_editor_crud.rs`) covering each CRUD op with both success and error cases.

11. **Add round-trip test** — Create a Pack, edit via PackEditor, save to temp dir, load, re-serialize, and assert byte-for-byte equality.

### Verification Approach

```bash
# Run all pack tests
cargo test --lib pack --all-targets

# Run integration tests specifically
cargo test --test pack_hd2_bundle

# Run new CRUD tests once added
cargo test --lib pack::tests

# Full test suite to catch any regressions
cargo test --all-targets
```

Expected: All tests pass; no new warnings. The round-trip test should confirm that editing a pack and saving produces the exact same YAML bytes as the original after deserialization.

## Constraints

- **Serde format is immutable** — No changes to Pack/Category/MacroConfig serde schema. The YAML structure must remain unchanged.
- **MacroConfig is borrowed from config.rs** — MacroConfig is defined in `config.rs` and re-used by Pack; any struct changes affect both. Prefer wrapper types (e.g., `MacroUpdates`) to avoid touching the canonical MacroConfig struct.
- **XDG and temp directories** — Tests must use `tempfile::tempdir()` and redirect `XDG_CONFIG_HOME` to stay hermetic. Tests that modify environment use `#[serial]` to avoid race conditions.
- **Existing tests must not break** — The 22 pack_hd2_bundle tests exercise Pack::load_from_dir, flatten, export, import, ProfileManager. PackEditor methods must not alter their behavior.
- **Round-trip byte equivalence** — The serde_yaml_ng crate produces deterministic output *within a single run* but field ordering and formatting may shift. The test should round-trip (YAML → Rust → YAML) and verify the deserialized Pack is structurally identical, then compare byte-for-byte after re-serialization. Alternatively, deserialize both original and edited packs and deep-compare them.

## Common Pitfalls

- **Macro name uniqueness within a category** — The code does not currently enforce unique names within a category. CRUD ops should check for duplicates when adding/editing and return a clear error.
- **Category order preservation** — Vec order matters for the YAML output. MoveMacro and AddCategory must preserve order.
- **Option<String> fields** — phrase, if_flag, set_flag, and sound are optional. CRUD ops must handle None gracefully and not panic on None comparisons.
- **KeyAction vec is mutable** — EditMacro must allow replacing the entire keys vec. Prefer consuming the old vec and pushing a new one rather than trying to edit in place.
- **Test flakiness** — `test_pack_export_import_with_sounds` has a known flake (MEM005): it fails under parallel cargo test due to tmpdir pollution. Run the full suite with `--test-threads=1` on this project to get a clean signal.
- **Serde field ordering** — serde_yaml_ng respects the struct field order in the source code. If MacroConfig fields are reordered in the future, the YAML output will differ (breaking byte-for-byte round-trip tests). Document this constraint.
