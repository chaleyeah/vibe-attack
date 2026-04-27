---
id: S02
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
completed_at: 2026-04-27T00:40:51.545Z
blocker_discovered: false
---

# S02: AppImage complete build

**build.sh produces complete AppDir with both binaries, both .so files, icon conversion, and working AppRun**

## What Happened

Updated packaging/appimage/build.sh to fully support the shared-ORT architecture: builds with --features gui to get vibe-attack-config, bundles both libonnxruntime.so and libsherpa-onnx-c-api.so via a find_so() helper (target/release → ldconfig → /usr search), converts SVG icon to PNG via rsvg-convert/inkscape/ImageMagick, copies both binaries, and sets LD_LIBRARY_PATH in AppRun for both .so files.

## Verification

build.sh contains both binary copies, find_so() for both .so files, SVG→PNG conversion logic, and AppRun LD_LIBRARY_PATH

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

- `packaging/appimage/build.sh` — Both binaries, sherpa .so, SVG→PNG, find_so() helper
