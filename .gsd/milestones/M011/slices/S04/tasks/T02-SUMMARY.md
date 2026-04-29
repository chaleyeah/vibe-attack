---
id: T02
parent: S04
milestone: M011
key_files:
  - .github/workflows/release.yml
key_decisions:
  - Refactored upload out of build-appimage into a dedicated release job (needs all three builds) — cleaner than keeping the appimage job as sole uploader and inserting download steps into it; enables symmetric artifact flow where every build job emits artifacts and the release job collects them all.
  - Source tarball for rpmbuild created with hardcoded prefix vibe-attack-1.0.0/ to match vibe-attack.spec Source0 expansion — the tag-driven tarball prefix from GITHUB_REF_NAME would require spec Source0 to be parameterised which rpmbuild does not support cleanly at build time.
  - libclang-dev included in both deb and rpm apt-get install lists because sherpa-onnx-sys is a transitive bindgen/clang-sys dependency (MEM092) — missing it would produce an opaque build failure.
duration: 
verification_result: passed
completed_at: 2026-04-29T01:55:41.194Z
blocker_discovered: false
---

# T02: Added build-deb and build-rpm jobs to release.yml and refactored artifact upload into a dedicated release job with all five artifact globs and fail_on_unmatched_files: true

**Added build-deb and build-rpm jobs to release.yml and refactored artifact upload into a dedicated release job with all five artifact globs and fail_on_unmatched_files: true**

## What Happened

The existing release.yml had a single `build-appimage` job that built the AppImage, source tarball, and hdpack and uploaded them directly to the GitHub Release in the same job. This task restructured the workflow into four jobs.

**build-deb** (new): Checks out, installs Rust, wires the sherpa-onnx prebuilt cache block verbatim from `build-appimage` (path: `target/sherpa-onnx-prebuilt`, key: `sherpa-onnx-1.12.39-linux-x64`), installs `libasound2-dev libclang-dev devscripts debhelper dh-cargo`, runs `dpkg-buildpackage -uc -us -b` from repo root, moves the emitted `vibe-attack_*.deb` from the parent directory back into the workspace, then uploads via `actions/upload-artifact@v4` (name: `deb`). libclang-dev is required because sherpa-onnx-sys is a transitive clang/bindgen dependency (MEM092).

**build-rpm** (new): Same checkout/Rust/rust-cache/sherpa-cache block, installs `libasound2-dev libclang-dev rpm`, creates the rpmbuild tree, generates a source tarball via `git archive --format=tar.gz --prefix=vibe-attack-1.0.0/ HEAD -o ~/rpmbuild/SOURCES/vibe-attack-1.0.0.tar.gz` (the tarball name matches `Source0` in `vibe-attack.spec`), copies the spec to `~/rpmbuild/SPECS/`, runs `rpmbuild -bb`, copies the output rpm back to workspace, uploads via `actions/upload-artifact@v4` (name: `rpm`).

**build-appimage** (modified): The `softprops/action-gh-release@v2` upload step was replaced with `actions/upload-artifact@v4` (name: `appimage`) so the new `release` job can collect all artifacts in one place.

**release** (new): Depends on `needs: [build-appimage, build-deb, build-rpm]`, downloads all three artifact bundles with `actions/download-artifact@v4`, then runs `softprops/action-gh-release@v2` with explicit five-glob `files:` block (`vibe-attack-*-x86_64.AppImage`, `vibe-attack-*.tar.gz`, `hd2-*.hdpack`, `vibe-attack_*.deb`, `vibe-attack-*.x86_64.rpm`) and `fail_on_unmatched_files: true` per MEM086.

The sherpa-onnx cache key (`sherpa-onnx-1.12.39-linux-x64`) now appears three times in the file — once per build job — so all three share the same Actions cache entry and avoid redundant network downloads per MEM089.

## Verification

Ran the exact combined verification command from the task plan: YAML parsed cleanly with python3 yaml.safe_load; grep confirmed `^  build-deb:` and `^  build-rpm:` job declarations; grep confirmed `vibe-attack_*.deb` and `vibe-attack-*.x86_64.rpm` globs in the upload step; grep confirmed `fail_on_unmatched_files: true`; grep -c for the sherpa cache key returned 3 (one per build job), satisfying the `>= 3` awk check. All checks exited 0.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `python3 -c 'import yaml; yaml.safe_load(open(".github/workflows/release.yml"))' && echo YAML valid` | 0 | ✅ pass | 120ms |
| 2 | `grep -q '^  build-deb:' .github/workflows/release.yml` | 0 | ✅ pass | 10ms |
| 3 | `grep -q '^  build-rpm:' .github/workflows/release.yml` | 0 | ✅ pass | 10ms |
| 4 | `grep -q 'vibe-attack_\*\.deb' .github/workflows/release.yml` | 0 | ✅ pass | 10ms |
| 5 | `grep -q 'vibe-attack-\*\.x86_64\.rpm' .github/workflows/release.yml` | 0 | ✅ pass | 10ms |
| 6 | `grep -q 'fail_on_unmatched_files: true' .github/workflows/release.yml` | 0 | ✅ pass | 10ms |
| 7 | `grep -c 'sherpa-onnx-1.12.39-linux-x64' .github/workflows/release.yml | awk '{ exit ($1 < 3) }'` | 0 | ✅ pass (count=3) | 15ms |

## Deviations

The RPM tarball prefix is hardcoded to `vibe-attack-1.0.0/` rather than derived from `GITHUB_REF_NAME`. The task plan specifies this approach explicitly (the tarball name must match `Source0` in the spec). This is intentional and correct — `rpmbuild %autosetup` expands the spec's `Version:` field, not a runtime env var, so a dynamic prefix would require complicating the spec.

## Known Issues

none

## Files Created/Modified

- `.github/workflows/release.yml`
