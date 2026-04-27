---
id: T02
parent: S01
milestone: M005
key_files:
  - packaging/appimage/build.sh
key_decisions:
  - (none)
duration: 
verification_result: untested
completed_at: 2026-04-27T00:37:50.336Z
blocker_discovered: false
---

# T02: Updated build.sh: SVG→PNG conversion via rsvg-convert/inkscape/convert, both binaries copied, libsherpa-onnx-c-api.so bundled

**Updated build.sh: SVG→PNG conversion via rsvg-convert/inkscape/convert, both binaries copied, libsherpa-onnx-c-api.so bundled**

## What Happened

Rewrote build.sh with: (1) cargo build --release --features gui to get vibe-attack-config, (2) find_so() helper that checks target/release then ldconfig then /usr for both ORT and sherpa .so files, (3) SVG→PNG conversion trying rsvg-convert then inkscape then ImageMagick convert with clear error on no converter, (4) vibe-attack-config copied alongside vibe-attack, (5) libsherpa-onnx-c-api.so bundled with a warning (not fatal) if not found.

## Verification

build.sh references assets/vibe-attack.svg, copies both binaries, bundles both .so files via find_so helper

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
