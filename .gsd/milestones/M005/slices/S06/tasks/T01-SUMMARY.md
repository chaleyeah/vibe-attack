---
id: T01
parent: S06
milestone: M005
key_files:
  - .github/workflows/release.yml
key_decisions:
  - (none)
duration: 
verification_result: untested
completed_at: 2026-04-27T00:40:39.185Z
blocker_discovered: false
---

# T01: .github/workflows/release.yml: triggers on v* tags, builds AppImage via build.sh, uploads as release asset

**.github/workflows/release.yml: triggers on v* tags, builds AppImage via build.sh, uploads as release asset**

## What Happened

Created .github/workflows/release.yml. Trigger: push to tags matching v*. Runs on ubuntu-22.04. Steps: checkout, dtolnay/rust-toolchain@stable, Swatinem/rust-cache@v2, apt-get (libasound2-dev, libclang-dev, librsvg2-bin, libfuse2, wget), download linuxdeploy + appimagetool from their continuous releases and install to /usr/local/bin, run build.sh with APPIMAGE_EXTRACT_AND_RUN=1 (needed for AppImages-running-inside-CI), rename artifact with tag, upload via softprops/action-gh-release@v2 with GITHUB_TOKEN. YAML validated with python3 yaml.safe_load.

## Verification

python3 yaml.safe_load returned without error; workflow has correct trigger (tags: v*), rust-cache step, build.sh call, and softprops upload step

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| — | No verification commands discovered | — | — | — |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `.github/workflows/release.yml`
