---
id: S05
parent: M011
milestone: M011
provides:
  - ["GitHub Release v1.0.0 live with all five distribution artifacts", "AUR-publishable PKGBUILD with real sha256sums pinned"]
requires:
  []
affects:
  []
key_files:
  - ["github/workflows/release.yml", "packaging/appimage/build.sh", "packaging/vibe-attack.spec", "packaging/PKGBUILD"]
key_decisions:
  - ["Used --nodeps for rpmbuild on Ubuntu since workflow pre-installs all build deps; avoids switching to a Fedora container", "Used LD_LIBRARY_PATH (not --library flags) for linuxdeploy to resolve dlopen-only libsherpa-onnx-c-api.so", "Extended find_so() in build.sh to search target/sherpa-onnx-prebuilt/ as fallback for Rust cache hit scenarios", "Deleted packaging/debian/compat and rely solely on debhelper-compat build-dep (modern debhelper convention)", "Removed manual README.md install from rpm spec %install section; let %doc handle it", "Added permissions: contents: write explicitly to release job (GITHUB_TOKEN is read-only by default)", "v1.0.0 tag treated as immutable post-publish; force-moving would invalidate PKGBUILD sha256sums"]
patterns_established:
  - ["Release pipeline: four jobs (build-appimage, build-deb, build-rpm → release) with fail_on_unmatched_files providing loud failure on missing globs", "AppImage .so discovery: search target/release/ first, fall back to target/sherpa-onnx-prebuilt/ hierarchy when Rust cache hits", "CI cross-distro packaging: build on Ubuntu with --nodeps for rpm and symlinked debian/ directory for deb"]
observability_surfaces:
  - none
drill_down_paths:
  - [".gsd/milestones/M011/slices/S05/tasks/T01-SUMMARY.md", ".gsd/milestones/M011/slices/S05/tasks/T02-SUMMARY.md"]
duration: ""
verification_result: passed
completed_at: 2026-04-29T11:47:29.180Z
blocker_discovered: false
---

# S05: Publish GitHub Release v1.0.0

**Published GitHub Release v1.0.0 with all five artifacts (AppImage, tarball, hdpack, .deb, .rpm) after fixing five CI defects across four workflow runs; pinned real sha256sums into PKGBUILD making the AUR submission publishable.**

## What Happened

S05 was the final assembly slice for M011: push the v1.0.0 tag, exercise the full 4-job release pipeline end-to-end for the first time, and pin real source hashes into packaging/PKGBUILD.

**T01 — Push v1.0.0 tag and verify release publishes all artifacts**

Pre-flight confirmed a clean working tree (only gitignored .gsd/ untracked files), no pre-existing local or remote v1.0.0 tag, and `gh auth status` showing `workflow` scope. The annotated tag was created and pushed, triggering workflow run 25087274992.

Five CI defects surfaced across four workflow iterations:

*Run 1 (25087274992) — three simultaneous failures:*
- **build-rpm**: `alsa-lib-devel is needed by vibe-attack-1.0.0-1.x86_64` — rpmbuild on Ubuntu rejects Fedora-style BuildRequires that apt can't satisfy. Fix: `rpmbuild -bb --nodeps` since the workflow pre-installs all real deps.
- **build-deb**: `cannot open file debian/changelog` — `packaging/debian/` is not at the standard `debian/` path. Fix: `ln -s packaging/debian debian` before `dpkg-buildpackage`.
- **build-appimage**: `linuxdeploy ERROR: Could not find dependency: libsherpa-onnx-c-api.so` — this .so is loaded via dlopen, not ELF RPATH, so ldd can't find it without LD_LIBRARY_PATH. Fix: set `LD_LIBRARY_PATH` to `AppDir/usr/lib/` before calling linuxdeploy (the `--library` flag approach was tried first and failed).

*Run 2 (25087583120) — two new failures:*
- **build-appimage**: linuxdeploy error persisted (the initial `--library` fix was wrong; switched to LD_LIBRARY_PATH).
- **build-deb**: `dh: error: debhelper compat level specified both in debian/compat and via build-dependency on debhelper-compat` — the legacy `packaging/debian/compat` file (containing `13`) conflicted with `debhelper-compat (= 13)` in Build-Depends. Fix: delete the compat file.
- **build-rpm**: `Installed (but unpackaged) file(s) found: /usr/share/doc/vibe-attack/README.md` — both `%install` (manual `install -Dm644`) and `%files` (`%doc README.md`) attempted to handle README, creating a conflict. Fix: remove the manual install line from `%install`.

*Run 3 (25087877037) — all three build jobs succeeded; release job failed:*
- **release**: `403 Resource not accessible by integration` — GITHUB_TOKEN defaults to read-only contents scope. Fix: add `permissions: contents: write` to the release job.

*Run 4 (25088175068) — AppImage failed due to Rust cache hit:*
- **build-appimage**: `libonnxruntime.so not found` — the Rust build cache was a full hit so `cargo build --release` was a no-op; the `ort` crate's build script never ran and never copied the .so to `target/release/`. Fix: extended `find_so()` in build.sh to search `target/sherpa-onnx-prebuilt/` as a fallback — the .so lives at `target/sherpa-onnx-prebuilt/sherpa-onnx-v1.12.39-linux-x64-shared-lib/lib/`.

*Run 5 (25088427524) — all four jobs succeeded:*
GitHub Release v1.0.0 published with all 5 assets: `vibe-attack-v1.0.0-x86_64.AppImage` (20 MB), `vibe-attack-v1.0.0.tar.gz` (181 MB), `hd2-v1.0.0.hdpack`, `vibe-attack_1.0.0-1_amd64.deb` (7.5 MB), `vibe-attack-1.0.0-1.x86_64.rpm` (11 MB).

**T02 — Pin PKGBUILD sha256sums to real release hashes**

With the v1.0.0 release live, sha256 digests were computed for both PKGBUILD source entries:
- source[0] (https://github.com/chaleyeah/vibe-attack/archive/v1.0.0.tar.gz): `da0a2427d4812c274ec5fbaf4fa5dd7e13d4fb0030a484f4e06753b8ff6f4c6c`
- source[1] (sherpa-onnx v1.12.39 prebuilt linux-x64 shared-lib tarball): `1b95e49f889dee65310cab832d6181db619ea3ac77ecd60fe8b301028145781c`

Both 'SKIP' placeholders in `packaging/PKGBUILD` were replaced with real 64-char hex digests using the exact quoting and indentation expected by makepkg. Packaging test suite remained at 15 passed.

## Verification

1. `gh release view v1.0.0 --json tagName,isDraft,assets` → tag=v1.0.0, draft=false, count=5, all five artifact names present (AppImage, tarball, hdpack, .deb, .rpm)
2. `gh run list --workflow=release.yml --limit=1 --json conclusion` → "success" (run 25088427524)
3. `grep -oE "'[0-9a-f]{64}'" packaging/PKGBUILD | wc -l` → 2 (both hashes pinned)
4. `! grep -q "'SKIP'" packaging/PKGBUILD` → exit 0 (no SKIP entries remain)
5. `cargo test --test packaging` → test result: ok. 15 passed; 0 failed
6. `git ls-remote --tags origin v1.0.0` → printed ref 48b065f7a3edd6983f96f8f00de3d512cb5e73cc refs/tags/v1.0.0

Note: curl HTTP 200 check against release download URL returned 404 — repository is private; unauthenticated GitHub release URLs are not accessible. Asset upload confirmed via gh API instead (all 5 assets in state: uploaded).

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

curl HTTP 200 verification not applicable: repository is private, so unauthenticated GitHub release download URLs return 404. Task plan assumed a public repo for this check. Asset upload state confirmed via `gh release view` API (all 5 assets in state: uploaded, non-draft release).

## Known Limitations

AUR submission itself (mkaurball, git push aur) is operator runbook work outside S05 scope, documented in docs/distribution-proofs/aur/README.md. The repository is currently private — release assets are not publicly accessible via unauthenticated URLs.

## Follow-ups

M011/S02-T03 (final-distro UAT loop on debian13/ubuntu2604/fedora44/cachyos VMs) is now unblocked — the releases/latest/download/ URL is live for authenticated users and the PKGBUILD sha256sums are pinned for AUR testing.

## Files Created/Modified

None.
