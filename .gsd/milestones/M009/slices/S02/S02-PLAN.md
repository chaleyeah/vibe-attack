# S02: PackEditor pure-logic state + CRUD

**Goal:** Build a pure-logic `PackEditor` struct in `src/pack/mod.rs` that wraps a `Pack` and exposes typed CRUD methods (AddMacro, EditMacro, RemoveMacro, MoveMacro, RenameCategory, AddCategory, RemoveCategory), each unit-tested for both success and error cases. Add a hermetic round-trip integration test proving edit → save → reload → re-serialize produces structurally identical packs.
**Demo:** cargo test passes including new PackEditor unit tests for AddMacro, EditMacro, RemoveMacro, MoveMacro, RenameCategory, AddCategory, RemoveCategory; round-trip: edit → save → reload → byte equivalence

## Must-Haves

- cargo test passes including new PackEditor unit tests for AddMacro, EditMacro, RemoveMacro, MoveMacro, RenameCategory, AddCategory, RemoveCategory; round-trip: edit → save → reload → byte equivalence (after deserialize-and-deep-compare, the structural assertion holds).

## Proof Level

- This slice proves: contract — pure-logic state machine; no runtime/UI integration in this slice (S03 binds it to egui, S05 wires the trigger path)

## Integration Closure

Upstream surfaces consumed: `src/pack/mod.rs` Pack/Category and `src/config.rs` MacroConfig/KeyAction (read-only — no schema changes). New wiring introduced: `PackEditor` is exported from `crate::pack`; no callers wire it yet (S03 binds it to the egui panel, S05 routes TriggerMacro). What remains before milestone is end-to-end usable: S03 (UI), S04 (import/export dialogs), S05 (trigger path), S06 (UAT).

## Verification

- Each CRUD method returns a `Result<()>` with a descriptive `anyhow::Error` naming the missing/duplicate identifier, so a future caller (UI panel, test) can render an actionable message. No structured logs needed at this layer — PackEditor is in-memory and synchronous; failure surfaces are the error returns themselves.

## Tasks

- [x] **T01: Define PackEditor struct and implement macro-level CRUD (AddMacro, EditMacro, RemoveMacro)** `est:1h`
  Create the `PackEditor` struct in `src/pack/mod.rs` as a mutable wrapper around `Pack`. Implement the three macro-level CRUD operations and a `MacroUpdates` helper struct for partial edits.

Skills used: tdd (red-green-refactor for each op), test (run cargo test).

## Why
S02 needs a pure-logic state object the UI (S03) and trigger path (S05) can build on. Macro-level CRUD is the densest and most-used surface, so it ships first with full unit-test coverage.

## Public API
```rust
pub struct PackEditor {
    pack: Pack,
}

#[derive(Debug, Clone, Default)]
pub struct MacroUpdates {
    pub phrase: Option<Option<String>>,    // Some(None) clears the field; None leaves unchanged
    pub if_flag: Option<Option<String>>,
    pub set_flag: Option<Option<String>>,
    pub sound: Option<Option<std::path::PathBuf>>,
    pub keys: Option<Vec<KeyAction>>,
}

impl PackEditor {
    pub fn new(pack: Pack) -> Self;
    pub fn pack(&self) -> &Pack;
    pub fn into_pack(self) -> Pack;
    pub fn add_macro(&mut self, category: &str, macro_config: MacroConfig) -> Result<()>;
    pub fn edit_macro(&mut self, category: &str, macro_name: &str, updates: MacroUpdates) -> Result<()>;
    pub fn remove_macro(&mut self, category: &str, macro_name: &str) -> Result<()>;
}
```

## Error semantics (use `anyhow::bail!` with explicit text)
- `add_macro`: error if category not found; error if a macro with the same name already exists in that category
- `edit_macro`: error if category not found; error if macro_name not found in that category
- `remove_macro`: error if category not found; error if macro_name not found

## Constraints
- Do NOT modify the `MacroConfig` or `Pack`/`Category` serde format. `MacroUpdates` is a sibling helper, not a serde type.
- Macro name uniqueness is enforced **within a category**, not globally.
- Preserve `Vec` ordering — pushing onto `category.macros` is correct for AddMacro; EditMacro mutates in place at the existing index.
- The `MacroUpdates::phrase: Option<Option<String>>` pattern lets callers distinguish 'leave unchanged' (`None`) from 'clear the field' (`Some(None)`). Same for if_flag/set_flag/sound. `keys: Option<Vec<KeyAction>>` replaces the entire vec when Some.

## Tests (in `src/pack/mod.rs::tests`)
- `editor_add_macro_success` — fresh PackEditor, add macro to existing category, assert it lands at the end of `categories[i].macros`
- `editor_add_macro_unknown_category_errors` — assert error message names the missing category
- `editor_add_macro_duplicate_name_errors` — pre-populate with a macro, attempt add with same name, assert error
- `editor_edit_macro_replaces_phrase_and_keys` — set `MacroUpdates { phrase: Some(Some("new".into())), keys: Some(vec![key]), ..Default::default() }`, assert old fields preserved (name, sound, etc.) and new ones applied
- `editor_edit_macro_can_clear_optional_field` — `MacroUpdates { phrase: Some(None), ..Default::default() }` clears phrase to None
- `editor_edit_macro_unknown_category_errors`
- `editor_edit_macro_unknown_macro_errors`
- `editor_remove_macro_success` — assert vec length decreases by 1 and named macro absent
- `editor_remove_macro_unknown_category_errors`
- `editor_remove_macro_unknown_macro_errors`

Use a small fixture helper at the top of the test module that builds a `Pack` with two categories (e.g. "Stratagems" and "Ship Modules") and 2 macros each, similar to the helpers in `tests/pack_hd2_bundle.rs` lines 21-50.

## Verify
```bash
RUSTFLAGS="-D warnings" cargo check --all-targets
cargo test --lib pack:: -- --test-threads=1
```
All new tests pass; no warnings.

## Done when
- `PackEditor`, `MacroUpdates`, and the three methods are defined and exported via `pub`
- 10 new unit tests under `src/pack/mod.rs::tests` pass
- `cargo check --all-targets` clean under `-D warnings`
- No changes to `MacroConfig`/`Pack`/`Category` definitions or serde behavior
  - Files: `src/pack/mod.rs`
  - Verify: RUSTFLAGS="-D warnings" cargo check --all-targets && cargo test --lib pack:: -- --test-threads=1

- [ ] **T02: Implement category-level CRUD (MoveMacro, RenameCategory, AddCategory, RemoveCategory)** `est:1h`
  Extend the `PackEditor` from T01 with the four category-level operations. Each method must validate state before mutation and return a descriptive error on failure.

Skills used: tdd, test.

## Why
UI (S03) needs to add new categories, rename them, move macros between categories, and delete empty categories. These operations are structurally riskier than macro-level CRUD because they mutate `Vec<Category>` and can leave the pack in a broken state if order or invariants slip — full unit coverage is required here.

## Public API additions
```rust
impl PackEditor {
    pub fn move_macro(&mut self, from_category: &str, to_category: &str, macro_name: &str) -> Result<()>;
    pub fn rename_category(&mut self, old_name: &str, new_name: &str) -> Result<()>;
    pub fn add_category(&mut self, name: &str) -> Result<()>;
    pub fn remove_category(&mut self, name: &str) -> Result<()>;
}
```

## Error semantics
- `move_macro`: error if from_category missing; if to_category missing; if macro_name missing from source; if a macro with the same name already exists in to_category. Atomic: do not remove from source unless the destination accepts it.
- `rename_category`: error if old_name missing; if new_name == old_name (no-op rejected); if new_name already in use
- `add_category`: error if name already in use; pushes an empty Category at the end of `pack.categories`
- `remove_category`: error if name missing; error if the category still has macros (must be emptied first via remove_macro). Does NOT cascade — caller must explicitly empty.

## Constraints
- Preserve `Vec<Category>` order — RemoveCategory uses `Vec::remove(idx)`, AddCategory uses `Vec::push`, RenameCategory mutates in place
- MoveMacro must be atomic: either both source-remove and dest-add succeed, or the pack is unchanged. Implement by validating both categories exist and the macro exists / no-name-collision in dest *before* doing any mutation.
- For RenameCategory, after rename, do NOT touch macro `if_flag`/`set_flag` or any other reference to the old category name — categories are not flag namespaces (per MEM027, MacroConfig.name is the flag-namespace identifier, not category names).

## Tests (append to `src/pack/mod.rs::tests`)
- `editor_move_macro_success` — macro disappears from source, appears at the end of dest
- `editor_move_macro_unknown_source_category_errors`
- `editor_move_macro_unknown_dest_category_errors`
- `editor_move_macro_unknown_macro_errors`
- `editor_move_macro_duplicate_in_dest_errors_and_leaves_source_unchanged` — pre-populate dest with same name; assert move fails AND source still has the macro (atomicity check)
- `editor_rename_category_success` — assert old name absent, new name at same index, macros preserved
- `editor_rename_category_unknown_errors`
- `editor_rename_category_duplicate_errors`
- `editor_add_category_success` — assert empty Category appended at end
- `editor_add_category_duplicate_errors`
- `editor_remove_category_success` — pre-empty category, then remove
- `editor_remove_category_non_empty_errors` — category with macros refuses removal
- `editor_remove_category_unknown_errors`

## Verify
```bash
RUSTFLAGS="-D warnings" cargo check --all-targets
cargo test --lib pack:: -- --test-threads=1
```
All new tests pass; existing T01 tests still pass; no warnings.

## Done when
- 4 category-level methods added and exported
- 13 new unit tests pass (combined with T01: 23 total under `src/pack/mod.rs::tests`)
- `editor_move_macro_duplicate_in_dest_errors_and_leaves_source_unchanged` proves atomicity
- `cargo check --all-targets` clean under `-D warnings`
  - Files: `src/pack/mod.rs`
  - Verify: RUSTFLAGS="-D warnings" cargo check --all-targets && cargo test --lib pack:: -- --test-threads=1

- [ ] **T03: Add hermetic round-trip integration test (edit → save → reload → structural equality)** `est:45m`
  Create `tests/pack_editor_roundtrip.rs` proving that PackEditor edits survive a full save/load cycle without corruption. This is the slice's integration-level proof that the pure-logic state machine composes correctly with the existing `Pack::save_to_dir` / `Pack::load_from_dir` boundary.

Skills used: test, verify-before-complete.

## Why
Unit tests in T01/T02 prove each CRUD op in isolation; this test proves that the *outcome* of a sequence of edits round-trips through serde_yaml_ng correctly — catching any field-ordering or serde-format regression that unit tests would miss.

## Test plan
Create one integration test file `tests/pack_editor_roundtrip.rs` with these tests:

1. `roundtrip_after_full_crud_sequence` — Build a starter Pack via local fixture helper (2 categories, 4 macros, mix of phrase/if_flag/set_flag/keys with timing). Wrap in PackEditor. Run a representative sequence: add_category("NewCat"), add_macro("NewCat", ...), edit_macro on existing, move_macro between categories, rename_category, remove_macro, remove_category (after emptying). Save to tempdir, reload, re-serialize to YAML, and assert that the reloaded Pack deep-equals the editor's in-memory pack (use field-by-field assertions: name, author, categories.len(), then per-category name + macro vec equality).

2. `roundtrip_yaml_text_stable_within_run` — Edit a Pack, save to one tempdir, save the same in-memory Pack to a second tempdir, read both `pack.yaml` files as bytes, and assert byte-for-byte equality. This guards against non-determinism in serde_yaml_ng output within a single process run.

3. `roundtrip_preserves_optional_fields` — Build a Pack containing macros with `phrase = None`, `if_flag = Some("flag")`, `set_flag = None`, `sound = None`, varied keys (some with dwell/gap overrides, some without). Run a no-op edit (add+remove same macro), save, reload, assert all Option fields preserved exactly.

## Constraints
- File must live at `tests/pack_editor_roundtrip.rs` (tracked in git, integration-test conventions)
- Use `tempfile::tempdir()` for the save/load destination; never write to the real `XDG_CONFIG_HOME` (no env mutation needed since we pass `dir.path()` directly to `save_to_dir`)
- No `#[serial]` attribute needed — these tests do not mutate process env (per MEM053 pattern)
- No fixtures from `.gitignore`d paths — define the starter Pack inline using small helper fns at the top of the test file (mirror the `key()`, `macro_simple()` helpers from `tests/pack_hd2_bundle.rs` lines 21-50)
- For deep equality, write a private `assert_packs_equal(left: &Pack, right: &Pack)` helper that walks categories and macros explicitly. Do NOT add a `PartialEq` derive to `Pack`/`Category`/`MacroConfig` — that would touch the canonical config types which are out of scope for this slice.

## Verify
```bash
RUSTFLAGS="-D warnings" cargo check --all-targets
cargo test --test pack_editor_roundtrip -- --test-threads=1
cargo test -- --test-threads=1
```
All three new tests pass; full suite still passes (no regression in pack_hd2_bundle, pack_hd2_coverage, etc.).

## Done when
- `tests/pack_editor_roundtrip.rs` exists, contains the 3 named tests, and they pass
- Full `cargo test -- --test-threads=1` is green
- `cargo check --all-targets` clean under `-D warnings`
- Per slice success criteria: 'round-trip: edit → save → reload → byte equivalence' is demonstrated by `roundtrip_yaml_text_stable_within_run` (byte equality of save outputs) plus `roundtrip_after_full_crud_sequence` (structural equality after save→load)
  - Files: `tests/pack_editor_roundtrip.rs`
  - Verify: RUSTFLAGS="-D warnings" cargo check --all-targets && cargo test --test pack_editor_roundtrip -- --test-threads=1 && cargo test -- --test-threads=1

## Files Likely Touched

- src/pack/mod.rs
- tests/pack_editor_roundtrip.rs
