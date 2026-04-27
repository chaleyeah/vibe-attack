---
id: S06
parent: M005
milestone: M005
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - (none)
key_decisions:
  - (none)
patterns_established:
  - (none)
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-27T00:41:27.515Z
blocker_discovered: false
---

# S06: GitHub Actions CI release pipeline

**.github/workflows/release.yml: on v* tag push, builds AppImage and uploads as release asset**

## What Happened

Created .github/workflows/release.yml. Trigger on push to tags v*. ubuntu-22.04 runner. Steps: checkout, dtolnay/rust-toolchain@stable, Swatinem/rust-cache@v2, apt-get (libasound2-dev, libclang-dev, librsvg2-bin, libfuse2, wget), download linuxdeploy + appimagetool AppImages and install to /usr/local/bin, run build.sh with APPIMAGE_EXTRACT_AND_RUN=1 (required when running AppImage tools inside GitHub Actions FUSE environment), rename artifact to include tag, upload via softprops/action-gh-release@v2. YAML validated with python3.

## Verification

python3 yaml.safe_load passes; workflow has trigger on v* tags, rust-cache, build.sh invocation, APPIMAGE_EXTRACT_AND_RUN env var, and softprops upload

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

None.

## Follow-ups

None.

## Files Created/Modified

- `.github/workflows/release.yml` — GitHub Actions release pipeline
