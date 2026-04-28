---
id: S02
parent: M009
milestone: M009
provides:
  - ["PackEditor struct with 7 CRUD methods exported from crate::pack", "27 unit tests covering all CRUD success and error paths", "3 hermetic round-trip integration tests in tests/pack_editor_roundtrip.rs", "byte-stable YAML serialization proof via roundtrip_yaml_text_stable_within_run"]
requires:
  []
affects:
  []
key_files:
  - (none)
key_decisions:
  - ["MacroUpdates uses Option<Option<T>> for optional fields to distinguish leave-unchanged from clear", "move_macro validates all preconditions before any Vec mutation (atomic)", "rename_category rejects same-name as explicit error, not silent no-op", "rename_category does NOT cascade to macro if_flag/set_flag (MEM027)", "assert_packs_equal walks fields explicitly — no PartialEq derive on canonical config types"]
patterns_established:
  - ["PackEditor wraps Pack as a mutable state machine with typed CRUD methods returning Result<()>", "validate-then-mutate order for all multi-step Vec operations ensures atomicity", "Option<Option<T>> partial-update pattern for optional fields in CRUD helpers", "Private assert_packs_equal helper for integration tests avoids PartialEq on canonical types"]
observability_surfaces:
  - none
drill_down_paths:
  - ["src/pack/mod.rs — PackEditor impl block and MacroUpdates struct", "tests/pack_editor_roundtrip.rs — assert_packs_equal helper and 3 integration tests"]
duration: ""
verification_result: passed
completed_at: 2026-04-28T02:36:32.633Z
blocker_discovered: false
---

# S02: PackEditor pure-logic state + CRUD

**Implemented PackEditor with 7 CRUD methods and 27 unit tests, plus 3 hermetic round-trip integration tests — all passing under -D warnings.**

## What Happened

S02 delivered a complete pure-logic `PackEditor` struct in `src/pack/mod.rs` that wraps a `Pack` and exposes typed, error-returning CRUD methods for every operation the UI (S03) and trigger path (S05) will need.

**T01 — Macro-level CRUD (add_macro, edit_macro, remove_macro):**
`PackEditor` and `MacroUpdates` were added above the existing `#[cfg(test)]` block. `MacroUpdates` uses `Option<Option<T>>` for optional fields (phrase, if_flag, set_flag, sound) so callers can distinguish "leave unchanged" from "clear the field", and `Option<Vec<KeyAction>>` to replace the entire keys vec. All three CRUD methods enforce invariants (category existence, macro-name uniqueness within category, in-place index preservation) and return `anyhow::Error` with a descriptive message naming the missing or duplicate identifier. A `fixture_pack()` helper builds a Pack with two categories and two macros each; all 10 specified unit tests passed.

**T02 — Category-level CRUD (move_macro, rename_category, add_category, remove_category):**
`move_macro` validates all preconditions atomically before any mutation: both category indices, the macro index, and the duplicate-name check in the destination are all resolved before removing from source or appending to dest. `rename_category` mutates only `category.name` in place (never cascades to macro flags per MEM027), and rejects same-name renames as an explicit error. `add_category` pushes an empty Category at the end; `remove_category` refuses non-empty categories with a macro-count in the error message. All 13 specified unit tests passed, including the atomicity coverage test `editor_move_macro_duplicate_in_dest_errors_and_leaves_source_unchanged`. Combined with T01: 27 pack tests total pass (25 in pack::tests + 2 in pack::manager::tests).

**T03 — Hermetic round-trip integration test:**
`tests/pack_editor_roundtrip.rs` contains three tests:
- `roundtrip_after_full_crud_sequence`: full CRUD sequence (add_category, add_macro, edit_macro, move_macro, rename_category, remove_macro, remove_category), save to tempdir, reload, assert_packs_equal walks every field.
- `roundtrip_yaml_text_stable_within_run`: saves the same in-memory Pack to two separate tempdirs and asserts byte-for-byte YAML equality, guarding against non-determinism in serde_yaml_ng.
- `roundtrip_preserves_optional_fields`: exercises macros with None phrase, Some if_flag, varied KeyAction dwell/gap overrides; a no-op edit confirms all Option fields survive round-trip unchanged.

`assert_packs_equal` is a private helper that walks categories and macros explicitly rather than deriving PartialEq on canonical config types (out of scope for this slice), giving precise per-field failure messages.

**Verification:** Full suite — `RUSTFLAGS="-D warnings" cargo check --all-targets` clean, `cargo test -- --test-threads=1` → 78 lib/integration tests passed, 1 ignored (KWS heavy test), 0 failed.

## Verification

1. `RUSTFLAGS="-D warnings" cargo check --all-targets` → exit 0, zero warnings (0.08s)
2. `cargo test --lib pack:: -- --test-threads=1` → 27 passed, 0 failed, 0 ignored
3. `cargo test --test pack_editor_roundtrip -- --test-threads=1` → 3 passed, 0 failed
4. `cargo test -- --test-threads=1` → 78 passed, 0 failed, 1 ignored (KWS gated test)

All slice success criteria met:
- PackEditor and MacroUpdates defined and exported from crate::pack
- 7 CRUD methods: add_macro, edit_macro, remove_macro, move_macro, rename_category, add_category, remove_category
- 27 unit tests under src/pack/mod.rs::tests (all pass)
- 3 integration tests in tests/pack_editor_roundtrip.rs (all pass)
- Byte-equivalent YAML proven by roundtrip_yaml_text_stable_within_run
- Structural equality after save→load proven by roundtrip_after_full_crud_sequence
- No changes to MacroConfig/Pack/Category serde format

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

None.

## Follow-ups

None.

## Files Created/Modified

- `src/pack/mod.rs` — 
- `tests/pack_editor_roundtrip.rs` — 
