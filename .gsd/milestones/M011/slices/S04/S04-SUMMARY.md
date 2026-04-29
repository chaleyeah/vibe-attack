---
id: S04
parent: M011
milestone: M011
provides:
  - ["version-1.0.0-manifests", "release-workflow-deb-rpm", "packaging-test-assertions"]
requires:
  []
affects:
  []
key_files:
  - (none)
key_decisions:
  - ["Refactored release.yml upload out of build-appimage into a dedicated release job (needs all three builds) — enables symmetric artifact flow where every build job emits artifacts and the release job collects them all (MEM116)", "RPM source tarball prefix hardcoded to vibe-attack-1.0.0/ to match spec Source0 expansion — rpmbuild %autosetup expands Version: not a runtime env var; dynamic prefix would require complicating the spec (MEM117)", "Debian and RPM changelog 0.1.0 entries preserved as required historical records per append-only changelog conventions — not stale version strings (MEM118)", "packaging.rs job-name tests use .lines().any(|l| l == '  build-deb:') for column-2 YAML anchoring rather than .contains() to prevent false positives (MEM119)", "libclang-dev included in both build-deb and build-rpm apt-get install lists because sherpa-onnx-sys is a transitive bindgen/clang-sys dependency per MEM092"]
patterns_established:
  - ["4-job release workflow: 3 parallel build jobs each emit upload-artifact, 1 release job collects via download-artifact and publishes — symmetric and extensible", "Static YAML contract tests in tests/packaging.rs: string-contains checks on raw YAML enforce CI job structure at cargo test time without requiring a real tag push", "Sherpa-onnx cache block copied verbatim into every build job in release.yml — enforced by packaging test counting occurrences of the cache key"]
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-29T01:58:53.500Z
blocker_discovered: false
---

# S04: Version bump + release CI (.deb / .rpm jobs)

**Bumped project to 1.0.0 across all packaging manifests and extended release.yml with build-deb and build-rpm jobs in a 4-job architecture, verified by 15 passing packaging tests.**

## What Happened

S04 delivered two closely related goals: a version bump to 1.0.0 across every packaging manifest, and a restructured release workflow that produces .deb and .rpm artifacts alongside the existing AppImage + tarball + hdpack.

**T01 — Version bump (all manifests):**
All five packaging manifests were updated atomically from 0.1.0 to 1.0.0 using the date 2026-04-28. Cargo.toml line 3 now reads `version = "1.0.0"`. The vibe-attack.spec Version: field and its %changelog have a new 1.0.0-1 entry dated Tue Apr 28 2026. PKGBUILD reads `pkgver=1.0.0` with sha256sums=('SKIP','SKIP') unchanged per MEM093. packaging/debian/changelog has a new prepended stanza `vibe-attack (1.0.0-1) unstable; urgency=medium` with a 2026-04-28 timestamp. CHANGELOG.md gained a `## [1.0.0] - 2026-04-28` block with all prior Unreleased content moved into it; `## [Unreleased]` is left as an empty header for future work. The 0.1.0 entries that remain in vibe-attack.spec and debian/changelog are correct historical records per RPM/Debian append-only changelog conventions — they are not active version strings. The "Notes on versioning" paragraph in CHANGELOG.md (which stated no numbered release had been cut) was removed as its premise was no longer accurate.

**T02 — release.yml restructure (4-job architecture):**
The existing release.yml had a single `build-appimage` job that built all artifacts and uploaded them directly to GitHub Releases. This was refactored into four jobs:
- **build-appimage** (modified): now emits its outputs (AppImage, source tarball, hdpack) via `actions/upload-artifact@v4` (name: `appimage`) instead of uploading directly to the release.
- **build-deb** (new): installs `libasound2-dev libclang-dev devscripts debhelper dh-cargo`, runs `dpkg-buildpackage -uc -us -b`, moves the emitted .deb back into the workspace, uploads via upload-artifact (name: `deb`). Includes the sherpa-onnx cache block verbatim from build-appimage per MEM089.
- **build-rpm** (new): installs `libasound2-dev libclang-dev rpm`, creates the rpmbuild tree, generates a source tarball with hardcoded prefix `vibe-attack-1.0.0/` to match the spec's Source0, runs `rpmbuild -bb`, copies the output .rpm to workspace, uploads via upload-artifact (name: `rpm`). Same sherpa-onnx cache block per MEM089.
- **release** (new): `needs: [build-appimage, build-deb, build-rpm]`, downloads all three artifact bundles via `actions/download-artifact@v4`, then runs `softprops/action-gh-release@v2` with five explicit globs (`vibe-attack-*-x86_64.AppImage`, `vibe-attack-*.tar.gz`, `hd2-*.hdpack`, `vibe-attack_*.deb`, `vibe-attack-*.x86_64.rpm`) and `fail_on_unmatched_files: true` per MEM086.

The sherpa-onnx cache key `sherpa-onnx-1.12.39-linux-x64` now appears 3 times in the file (once per build job), ensuring all three release build jobs share the same cache entry and avoid redundant network downloads per MEM089.

**T03 — packaging test assertions:**
Five new tests were added to tests/packaging.rs: `release_yml_has_build_deb_job`, `release_yml_has_build_rpm_job`, `release_yml_uploads_deb_artifact`, `release_yml_uploads_rpm_artifact`, and `release_yml_caches_sherpa_onnx_in_all_release_jobs`. Job-name tests use `.lines().any(|l| l == "  build-deb:")` for column-2 anchored matching (preventing false positives from step-level keys). All 15 packaging tests pass (`cargo test --test packaging`).

## Verification

All slice-level verification checks passed:

**Version bump (T01):**
- `grep -q '^version = "1.0.0"' Cargo.toml` → exit 0
- `grep -q '^Version:        1.0.0' packaging/vibe-attack.spec` → exit 0
- `grep -q '^pkgver=1.0.0' packaging/PKGBUILD` → exit 0
- `head -1 packaging/debian/changelog | grep -q '1.0.0-1'` → exit 0
- `grep -q '## [1.0.0] - 2026-04-28' CHANGELOG.md` → exit 0

**release.yml structure (T02):**
- `python3 -c 'import yaml; yaml.safe_load(open(".github/workflows/release.yml"))'` → YAML valid
- `grep -q '^  build-deb:' .github/workflows/release.yml` → exit 0
- `grep -q '^  build-rpm:' .github/workflows/release.yml` → exit 0
- `grep -q 'vibe-attack_\*\.deb' .github/workflows/release.yml` → exit 0
- `grep -q 'vibe-attack-\*\.x86_64\.rpm' .github/workflows/release.yml` → exit 0
- `grep -q 'fail_on_unmatched_files: true' .github/workflows/release.yml` → exit 0
- `grep -c 'sherpa-onnx-1.12.39-linux-x64' | awk '{exit ($1 < 3)}'` → exit 0 (count=3)

**Packaging tests (T03):**
`cargo test --test packaging` → 15 passed, 0 failed, 0 ignored
- release_yml_has_build_deb_job ... ok
- release_yml_has_build_rpm_job ... ok
- release_yml_uploads_deb_artifact ... ok
- release_yml_uploads_rpm_artifact ... ok
- release_yml_caches_sherpa_onnx_in_all_release_jobs ... ok
- All 10 pre-existing tests continue to pass

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

["build-deb and build-rpm jobs have not been exercised by a real tag push — static assertions verify workflow structure but not runtime correctness; that is S05's responsibility per MEM111", "RPM tarball prefix is hardcoded to vibe-attack-1.0.0/ — future version bumps must update this string in the build-rpm job step alongside the spec Version: field"]

## Follow-ups

["S05: Push v1.0.0 tag to validate build-deb and build-rpm jobs actually produce artifacts in GitHub Actions; pin AUR PKGBUILD sha256sums to real release hashes after the tag push"]

## Files Created/Modified

- `Cargo.toml` — Bumped version from 0.1.0 to 1.0.0
- `packaging/vibe-attack.spec` — Bumped Version: to 1.0.0; added %changelog entry for 1.0.0-1 dated 2026-04-28
- `packaging/PKGBUILD` — Bumped pkgver to 1.0.0; sha256sums unchanged
- `packaging/debian/changelog` — Prepended vibe-attack (1.0.0-1) stanza dated 2026-04-28
- `CHANGELOG.md` — Added ## [1.0.0] - 2026-04-28 block; moved Unreleased content into it; removed stale versioning note
- `.github/workflows/release.yml` — Added build-deb and build-rpm jobs; refactored build-appimage to emit upload-artifact; added release collector job with 5-glob upload
- `tests/packaging.rs` — Added 5 new static assertions for build-deb, build-rpm, deb/rpm globs, and sherpa cache count
