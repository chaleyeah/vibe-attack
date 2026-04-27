# S02: AppImage complete build

**Goal:** AppImage build script complete: both binaries, both .so deps, icon, working AppDir
**Demo:** ./packaging/appimage/build.sh exits 0; produced AppImage passes --appimage-extract sanity check showing both binaries and .so files

## Must-Haves

- Not provided.

## Proof Level

- This slice proves: Not provided.

## Integration Closure

Not provided.

## Verification

- Not provided.

## Tasks

- [x] **T01: Complete build.sh** `est:20m`
  Both binaries, SVG→PNG, libsherpa-onnx-c-api.so bundling, AppRun LD_LIBRARY_PATH
  - Files: `packaging/appimage/build.sh`
  - Verify: build.sh exits 0 on machine with cargo + rsvg-convert

## Files Likely Touched

- packaging/appimage/build.sh
