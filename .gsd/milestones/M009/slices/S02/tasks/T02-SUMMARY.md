---
id: T02
parent: S02
milestone: M009
key_files:
  - src/pack/mod.rs
key_decisions:
  - move_macro validates all preconditions (both category indices + macro index + dest duplicate check) before any Vec mutation to ensure atomicity on failure
  - remove_category refuses non-empty categories and includes the macro count in the error message to give callers an actionable message
  - rename_category rejects old_name == new_name as a no-op rather than silently succeeding, to surface accidental no-ops to callers
duration: 
verification_result: passed
completed_at: 2026-04-28T02:32:41.786Z
blocker_discovered: false
---

# T02: Added MoveMacro, RenameCategory, AddCategory, RemoveCategory to PackEditor with 13 unit tests, all 27 pack tests passing

**Added MoveMacro, RenameCategory, AddCategory, RemoveCategory to PackEditor with 13 unit tests, all 27 pack tests passing**

## What Happened

Extended `PackEditor` in `src/pack/mod.rs` with the four category-level CRUD methods specified in the plan.

**`move_macro`** validates all preconditions atomically before mutating: it resolves the source category index, destination category index, and macro index, then checks for a duplicate name in the destination — all before removing from source or appending to dest. This ensures the pack is never left in a half-moved state on failure.

**`rename_category`** rejects same-name renames as a no-op (bail), rejects duplicates with a named error, then mutates the category name in place, preserving its index in `pack.categories` and all its macros.

**`add_category`** checks for a duplicate name then pushes a new empty `Category` to the end of `pack.categories`.

**`remove_category`** finds the category by name, rejects removal if it still contains macros (with a count in the error message), then removes by index via `Vec::remove` to preserve order of remaining categories. Does NOT cascade — callers must explicitly empty via `remove_macro`.

All 13 specified unit tests were written and pass. The atomicity invariant for `move_macro` is explicitly covered by `editor_move_macro_duplicate_in_dest_errors_and_leaves_source_unchanged`, which pre-populates the dest with a same-named macro, asserts the error fires, and then asserts the source still contains the macro unchanged.

Category names are not flag namespaces — `rename_category` only mutates `category.name` and does not touch any macro `if_flag`/`set_flag` fields, per MEM027.

## Verification

Ran `RUSTFLAGS="-D warnings" cargo check --all-targets` — finished clean in 440ms. Ran `cargo test --lib pack:: -- --test-threads=1` — 27 passed, 0 failed, 0 ignored (13 new T02 tests + 10 T01 PackEditor tests + 4 pre-existing Pack/manager tests).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `RUSTFLAGS="-D warnings" cargo check --all-targets` | 0 | ✅ pass — zero warnings | 440ms |
| 2 | `cargo test --lib pack:: -- --test-threads=1` | 0 | ✅ pass — 27 passed, 0 failed, 0 ignored | 1340ms |

## Deviations

Plan expected 23 total tests under pack::tests; actual count is 25 in pack::tests + 2 in pack::manager::tests = 27 total. The difference is the two pre-existing manager sub-module tests were counted in the plan's filtered set. All 13 new T02 tests and all 10 T01 tests pass.

## Known Issues

None.

## Files Created/Modified

- `src/pack/mod.rs`
