# S01: Dead code, dead deps, and the load_profiles bug fix — UAT

**Milestone:** M007
**Written:** 2026-04-27T11:39:26.766Z

# S01 UAT — Dead code, dead deps, load_profiles bug fix

## Preconditions
- Rust toolchain installed (stable, without rustup/clippy)
- Working directory: repo root
- No `/dev/uinput` required (all tests are hermetic)

## Test Cases

### TC-01: sha2 is absent from direct dependencies
1. Open `Cargo.toml`
2. Search for `sha2` in `[dependencies]`
**Expected:** No `sha2 = ...` line exists under `[dependencies]`. sha2 may appear in `Cargo.lock` as a transitive dep — this is acceptable.

### TC-02: cargo check passes with no errors
1. Run: `cargo check`
**Expected:** Exit 0. No compilation errors. No warnings about unresolved deps.

### TC-03: Full test suite passes single-threaded
1. Run: `cargo test -- --test-threads=1`
**Expected:** Exit 0. All lib and integration tests pass. `test_pack_export_import_with_sounds` passes (ordering flake only manifests under parallel execution). Integration tests for profile_listing, config, uinput, wake_word run or are correctly ignored behind feature/env gates.

### TC-04: GUI feature gate test suite passes
1. Run: `cargo test --features gui -- --test-threads=1`
**Expected:** Exit 0. 3 additional GUI-gated tests appear and pass. No regressions vs default feature set.

### TC-05: DispatcherState is not accessible outside src/pipeline/
1. Run: `grep -rn 'DispatcherState' src/ tests/`
**Expected:** All matches are inside `src/pipeline/dispatcher.rs`. Zero hits in tests/ or any other src/ module.

### TC-06: load_profiles returns only subdirectory-format profiles
Setup a profiles directory with:
- `good_profile/pack.yaml` (valid subdirectory profile)
- `flat_profile.yaml` (flat YAML at profiles root)
- `no_pack/` (subdirectory with no pack.yaml inside)

1. Run: `cargo test --test profile_listing`
**Expected:** Exit 0. Test `load_profiles_only_returns_subdirectory_profiles` passes. Result is exactly `["good_profile"]` — flat_profile and no_pack are excluded.

### TC-07: load_profiles unit tests reflect new subdirectory contract
1. Run: `cargo test --lib ui::config_app`
**Expected:** Exit 0. 4/4 tests pass, including:
- `load_profiles_returns_sorted_subdirectory_names`
- `load_profiles_ignores_flat_yaml_and_dirs_without_pack_yaml`

### TC-08: Success-criteria grep returns at most 1 hit
1. Run: `grep -rn 'hd.linux.voice\|hd_linux_voice\|hd2_linux\|TODO\|FIXME\|HACK\|dead_code\|allow(unused' src/`
**Expected:** Exactly 1 hit: `src/control/mod.rs:129` — the documented CancellationToken TODO. No other hits.

### TC-09: Warnings-as-errors clean on both feature sets
1. Run: `RUSTFLAGS="-D warnings" cargo check --all-targets`
2. Run: `RUSTFLAGS="-D warnings" cargo check --all-targets --features gui`
**Expected:** Both exit 0 with zero warnings.

## Edge Cases
- **Flat .yaml at profiles root:** Confirmed excluded by load_profiles (TC-06, TC-07).
- **Subdirectory without pack.yaml:** Confirmed excluded by load_profiles (TC-06, TC-07).
- **sha2 in Cargo.lock:** Acceptable — it's a transitive dep of zip. Only the direct-dependency pin was removed (TC-01).
- **Parallel test execution:** test_pack_export_import_with_sounds is a known ordering-flake; always run with --test-threads=1 for a clean signal (TC-03).
