# S04: AUR PKGBUILD finalization and submission — UAT

**Milestone:** M010
**Written:** 2026-04-28T04:23:26.547Z

# S04 UAT: AUR PKGBUILD Finalization

## Preconditions
- Working directory: `/home/chadmin/Github/hd-linux-voice`
- `packaging/PKGBUILD` exists
- `tests/packaging.rs` exists
- `docs/distribution-proofs/aur/README.md` exists

---

## Test 1: PKGBUILD contains clang in makedepends

**Steps:**
1. Run: `grep "^makedepends=" packaging/PKGBUILD`

**Expected:** Output contains `clang` inside the `makedepends=(...)` array, e.g. `makedepends=('rust' 'clang' ...)`

---

## Test 2: PKGBUILD lists sherpa-onnx offline source archive

**Steps:**
1. Run: `grep "sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2" packaging/PKGBUILD`

**Expected:** Line is present and references the correct GitHub release URL under `k2-fsa/sherpa-onnx`

---

## Test 3: PKGBUILD exports SHERPA_ONNX_ARCHIVE_DIR in build()

**Steps:**
1. Run: `grep "SHERPA_ONNX_ARCHIVE_DIR" packaging/PKGBUILD`

**Expected:** Line `export SHERPA_ONNX_ARCHIVE_DIR="$srcdir"` (or equivalent) appears inside the `build()` function

---

## Test 4: PKGBUILD retains onnxruntime in depends with comment

**Steps:**
1. Run: `grep -A2 "onnxruntime" packaging/PKGBUILD`

**Expected:** `onnxruntime` appears in the `depends=` array; a comment nearby explains the RPATH/$ORIGIN distinction between AppImage and native Arch package layouts

---

## Test 5: All 10 packaging tests pass

**Steps:**
1. Run: `cargo test --test packaging`

**Expected:** `test result: ok. 10 passed; 0 failed; 0 ignored`
Specifically: `pkgbuild_has_clang_in_makedepends`, `pkgbuild_includes_sherpa_onnx_offline_source`, `pkgbuild_sets_sherpa_onnx_archive_dir`, and `pkgbuild_declares_onnxruntime_runtime_dep` all pass

---

## Test 6: AUR submission doc exists and is complete

**Steps:**
1. Run: `test -f docs/distribution-proofs/aur/README.md && echo EXISTS`
2. Run: `grep "STATUS:" docs/distribution-proofs/aur/README.md`
3. Run: `grep "namcap" docs/distribution-proofs/aur/README.md`
4. Run: `grep "aur.archlinux.org" docs/distribution-proofs/aur/README.md`
5. Run: `grep "sha256sum\|updpkgsums" docs/distribution-proofs/aur/README.md`
6. Run: `wc -l docs/distribution-proofs/aur/README.md`

**Expected:**
- File exists
- `STATUS: pending submission` line present
- `namcap` verification step present
- AUR git push target (`ssh://aur@aur.archlinux.org/vibe-attack.git`) documented
- sha256 pinning commands present
- Line count > 30

---

## Edge Cases

**Edge case — sha256sums placeholder:** `grep "sha256sums" packaging/PKGBUILD` should show `'SKIP'` for both entries. This is intentional — real hashes are pinned at release time per the workflow in `docs/distribution-proofs/aur/README.md`. A SKIP in PKGBUILD at this stage is correct behavior, not a defect.

**Edge case — pkgver placeholder:** `grep "^pkgver=" packaging/PKGBUILD` should show `0.1.0`. This placeholder is correct until the v0.1.0 git tag is cut and the release workflow runs.
