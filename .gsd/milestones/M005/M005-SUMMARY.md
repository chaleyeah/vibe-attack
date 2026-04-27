---
id: M005
title: "Distribution Packaging"
status: complete
completed_at: 2026-04-27T00:42:25.622Z
key_decisions:
  - SVG icon used natively in Debian/PKGBUILD/RPM (hicolor/scalable/apps/); build.sh converts SVG→PNG at AppImage build time via rsvg-convert/inkscape/ImageMagick with graceful fallback
  - libsherpa-onnx-c-api.so bundled as a warning (non-fatal) in build.sh to avoid breaking builds on machines where the .so is system-installed
  - APPIMAGE_EXTRACT_AND_RUN=1 set in CI workflow — required when running AppImage tools inside GitHub Actions where FUSE may not be available
  - release.yml installs linuxdeploy+appimagetool from their continuous releases rather than distro packages to get the latest stable versions
key_files:
  - assets/vibe-attack.svg
  - packaging/appimage/build.sh
  - packaging/appimage/vibe-attack.desktop
  - packaging/debian/control
  - packaging/debian/rules
  - packaging/debian/changelog
  - packaging/debian/compat
  - packaging/debian/copyright
  - packaging/PKGBUILD
  - packaging/vibe-attack.spec
  - .github/workflows/release.yml
lessons_learned:
  - Packaging in CI requires APPIMAGE_EXTRACT_AND_RUN=1 when linuxdeploy/appimagetool are themselves AppImages
  - debian/rules must be chmod +x — git tracks the executable bit, so it needs to be set before commit
  - RPM %check is the right place to document why audio hardware tests are skipped, not a comment in %install
---

# M005: Distribution Packaging

**Packaging infrastructure complete: AppImage, Debian, Arch, RPM, and GitHub Actions release CI all wired up with a consistent SVG icon**

## What Happened

M005 delivered all six planned slices in a single session. S01 created assets/vibe-attack.svg (crosshair/microphone design in navy+red) and wired it into all packaging paths. S02 completed build.sh with shared-ORT awareness (libsherpa-onnx-c-api.so bundled alongside libonnxruntime.so, AppRun sets LD_LIBRARY_PATH for both). S03 created packaging/debian/ with all five required Debian packaging files. S04 completed the PKGBUILD with real metadata, both binaries, and icon. S05 created packaging/vibe-attack.spec for Fedora/RHEL. S06 created .github/workflows/release.yml — on tag push it installs linuxdeploy+appimagetool, runs build.sh, renames the AppImage with the tag, and uploads via softprops/action-gh-release. All packaging formats share the same .desktop file and SVG icon.

## Success Criteria Results



## Definition of Done Results



## Requirement Outcomes



## Deviations

None.

## Follow-ups

["Compute real sha256sums for PKGBUILD once v0.1.0 tag is pushed to GitHub", "Test AppImage end-to-end on clean Ubuntu 22.04 and Arch machines", "Consider adding a test CI job (cargo test on ubuntu-latest) alongside the release job"]
