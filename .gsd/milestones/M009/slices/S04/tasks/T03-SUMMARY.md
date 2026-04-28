---
id: T03
parent: S04
milestone: M009
key_files:
  - tests/pack_lifecycle.rs
key_decisions:
  - Fixture pack named 'LifecycleRoundTripFixture' to avoid collision with pack names used by other test files that mutate XDG_CONFIG_HOME
  - Helper functions key/key_timed/macro_simple/macro_flagged inlined directly (not shared) — two tiny functions don't warrant a shared test util crate
  - Deep per-field per-key assertion loop instead of PartialEq derive — produces actionable failure messages naming the specific macro and field that diverged
duration: 
verification_result: passed
completed_at: 2026-04-28T03:04:42.848Z
blocker_discovered: false
---

# T03: Added tests/pack_lifecycle.rs with two hermetic integration tests covering the export → import_to round-trip for macro fields and sounds bundling

**Added tests/pack_lifecycle.rs with two hermetic integration tests covering the export → import_to round-trip for macro fields and sounds bundling**

## What Happened

Created `tests/pack_lifecycle.rs` with two integration tests that prove the export → import_to contract without any XDG_CONFIG_HOME mutation.

**Test 1 — `pack_export_then_import_to_round_trips_macros`:** Builds a `LifecycleRoundTripFixture` pack with 3 categories and 7 macros that exercise every `MacroConfig` field: `phrase`, `if_flag`, `set_flag`, `sound: None`, plain keys, and keys with `dwell_ms`/`gap_ms` overrides. Saves to a source tempdir, calls `pack.export()`, then `Pack::import_to()` into a separate dest tempdir. Reloads the extracted pack via `Pack::load_from_dir` and asserts identical: `name`, `author`, category count, category names in order, macro counts per category, macro names in order, and every `MacroConfig` field of every macro including per-key `dwell_ms`/`gap_ms`.

**Test 2 — `pack_export_imports_sounds_subdirectory`:** Writes a small dummy wav bytes buffer under `source_dir/sounds/test.wav`, exports, imports via `import_to`, and asserts that `dest_dir/LifecycleRoundTripFixture/sounds/test.wav` exists with byte-identical content. This locks in the sounds-bundling behaviour at the integration level.

Both tests use `tempfile::tempdir()` for isolation, no `#[serial]` annotation, and no shared state — fully parallel-safe. The fixture pack name `LifecycleRoundTripFixture` does not collide with any name used in other test files.

## Verification

Ran `cargo test --test pack_lifecycle -- --test-threads=1`: 2 passed, 0 failed. Ran full suite `cargo test -- --test-threads=1`: all tests passed (0 failures, several ignored due to hardware/model requirements as expected).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test pack_lifecycle -- --test-threads=1` | 0 | ✅ pass | 860ms |
| 2 | `cargo test -- --test-threads=1` | 0 | ✅ pass | 95000ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `tests/pack_lifecycle.rs`
