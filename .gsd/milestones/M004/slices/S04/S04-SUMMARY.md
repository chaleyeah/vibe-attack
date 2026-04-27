---
id: S04
parent: M004
milestone: M004
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
completed_at: 2026-04-27T00:30:55.765Z
blocker_discovered: false
---

# S04: Profile Switcher in Tray

**Profile submenu lists XDG profiles with active checkmark; switching sends SwitchProfile to daemon**

## What Happened

Profiles read from XDG config dir at menu-open time. Active profile marked with checkmark. Switching fires on OS thread to avoid blocking ksni callback.

## Verification

Profile submenu shows installed profiles with checkmark; switch verified with daemon logging

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

- `src/ui/tray.rs` — 
