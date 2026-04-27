---
id: T05
parent: S01
milestone: M007
key_files:
  - (none)
key_decisions:
  - cargo clippy is not available on this system (no rustup, no rust-clippy package); RUSTFLAGS=-D warnings cargo check --all-targets used as substitute — catches all rustc-native warnings, which clippy is a superset of; CI uses a rustup-provisioned toolchain for the real clippy run
  - test_pack_export_import_with_sounds fails under parallel execution (pre-existing tmpdir pollution flake, documented in T02); runs clean with --test-threads=1
duration: 
verification_result: passed
completed_at: 2026-04-27T11:37:39.124Z
blocker_discovered: false
---

# T05: Full S01 verification passes: cargo test (40 lib + integration tests clean with --test-threads=1), cargo check -D warnings clean on both default and gui feature sets, grep audit returns exactly one hit (documented control/mod.rs CancellationToken TODO)

**Full S01 verification passes: cargo test (40 lib + integration tests clean with --test-threads=1), cargo check -D warnings clean on both default and gui feature sets, grep audit returns exactly one hit (documented control/mod.rs CancellationToken TODO)**

## What Happened

Ran all five verification checks required by the task plan. 

**cargo test (no features, --test-threads=1):** 40 lib tests + all integration tests pass. The `test_pack_export_import_with_sounds` test fails only under parallel execution due to a pre-existing tmpdir pollution flake (documented in T02); it passes cleanly in isolation and with `--test-threads=1`.

**cargo test --features gui (--test-threads=1):** 43 lib tests pass (3 extra GUI-gated tests activate under this feature set). All integration tests pass.

**cargo clippy / cargo check -D warnings:** `cargo clippy` is not available on this system — rustup is absent and the rust-clippy package is not installed. CI uses a rustup-provisioned toolchain. As the best available substitute, ran `RUSTFLAGS="-D warnings" cargo check --all-targets` (default features) and `RUSTFLAGS="-D warnings" cargo check --all-targets --features gui`. Both exit 0 with zero warnings — the compiler's own warning set (which clippy is a superset of) is clean on both feature gates.

**Success-criteria grep:** `grep -rn 'hd.linux.voice\|hd_linux_voice\|hd2_linux\|TODO\|FIXME\|HACK\|dead_code\|allow(unused' src/` returned exactly one hit: `src/control/mod.rs:129: // TODO: wire to CancellationToken in a future slice` — the documented, justified TODO that is the only acceptable remaining hit per the slice contract.

## Verification

cargo test -- --test-threads=1 (exit 0, 40 passed); cargo test --features gui -- --test-threads=1 (exit 0, 43 passed); RUSTFLAGS="-D warnings" cargo check --all-targets (exit 0, no warnings); RUSTFLAGS="-D warnings" cargo check --all-targets --features gui (exit 0, no warnings); grep audit returns exactly 1 hit (control/mod.rs CancellationToken TODO)

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test -- --test-threads=1` | 0 | ✅ pass — 40 lib + integration tests, 0 failed | 5200ms |
| 2 | `cargo test --features gui -- --test-threads=1` | 0 | ✅ pass — 43 lib + integration tests, 0 failed | 6100ms |
| 3 | `RUSTFLAGS="-D warnings" cargo check --all-targets` | 0 | ✅ pass — 0 warnings (clippy unavailable; cargo check -D warnings is closest substitute) | 7770ms |
| 4 | `RUSTFLAGS="-D warnings" cargo check --all-targets --features gui` | 0 | ✅ pass — 0 warnings | 11600ms |
| 5 | `grep -rn 'hd.linux.voice|hd_linux_voice|hd2_linux|TODO|FIXME|HACK|dead_code|allow(unused' src/` | 0 | ✅ pass — exactly 1 hit: src/control/mod.rs:129 (documented CancellationToken TODO) | 30ms |

## Deviations

cargo clippy could not be run — not installed on this system. Substituted RUSTFLAGS="-D warnings" cargo check --all-targets for both feature sets, which enforces rustc's own warning-as-error flag. The real clippy lint run will occur in CI via the rustup-provisioned toolchain.

## Known Issues

cargo clippy requires rustup, which is absent from this environment. The CI workflow (.github/workflows/ci.yml lines 66–104) runs the authoritative clippy check on the CI runner where rustup provisions the toolchain. Local verification used cargo check -D warnings as the best available substitute.

## Files Created/Modified

None.
