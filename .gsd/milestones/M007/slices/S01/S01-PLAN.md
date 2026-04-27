# S01: Dead code, dead deps, and the load_profiles bug fix

**Goal:** Remove dead code (sha2 dependency), narrow DispatcherState visibility, and fix the latent load_profiles bug so the config UI lists profiles that handle_switch_profile can actually load. Add an integration test that pins the load_profiles behavior to the canonical {name}/pack.yaml subdirectory format.
**Demo:** cargo test passes; cargo clippy -D warnings clean; cargo check confirms no transitive breakage from sha2 removal; new integration test asserts load_profiles returns subdirectory-format profiles and ignores flat .yaml files; success-criteria grep returns zero hits in src/ except justified TODO in control/mod.rs

## Must-Haves

- sha2 absent from Cargo.toml and Cargo.lock; cargo check and cargo test pass; DispatcherState is pub(crate) with no compilation errors; load_profiles reads {name}/pack.yaml subdirectories and ignores flat *.yaml files; new hermetic integration test in tests/ exercises load_profiles against a fixture containing both formats; cargo clippy --all-targets -- -D warnings clean on default and gui features.

## Proof Level

- This slice proves: live integration — load_profiles fix is exercised by a new hermetic integration test using a tempdir fixture with one subdirectory profile and one flat .yaml. Other changes (sha2 removal, DispatcherState visibility) are verified by the existing 80+ test suite continuing to pass.

## Integration Closure

After this slice, the three places that touch the profiles directory — load_profiles (UI list), handle_switch_profile (control plane switch command), and Pack::load_from_dir (pack loader) — all agree on the {name}/pack.yaml subdirectory format. The user-facing inconsistency (UI lists a profile, switch command fails) is closed.

## Verification

- None. No log schema, tracing span, or metric is added or removed.

## Tasks

- [x] **T01: Remove sha2 dependency from Cargo.toml** `est:10m`
  Confirm via grep that sha2 is unused in src/ and tests/, then remove it from Cargo.toml [dependencies]. Run cargo check to confirm no transitive resolution failure. Run cargo test to confirm no regression. Commit Cargo.toml and the resulting Cargo.lock change.
  - Files: `Cargo.toml`, `Cargo.lock`
  - Verify: grep -rn 'use sha2\|sha2::' src/ tests/ returns no matches; cargo check succeeds; cargo test passes

- [x] **T02: Narrow DispatcherState visibility from pub to pub(crate)** `est:10m`
  Change `pub struct DispatcherState` (and any associated impls/methods that are pub) in src/pipeline/dispatcher.rs to pub(crate). Confirm via grep that DispatcherState is not referenced outside src/pipeline/. Run cargo check and cargo test.
  - Files: `src/pipeline/dispatcher.rs`
  - Verify: grep -rn 'DispatcherState' src/ tests/ shows references only inside src/pipeline/; cargo check succeeds; cargo test passes

- [x] **T03: Fix load_profiles to scan for {name}/pack.yaml subdirectories** `est:30m`
  In src/ui/config_app.rs, replace the existing load_profiles implementation that scans for flat profiles/*.yaml files with one that iterates entries of the profiles directory, treats each entry as a profile only if it is a directory containing a pack.yaml file, and uses the directory name as the profile name. The format must match what Pack::load_from_dir and handle_switch_profile already use.
  - Files: `src/ui/config_app.rs`
  - Verify: Code review: load_profiles iterates read_dir, filters DirEntry::file_type().is_dir() && entry.path().join("pack.yaml").exists(); flat .yaml files are NOT included in the returned list; cargo check passes

- [x] **T04: Add integration test for load_profiles canonical format** `est:45m`
  Add a new integration test (tests/profile_listing.rs or extend an existing test file in tests/) that creates a tempdir profiles directory containing: (a) one subdirectory named 'good_profile' containing a valid pack.yaml, (b) one flat file named 'flat_profile.yaml', (c) one empty subdirectory named 'no_pack' with no pack.yaml. Call load_profiles (or extract a testable helper if needed) and assert the returned list contains exactly ['good_profile'] and excludes both 'flat_profile' and 'no_pack'. Document the test rationale in a comment referencing the M007/S01 fix.
  - Files: `tests/profile_listing.rs`
  - Verify: cargo test --test profile_listing passes; the test creates the three fixture entries, calls the loader, and asserts only 'good_profile' is returned

- [x] **T05: Run full verification — test, clippy, success-criteria grep** `est:15m`
  Run cargo test, cargo test --features gui, cargo clippy --all-targets -- -D warnings, cargo clippy --all-targets --features gui -- -D warnings, and the success-criteria grep (`grep -rn 'hd.linux.voice\|hd_linux_voice\|hd2_linux\|TODO\|FIXME\|HACK\|dead_code\|allow(unused' src/`). All must pass or have explicit justification (the known control/mod.rs TODO about CancellationToken is the only acceptable remaining hit).
  - Verify: All four cargo invocations exit 0; grep returns at most one hit (the documented control/mod.rs TODO); record the grep output in the slice summary

## Files Likely Touched

- Cargo.toml
- Cargo.lock
- src/pipeline/dispatcher.rs
- src/ui/config_app.rs
- tests/profile_listing.rs
