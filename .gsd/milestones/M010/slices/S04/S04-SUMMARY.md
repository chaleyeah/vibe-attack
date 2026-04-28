---
id: S04
parent: M010
milestone: M010
provides:
  - ["AUR-submission-ready PKGBUILD with correct makedepends, offline sherpa-onnx source, and documented onnxruntime dependency rationale", "CI-enforced packaging assertions in tests/packaging.rs for clang makedep, sherpa offline source, and SHERPA_ONNX_ARCHIVE_DIR", "Full AUR submission workflow documentation for maintainer chaleyeah at docs/distribution-proofs/aur/README.md"]
requires:
  []
affects:
  - ["S05", "S06"]
key_files:
  - ["packaging/PKGBUILD", "tests/packaging.rs", "docs/distribution-proofs/aur/README.md"]
key_decisions:
  - ["onnxruntime kept in depends: RPATH=$ORIGIN resolves in AppImage (co-located .so files) but not in Arch native package (only binaries in /usr/bin/); system onnxruntime package required at runtime", "sha256sums use SKIP placeholders in development; pinned with real hashes at release time using workflow in docs/distribution-proofs/aur/README.md", "SHERPA_ONNX_ARCHIVE_DIR=$srcdir is the documented sherpa-onnx-sys escape hatch preventing network downloads inside makepkg sandbox"]
patterns_established:
  - ["Offline Arch builds: add prebuilt archive as source[] entry + export SHERPA_ONNX_ARCHIVE_DIR=$srcdir in build() to bypass cargo-time network calls", "Packaging cross-check: compare makedepends across Debian Build-Depends, RPM spec BuildRequires, and PKGBUILD makedepends to catch missing build-time deps", "AUR release workflow: SKIP sha256sums in-repo → pin real hashes at tag time → namcap → chroot makepkg → push PKGBUILD + .SRCINFO"]
observability_surfaces:
  - ["cargo test --test packaging — structural assertions enforce PKGBUILD shape stays correct across future edits", "docs/distribution-proofs/aur/README.md STATUS field — updated from 'pending submission' to 'submitted' after real AUR push"]
drill_down_paths:
  - [".gsd/milestones/M010/slices/S04/tasks/T01-SUMMARY.md", ".gsd/milestones/M010/slices/S04/tasks/T02-SUMMARY.md", ".gsd/milestones/M010/slices/S04/tasks/T03-SUMMARY.md"]
duration: ""
verification_result: passed
completed_at: 2026-04-28T04:23:26.547Z
blocker_discovered: false
---

# S04: AUR PKGBUILD finalization and submission

**PKGBUILD is AUR-submission-ready: clang in makedepends, sherpa-onnx offline source wired, onnxruntime dependency decision documented, and full submission workflow captured in docs.**

## What Happened

S04 finalized `packaging/PKGBUILD` for AUR submission readiness and established the release-time workflow for pinning hashes and pushing to aur.archlinux.org.

**T01 — PKGBUILD fixes (packaging/PKGBUILD)**

Three structural changes were made to the PKGBUILD:

1. **`clang` added to `makedepends`**: `bindgen`/`clang-sys` (transitive deps of `sherpa-onnx-sys`) require `libclang.so` at build time. Both the Debian `Build-Depends` (`libclang-dev`) and RPM spec (`clang-devel`) had this; the Arch PKGBUILD was the only packaging file missing it.

2. **sherpa-onnx prebuilt archive as `source[1]`**: Points to `https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.12.39/sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2`. Without this, `cargo build` inside makepkg's clean chroot sandbox would attempt a network download (which fails in sandbox environments). The `sherpa-onnx-sys/build.rs` escape hatch (`SHERPA_ONNX_ARCHIVE_DIR`) is how this is bypassed.

3. **`export SHERPA_ONNX_ARCHIVE_DIR="$srcdir"` in `build()`**: Tells `sherpa-onnx-sys` to look in `$srcdir` (where makepkg unpacks sources) for the archive, bypassing the network call. `sha256sums` for both entries use `'SKIP'` placeholders; T03 documents the release-time workflow for replacing them.

4. **`onnxruntime` kept in `depends` with explanatory comment**: Audited the binary with `readelf -d`. `libsherpa-onnx-c-api.so` has `NEEDED: libonnxruntime.so` with `RPATH: $ORIGIN`. In AppImage, both `.so` files co-locate in `usr/lib/` so `$ORIGIN` resolves. In the native Arch package, only the two binaries land in `/usr/bin/` — the `.so` files are not installed — so the system `onnxruntime` package (providing `/usr/lib/libonnxruntime.so`) is a genuine runtime dependency. An inline comment explains this.

**T02 — packaging.rs assertions (tests/packaging.rs)**

Three new tests lock in T01's changes as CI-enforced invariants:
- `pkgbuild_has_clang_in_makedepends` — asserts `makedepends=` and `'clang'` both appear in PKGBUILD
- `pkgbuild_includes_sherpa_onnx_offline_source` — asserts the exact archive filename is in the source array
- `pkgbuild_sets_sherpa_onnx_archive_dir` — asserts `SHERPA_ONNX_ARCHIVE_DIR` is exported in `build()`

The pre-existing `pkgbuild_declares_onnxruntime_runtime_dep` test was left unchanged (T01 kept `onnxruntime` in `depends`). All 10 packaging tests pass.

**T03 — AUR submission workflow doc (docs/distribution-proofs/aur/README.md)**

Created a 204-line document covering: (1) pre-submission checklist with exact `sha256sum`/`updpkgsums` commands for both source entries; (2) verification checklist (`namcap PKGBUILD`, `extra-x86_64-build` chroot, `makepkg --offline`, smoke tests); (3) submission steps — SSH config, `git clone ssh://aur@aur.archlinux.org/vibe-attack.git`, copy PKGBUILD, generate `.SRCINFO`, commit and push as chaleyeah; (4) notes explaining the onnxruntime dependency rationale and the SHERPA_ONNX_ARCHIVE_DIR mechanism. `STATUS: pending submission` at top per the proof-transcript convention.

`DECISIONS.md` was left unchanged because T01 kept `onnxruntime` (no structural removal occurred).

## Verification

All slice-level verification checks passed:
- `grep -q "^makedepends=.*clang" packaging/PKGBUILD` → exit 0
- `grep -q "sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2" packaging/PKGBUILD` → exit 0
- `grep -q "SHERPA_ONNX_ARCHIVE_DIR" packaging/PKGBUILD` → exit 0
- `cargo test --test packaging` → 10/10 tests passed (0 failed), including all 3 new T02 assertions
- `test -f docs/distribution-proofs/aur/README.md && grep -q "makepkg" ... && grep -q "namcap" ... && grep -q "aur.archlinux.org" ... && grep -q "STATUS:"` → exit 0
- `wc -l docs/distribution-proofs/aur/README.md` → 204 lines (minimum 30)

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

sha256sums in packaging/PKGBUILD contain SKIP placeholders — must be replaced with real hashes at release time. Actual makepkg -si clean-chroot run and git push to aur.archlinux.org are release-time operations outside the current CI environment, deferred to S06 final UAT and the human maintainer.

## Follow-ups

None.

## Files Created/Modified

None.
