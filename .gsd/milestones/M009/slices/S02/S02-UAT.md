# S02: PackEditor pure-logic state + CRUD — UAT

**Milestone:** M009
**Written:** 2026-04-28T02:36:32.633Z

# S02 UAT — PackEditor Pure-Logic State + CRUD

## Preconditions
- Rust toolchain installed (`cargo --version`)
- Working directory: `/home/chadmin/Github/hd-linux-voice`
- No external processes or env vars required (all tests are hermetic)

## Test Cases

### TC-01: All pack unit tests pass
```bash
cargo test --lib pack:: -- --test-threads=1
```
**Expected:** `test result: ok. 27 passed; 0 failed; 0 ignored`
Verify the following named tests appear in output:
- `editor_add_macro_success`
- `editor_add_macro_unknown_category_errors`
- `editor_add_macro_duplicate_name_errors`
- `editor_edit_macro_replaces_phrase_and_keys`
- `editor_edit_macro_can_clear_optional_field`
- `editor_edit_macro_unknown_category_errors`
- `editor_edit_macro_unknown_macro_errors`
- `editor_remove_macro_success`
- `editor_remove_macro_unknown_category_errors`
- `editor_remove_macro_unknown_macro_errors`
- `editor_move_macro_success`
- `editor_move_macro_unknown_source_category_errors`
- `editor_move_macro_unknown_dest_category_errors`
- `editor_move_macro_unknown_macro_errors`
- `editor_move_macro_duplicate_in_dest_errors_and_leaves_source_unchanged`
- `editor_rename_category_success`
- `editor_rename_category_unknown_errors`
- `editor_rename_category_duplicate_errors`
- `editor_add_category_success`
- `editor_add_category_duplicate_errors`
- `editor_remove_category_success`
- `editor_remove_category_non_empty_errors`
- `editor_remove_category_unknown_errors`

### TC-02: Round-trip integration tests pass
```bash
cargo test --test pack_editor_roundtrip -- --test-threads=1
```
**Expected:** `test result: ok. 3 passed; 0 failed; 0 ignored`
Verify these three tests appear:
- `roundtrip_after_full_crud_sequence`
- `roundtrip_yaml_text_stable_within_run`
- `roundtrip_preserves_optional_fields`

### TC-03: No regressions in full suite
```bash
cargo test -- --test-threads=1
```
**Expected:** All tests pass; specifically `pack_hd2_bundle`, `pack_hd2_coverage`, and `pack_editor_roundtrip` suites all green.

### TC-04: Clean compile under warnings-as-errors
```bash
RUSTFLAGS="-D warnings" cargo check --all-targets
```
**Expected:** `Finished` with zero warnings.

### TC-05: Atomicity — move_macro leaves source intact on failure
Covered by `editor_move_macro_duplicate_in_dest_errors_and_leaves_source_unchanged`.
**Expected behavior:** When `move_macro` is called and the destination already contains a macro with the same name, the call returns an error AND the source category still contains the macro unchanged (no partial mutation).

### TC-06: Optional field clear semantics
Covered by `editor_edit_macro_can_clear_optional_field`.
**Expected behavior:** `MacroUpdates { phrase: Some(None), ..Default::default() }` clears the phrase field to None; all other fields (name, keys, if_flag, set_flag, sound) remain at their prior values.

### TC-07: Category rename does not affect macro flags
Covered by `editor_rename_category_success`.
**Expected behavior:** After `rename_category("Stratagems", "Strats")`, all macros in that category retain their original `if_flag` and `set_flag` values unchanged.

### TC-08: remove_category refuses non-empty category
Covered by `editor_remove_category_non_empty_errors`.
**Expected behavior:** Calling `remove_category` on a category that still has macros returns an error containing the macro count; the category remains in the pack.

## Edge Cases
- Duplicate macro name within same category → `add_macro` errors (TC-01: `editor_add_macro_duplicate_name_errors`)
- Same-name rename → `rename_category` errors (TC-01: `editor_rename_category_duplicate_errors` + unknown variant)
- YAML round-trip byte stability → two saves of the same Pack produce identical bytes (TC-02: `roundtrip_yaml_text_stable_within_run`)
- None/Some Option fields preserved across save→load → `roundtrip_preserves_optional_fields`
