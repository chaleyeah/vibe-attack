---
id: T02
parent: S04
milestone: M010
key_files:
  - tests/packaging.rs
key_decisions:
  - onnxruntime test left intact: T01 kept onnxruntime in depends, so pkgbuild_declares_onnxruntime_runtime_dep remains valid and was not replaced
duration: 
verification_result: passed
completed_at: 2026-04-28T04:20:08.264Z
blocker_discovered: false
---

# T02: Add three packaging.rs assertions enforcing PKGBUILD AUR-readiness (clang makedep, sherpa-onnx offline source, SHERPA_ONNX_ARCHIVE_DIR)

**Add three packaging.rs assertions enforcing PKGBUILD AUR-readiness (clang makedep, sherpa-onnx offline source, SHERPA_ONNX_ARCHIVE_DIR)**

## What Happened

Extended `tests/packaging.rs` with three new tests that lock in T01's PKGBUILD changes as structural invariants:

1. **`pkgbuild_has_clang_in_makedepends`** — checks that `makedepends=` is present in PKGBUILD and that `'clang'` appears in the file. This ensures the bindgen/clang-sys build-time requirement stays wired.

2. **`pkgbuild_includes_sherpa_onnx_offline_source`** — asserts the exact filename `sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2` appears in the PKGBUILD source array. Removing or renaming the offline archive entry will now fail CI immediately.

3. **`pkgbuild_sets_sherpa_onnx_archive_dir`** — asserts `SHERPA_ONNX_ARCHIVE_DIR` appears in PKGBUILD, ensuring the build() function exports the env var the sherpa-onnx-sys build script uses to bypass its network fetch.

The pre-existing `pkgbuild_declares_onnxruntime_runtime_dep` test was left unchanged — T01 confirmed that `onnxruntime` must stay in `depends` (the $ORIGIN rpath only works in AppImage, not in the native Arch package layout), so the test is still correct.

All three new tests plus the seven pre-existing packaging tests pass (10/10).

## Verification

Ran `cargo test --test packaging -- pkgbuild` → 4 passed (3 new + 1 pre-existing). Ran `cargo test --test packaging` → all 10 tests passed, 0 failed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test packaging -- pkgbuild` | 0 | ✅ pass | 330ms |
| 2 | `cargo test --test packaging` | 0 | ✅ pass | 80ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `tests/packaging.rs`
