---
id: T01
parent: S04
milestone: M004
key_files:
  - src/ui/tray.rs
key_decisions:
  - (none)
duration: 
verification_result: passed
completed_at: 2026-04-27T00:30:36.419Z
blocker_discovered: false
---

# T01: Profile submenu lists XDG profiles with active checkmark; selecting fires SwitchProfile

**Profile submenu lists XDG profiles with active checkmark; selecting fires SwitchProfile**

## What Happened

Profile list read from XDG config dir at menu-open time (no cache). Active profile gets checkmark. Selecting another profile fires SwitchProfile on a dedicated OS thread so the ksni callback never blocks.

## Verification

Profile submenu shows installed profiles with active checkmark; switching sends SwitchProfile and daemon logs the switch — manually verified

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `manual: open tray profile submenu with hd2 installed, verify checkmark and profile switch` | 0 | pass | 0ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/ui/tray.rs`
