---
id: T01
parent: S02
milestone: M005
key_files:
  - packaging/appimage/build.sh
key_decisions:
  - (none)
duration: 
verification_result: untested
completed_at: 2026-04-27T00:40:08.482Z
blocker_discovered: false
---

# T01: build.sh updated: both binaries, SVG→PNG conversion, libsherpa-onnx-c-api.so bundled, AppRun sets LD_LIBRARY_PATH for both .so files

**build.sh updated: both binaries, SVG→PNG conversion, libsherpa-onnx-c-api.so bundled, AppRun sets LD_LIBRARY_PATH for both .so files**

## What Happened

Rewrote packaging/appimage/build.sh to build with --features gui (for vibe-attack-config), extract both binaries, convert SVG icon to PNG via rsvg-convert/inkscape/ImageMagick with graceful fallback, and bundle both libonnxruntime.so and libsherpa-onnx-c-api.so via a find_so() helper. AppRun sets LD_LIBRARY_PATH to the AppDir usr/lib so both .so files are found at runtime inside the FUSE mount.

## Verification

YAML valid; build.sh references both binaries and both .so files; find_so() helper checks target/release then ldconfig then /usr

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| — | No verification commands discovered | — | — | — |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `packaging/appimage/build.sh`
