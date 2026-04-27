---
id: S04
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
completed_at: 2026-04-27T00:41:10.638Z
blocker_discovered: false
---

# S04: Arch PKGBUILD complete

**PKGBUILD updated with real URL, maintainer, both binaries, and icon — AUR-ready**

## What Happened

Updated packaging/PKGBUILD: real maintainer and url set to github.com/chaleyeah/vibe-attack, two cargo build steps to build both daemon and GUI binary, package() installs both binaries and the SVG icon to hicolor/scalable/apps/.

## Verification

PKGBUILD url= and source= reference chaleyeah/vibe-attack; both vibe-attack and vibe-attack-config in package(); SVG icon installed

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

- `packaging/PKGBUILD` — Real URL/maintainer, both binaries, SVG icon
