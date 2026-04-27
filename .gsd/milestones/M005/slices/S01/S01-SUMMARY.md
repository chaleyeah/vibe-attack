---
id: S01
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
completed_at: 2026-04-27T00:38:06.562Z
blocker_discovered: false
---

# S01: Icon and shared assets

**SVG icon created; build.sh updated to bundle both binaries and both .so files; .desktop file cleaned up**

## What Happened

Created assets/vibe-attack.svg (256x256 dark navy + red crosshair/mic design). Updated build.sh to: build with --features gui, copy both vibe-attack and vibe-attack-config, convert SVG to PNG via rsvg-convert/inkscape/ImageMagick, and bundle libsherpa-onnx-c-api.so alongside libonnxruntime.so. Added StartupWMClass to .desktop file.

## Verification

assets/vibe-attack.svg exists; build.sh references both .so files and both binaries; .desktop has StartupWMClass

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

- `assets/vibe-attack.svg` — New SVG icon
- `packaging/appimage/build.sh` — Both binaries, SVG→PNG conversion, sherpa .so bundling
- `packaging/appimage/vibe-attack.desktop` — Added StartupWMClass
