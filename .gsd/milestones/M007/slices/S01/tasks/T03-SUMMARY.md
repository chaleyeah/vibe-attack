---
id: T03
parent: S01
milestone: M007
key_files:
  - src/ui/config_app.rs
key_decisions:
  - Updated both the implementation and existing tests — the two tests that validated flat .yaml scanning were rewritten rather than leaving them as dead, always-passing fixtures for a behavior that no longer exists
duration: 
verification_result: passed
completed_at: 2026-04-27T11:33:19.276Z
blocker_discovered: false
---

# T03: Fixed load_profiles to scan for {name}/pack.yaml subdirectories instead of flat *.yaml files, matching the format used by Pack::load_from_dir and handle_switch_profile

**Fixed load_profiles to scan for {name}/pack.yaml subdirectories instead of flat *.yaml files, matching the format used by Pack::load_from_dir and handle_switch_profile**

## What Happened

The existing `load_profiles` implementation in `src/ui/config_app.rs` scanned the profiles directory for flat `*.yaml` files and returned their file stems. This was inconsistent with `Pack::load_from_dir` and `handle_switch_profile`, which expect profiles to live in subdirectories of the form `{name}/pack.yaml`. The config UI would list profile names that the switch handler could never actually load.

The fix replaces the `filter_map` predicate: instead of checking `path.extension() == Some("yaml")` and returning `file_stem()`, the new code checks `entry.file_type().ok()?.is_dir() && path.join("pack.yaml").exists()` and returns `file_name()` (the directory name). The doc comment was updated to reflect the new contract.

The two tests that validated the old flat-file behavior were rewritten to use subdirectory fixtures:
- `load_profiles_returns_sorted_subdirectory_names`: creates three subdirs each with `pack.yaml`, asserts sorted order.
- `load_profiles_ignores_flat_yaml_and_dirs_without_pack_yaml`: places a flat `hd2.yaml`, a directory without `pack.yaml`, and a valid `hd2/pack.yaml`; asserts only `["hd2"]` is returned.

All 4 config_app tests pass. `cargo check` is clean. The pre-existing `test_pack_export_import_with_sounds` flake (documented in T02) reappears when tests run concurrently but passes in isolation — unrelated to this change.

## Verification

cargo check — clean, 0 errors. cargo test --lib ui::config_app — 4/4 pass. cargo test --lib pack::tests::test_pack_export_import_with_sounds — passes in isolation (pre-existing concurrency flake, not caused by this task). Code review confirms: filter uses is_dir() && path.join("pack.yaml").exists(); flat .yaml files at the profiles root produce None and are excluded.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | ✅ pass | 360ms |
| 2 | `cargo test --lib ui::config_app` | 0 | ✅ pass — 4/4 tests | 1100ms |
| 3 | `cargo test --lib pack::tests::test_pack_export_import_with_sounds` | 0 | ✅ pass in isolation (pre-existing concurrency flake) | 80ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/ui/config_app.rs`
