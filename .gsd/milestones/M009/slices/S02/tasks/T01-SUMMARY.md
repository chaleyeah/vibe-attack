---
id: T01
parent: S02
milestone: M009
key_files:
  - src/pack/mod.rs
key_decisions:
  - MacroUpdates uses Option<Option<T>> for optional fields (phrase, if_flag, set_flag, sound) to distinguish leave-unchanged from clear, matching the plan API contract
  - Macro name uniqueness enforced within-category only, not globally across the Pack
duration: 
verification_result: passed
completed_at: 2026-04-28T02:30:37.556Z
blocker_discovered: false
---

# T01: Added PackEditor struct with MacroUpdates and macro-level CRUD (add_macro, edit_macro, remove_macro) to src/pack/mod.rs, with 10 unit tests all passing

**Added PackEditor struct with MacroUpdates and macro-level CRUD (add_macro, edit_macro, remove_macro) to src/pack/mod.rs, with 10 unit tests all passing**

## What Happened

Implemented `PackEditor` and `MacroUpdates` directly in `src/pack/mod.rs`, inserted above the existing `#[cfg(test)]` block.

`MacroUpdates` uses the `Option<Option<T>>` pattern for optional fields so callers can distinguish "leave unchanged" (`None`) from "clear the field" (`Some(None)`), and `Option<Vec<KeyAction>>` replaces the entire keys vec when `Some`.

`PackEditor::new` wraps a `Pack`. The three CRUD methods enforce all required invariants:
- `add_macro`: fails with a named error if the category is missing, or if a macro with the same name already exists in that category; otherwise appends to the end of `category.macros`.
- `edit_macro`: fails if the category or macro is missing; otherwise mutates the macro in-place at its existing index, applying only the fields that are `Some`.
- `remove_macro`: fails if the category or macro is missing; otherwise removes by index via `Vec::remove`.

A `fixture_pack()` helper at the top of the test module builds a `Pack` with two categories ("Stratagems" and "Ship Modules") and two macros each, matching the plan's guidance. All 10 specified unit tests were written and pass.

The only import change was adding `bail` and `KeyAction` to the existing `use anyhow::{Context, Result}` and `use crate::config::MacroConfig` imports — no serde format changes were made.

## Verification

Ran `RUSTFLAGS="-D warnings" cargo check --all-targets` — finished clean with zero warnings. Ran `cargo test --lib pack:: -- --test-threads=1` — 14 tests passed (10 new PackEditor tests + 4 pre-existing Pack tests), 0 failed, 0 ignored.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `RUSTFLAGS="-D warnings" cargo check --all-targets` | 0 | ✅ pass | 620ms |
| 2 | `cargo test --lib pack:: -- --test-threads=1` | 0 | ✅ pass — 14 passed, 0 failed | 1560ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/pack/mod.rs`
