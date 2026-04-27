---
id: S03
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
completed_at: 2026-04-27T00:40:58.954Z
blocker_discovered: false
---

# S03: Debian package

**packaging/debian/ directory created and ready for dpkg-buildpackage**

## What Happened

Created packaging/debian/ with all five required files: control (Depends: libasound2, Section: games), rules (executable, two cargo build steps, installs both binaries + SVG icon + .desktop + README), changelog (0.1.0-1), compat (13), copyright (AGPL-3.0-only).

## Verification

debian/control has libasound2 dependency; rules is executable; both binaries and SVG icon in install section

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

- `packaging/debian/control` — Package metadata, Depends, BuildDepends
- `packaging/debian/rules` — Build and install rules, executable
- `packaging/debian/changelog` — Debian changelog
- `packaging/debian/compat` — Debhelper compat level 13
- `packaging/debian/copyright` — AGPL-3.0-only copyright
