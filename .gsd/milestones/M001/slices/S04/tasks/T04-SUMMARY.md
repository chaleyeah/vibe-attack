---
id: T04
parent: S04
milestone: M001
key_files:
  - tests/pack_hd2_bundle.rs
key_decisions:
  - T04 task plan was a legacy stub; derived intent as fixing the path bug identified in T03
  - Fixture path must use dir.path().join("hd-linux-voice/profiles") not join("profiles") because xdg::BaseDirectories::with_prefix appends the prefix to config_home before the test can set XDG_CONFIG_HOME
duration: 
verification_result: passed
completed_at: 2026-04-25T19:26:02.505Z
blocker_discovered: false
---

# T04: Fix test fixture path bug in profile_manager_get_active_pack_resolves_from_profiles_dir — dir.path().join("profiles") corrected to join("hd-linux-voice/profiles") to match xdg::BaseDirectories::with_prefix output

**Fix test fixture path bug in profile_manager_get_active_pack_resolves_from_profiles_dir — dir.path().join("profiles") corrected to join("hd-linux-voice/profiles") to match xdg::BaseDirectories::with_prefix output**

## What Happened

T04's plan was a legacy stub ("Migrated from legacy planning format") with no implementation details. Derived intent from carry-forward context: T03 identified a concrete test fixture path bug that would cause `profile_manager_get_active_pack_resolves_from_profiles_dir` to fail at runtime.

**Root cause:** `get_profiles_dir()` in `src/pack/mod.rs` calls `xdg::BaseDirectories::with_prefix("hd-linux-voice")`, which appends the prefix to `$XDG_CONFIG_HOME`. So with `XDG_CONFIG_HOME=/tmp/x`, `get_profiles_dir()` returns `/tmp/x/hd-linux-voice/profiles/`. The test at lines 466-468 created the fixture pack at `dir.path().join("profiles").join("Helldivers 2")` — missing the `hd-linux-voice/` segment — so `get_active_pack()` would look in `/tmp/x/hd-linux-voice/profiles/Helldivers 2/` and find nothing.

**Fix applied:** Changed line 466 of `tests/pack_hd2_bundle.rs` from:
```rust
let profiles_dir = dir.path().join("profiles");
```
to:
```rust
let profiles_dir = dir.path().join("hd-linux-voice/profiles");
```

**Scope check:** Audited all other XDG-path-dependent tests in the file. `pack_import_from_zip_reads_name_and_macros`, `pack_import_extracts_sounds_to_profile_dir`, and `hd2_pack_full_lifecycle_export_import_activate_retrieve` all route through `Pack::import()` which calls the production `get_profiles_dir()` internally — their assertions already use the `hd-linux-voice/profiles/...` path and are correct. `profile_manager_get_active_pack_none_when_no_active` and `profile_manager_get_active_pack_none_when_dir_missing` don't create fixtures in the profiles dir so they are unaffected.

**Runtime verification:** `cargo test` was blocked by shell approval policy (same constraint as T02/T03). Static correctness confirmed by cross-referencing the xdg-3.0.0 `with_prefix` source (appends prefix to config_home), `get_profiles_dir()` implementation (calls `xdg::BaseDirectories::with_prefix("hd-linux-voice")` then `.get_config_home().join("profiles")`), and the corrected fixture path logic.

## Verification

Static cross-reference verification: (1) read src/pack/mod.rs lines 152-158 to confirm get_profiles_dir() uses with_prefix("hd-linux-voice") and appends "profiles" to the xdg config home; (2) confirmed the corrected fixture path dir.path().join("hd-linux-voice/profiles").join("Helldivers 2") now matches the path get_active_pack() will search; (3) grep confirmed no other bare join("profiles") instances remain in the test file; (4) all 18 tests present and accounted for. cargo test blocked by shell approval policy.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `grep -n 'get_profiles_dir\|with_prefix\|join.*profiles' /home/chadmin/Github/hd-linux-voice/src/pack/mod.rs` | 0 | ✅ pass — confirmed get_profiles_dir() uses with_prefix("hd-linux-voice") returning XDG_CONFIG_HOME/hd-linux-voice/profiles/ | 10ms |
| 2 | `grep -n 'profiles_dir = dir.path' /home/chadmin/Github/hd-linux-voice/tests/pack_hd2_bundle.rs` | 0 | ✅ pass — corrected to join("hd-linux-voice/profiles") at line 466; no other bare join("profiles") instances found | 10ms |
| 3 | `grep -c '#\[test\]' /home/chadmin/Github/hd-linux-voice/tests/pack_hd2_bundle.rs` | 0 | ✅ pass — 18 #[test] functions confirmed present | 10ms |

## Deviations

T04 plan was a legacy stub with no implementation steps. Derived execution intent from T03 carry-forward context identifying a specific test fixture path bug. Fix was a single-line change to tests/pack_hd2_bundle.rs.

## Known Issues

cargo test could not be run due to shell approval policy — runtime pass/fail unconfirmed, but static analysis strongly supports correctness of the fix.

## Files Created/Modified

- `tests/pack_hd2_bundle.rs`
