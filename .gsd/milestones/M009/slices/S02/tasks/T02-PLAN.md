---
estimated_steps: 47
estimated_files: 1
skills_used: []
---

# T02: Implement category-level CRUD (MoveMacro, RenameCategory, AddCategory, RemoveCategory)

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

## Inputs

- ``src/pack/mod.rs``

## Expected Output

- ``src/pack/mod.rs``

## Verification

RUSTFLAGS="-D warnings" cargo check --all-targets && cargo test --lib pack:: -- --test-threads=1

## Observability Impact

MoveMacro must be observably atomic — its failure mode (duplicate name in dest) leaves source intact. The dedicated atomicity test makes the invariant inspectable. Other ops follow the same Result-returns-named-identifier pattern as T01.
