---
id: T04
parent: S01
milestone: M007
key_files:
  - tests/profile_listing.rs
key_decisions:
  - Used XDG_CONFIG_HOME env-var override (same pattern as unit tests in config_app.rs) to redirect load_profiles() at the tempdir fixture — no API change needed
  - Used serial_test::serial to guard against env-var races with other tests that also set XDG_CONFIG_HOME
duration: 
verification_result: passed
completed_at: 2026-04-27T11:34:56.183Z
blocker_discovered: false
---

# T04: Add integration test pins load_profiles to {name}/pack.yaml subdirectory format, excluding flat .yaml files and dirs without pack.yaml

**Add integration test pins load_profiles to {name}/pack.yaml subdirectory format, excluding flat .yaml files and dirs without pack.yaml**

## What Happened

Created `tests/profile_listing.rs` with a single integration test (`load_profiles_only_returns_subdirectory_profiles`) that sets up a tempdir profiles directory containing three fixture entries: (a) a valid `good_profile/pack.yaml` subdirectory, (b) a flat `flat_profile.yaml` file at the profiles root, and (c) a `no_pack/` subdirectory with no `pack.yaml`. The test sets `XDG_CONFIG_HOME` to point `load_profiles()` at the tempdir, calls `vibe_attack::ui::config_app::load_profiles()`, and asserts the returned list is exactly `["good_profile"]`. The test uses `serial_test::serial` (already in dev-dependencies) to avoid XDG_CONFIG_HOME env-var races if similar tests run concurrently. A comment at the top of the file references the M007/S01 fix rationale. The test compiled and passed on the first run (`1 passed; 0 failed`). `cargo check` is clean with no new warnings.

## Verification

cargo test --test profile_listing — 1/1 pass. cargo check — clean, 0 errors/warnings.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test profile_listing` | 0 | ✅ pass — 1/1 tests | 1750ms |
| 2 | `cargo check` | 0 | ✅ pass — clean | 80ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `tests/profile_listing.rs`
