---
id: S02
parent: M013
milestone: M013
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
completed_at: 2026-05-03T20:46:42.715Z
blocker_discovered: false
---

# S02: Add AUR PKGBUILD validation to CI

**validate-pkgbuild CI job added; catches syntax and structural regressions on every tag push**

## What Happened

Added validate-pkgbuild job to ci.yml. Runs bash -n syntax check then sources the PKGBUILD in a subshell to verify all required fields and function definitions are present. Runs on ubuntu-22.04 without an Arch container — the checks are distro-agnostic bash.

## Verification

bash -n packaging/PKGBUILD exits 0; job present in ci.yml.

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

- `.github/workflows/ci.yml` — Added validate-pkgbuild job
