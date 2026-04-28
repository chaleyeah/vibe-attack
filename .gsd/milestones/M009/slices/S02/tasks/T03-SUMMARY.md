---
id: T03
parent: S02
milestone: M009
key_files:
  - tests/pack_editor_roundtrip.rs
key_decisions:
  - assert_packs_equal walks category and macro vecs explicitly (no PartialEq derive) so divergence is reported at the exact field that differs, as required by the slice constraint
  - macro_optional() helper added alongside macro_simple() to cover None-phrase and flag-gated fixtures without repeating MacroConfig construction inline
duration: 
verification_result: passed
completed_at: 2026-04-28T02:34:49.285Z
blocker_discovered: false
---

# T03: Added tests/pack_editor_roundtrip.rs with 3 hermetic integration tests proving PackEditor edits survive save→reload and that YAML output is byte-stable within a run

**Added tests/pack_editor_roundtrip.rs with 3 hermetic integration tests proving PackEditor edits survive save→reload and that YAML output is byte-stable within a run**

## What Happened

Created `tests/pack_editor_roundtrip.rs` with three integration tests that exercise the full save/load boundary:

1. `roundtrip_after_full_crud_sequence` — builds a 2-category, 4-macro starter Pack, runs every PackEditor method in sequence (add_category, add_macro, edit_macro, move_macro, rename_category, remove_macro, remove_category), then saves to a tempdir, reloads, and asserts structural equality using an explicit field-walk helper `assert_packs_equal` that reports diverging paths down to individual key timing fields.

2. `roundtrip_yaml_text_stable_within_run` — applies edits, then saves the same in-memory Pack to two separate tempdirs and asserts byte-for-byte equality of the resulting `pack.yaml` files, guarding against serde_yaml_ng non-determinism.

3. `roundtrip_preserves_optional_fields` — builds a Pack with every combination of None/Some for phrase, if_flag, set_flag, and mixed timed/untimed keys; applies a no-op add+remove edit; saves and reloads; asserts all Option fields are exactly preserved, with explicit per-field assertions on dwell_ms, gap_ms, and empty key vecs.

The test file mirrors the fixture-helper pattern from `tests/pack_hd2_bundle.rs` (key(), key_timed(), macro_simple()) and defines an additional macro_optional() helper for None-phrase and flag-gated variants. No `#[serial]` attribute is needed — tests write only to their own tempdirs and do not mutate process env. No PartialEq derive was added to Pack/Category/MacroConfig, per the slice constraint.

## Verification

Ran three verification commands in sequence:

1. `RUSTFLAGS="-D warnings" cargo check --all-targets` — clean, no warnings.
2. `cargo test --test pack_editor_roundtrip -- --test-threads=1` — all 3 new tests pass.
3. `cargo test -- --test-threads=1` — full suite: 78 unit tests + 3 roundtrip + all other integration tests pass; 0 failures, 0 regressions.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `RUSTFLAGS="-D warnings" cargo check --all-targets` | 0 | ✅ pass | 150ms |
| 2 | `cargo test --test pack_editor_roundtrip -- --test-threads=1` | 0 | ✅ pass — 3/3 tests pass | 2280ms |
| 3 | `cargo test -- --test-threads=1` | 0 | ✅ pass — full suite green, 0 failures | 1440ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `tests/pack_editor_roundtrip.rs`
