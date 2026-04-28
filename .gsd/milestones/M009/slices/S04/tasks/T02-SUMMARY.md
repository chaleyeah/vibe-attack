---
id: T02
parent: S04
milestone: M009
key_files:
  - src/pack/mod.rs
key_decisions:
  - import_to takes the parent profiles dir (not the final pack dir) matching original import() semantics — callers pass profiles_dir and the function appends pack.name internally
duration: 
verification_result: passed
completed_at: 2026-04-28T03:03:22.893Z
blocker_discovered: false
---

# T02: Refactored Pack::import into Pack::import_to(zip_path, dest_dir) with tracing and a hermetic unit test; import() now delegates to import_to

**Refactored Pack::import into Pack::import_to(zip_path, dest_dir) with tracing and a hermetic unit test; import() now delegates to import_to**

## What Happened

Extracted the extraction logic from `Pack::import` into a new public `Pack::import_to(zip_path: &Path, dest_dir: &Path) -> Result<Pack>`. The new function takes the parent profiles directory as `dest_dir` and extracts into `dest_dir.join(&pack.name)`, preserving the original semantics: path-traversal protection via `enclosed_name()`, collision handling via `remove_dir_all` before extraction, and `create_dir_all` for directory entries. Added `tracing::info!` at the start of `import_to` with `zip_path` and `dest_dir` fields, and another at the end with `macro_count` after successful extraction. The existing `Pack::import` was reduced to a 3-line wrapper: it resolves `get_profiles_dir()` and calls `Self::import_to(zip_path, &profiles_dir)`. All existing call sites in tests (which use `XDG_CONFIG_HOME` override) continue to work through the wrapper unchanged. Added a new inline unit test `test_import_to_extracts_into_dest_dir` in the `#[cfg(test)] mod tests` block: it builds a Pack with one macro, exports to a tempdir zip, then calls `import_to` against a second tempdir (no XDG_CONFIG_HOME mutation), and asserts the imported pack name, category count, macro name, and presence of `pack.yaml` on disk.

## Verification

Ran `cargo test --lib pack:: -- --test-threads=1`: 28 tests passed (0 failed), including the new `test_import_to_extracts_into_dest_dir`. Ran `cargo test --test pack_hd2_bundle -- --test-threads=1`: 22 tests passed (0 failed), confirming the `import()` wrapper path is unbroken.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --lib pack:: -- --test-threads=1` | 0 | ✅ pass | 1350ms |
| 2 | `cargo test --test pack_hd2_bundle -- --test-threads=1` | 0 | ✅ pass | 1670ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/pack/mod.rs`
