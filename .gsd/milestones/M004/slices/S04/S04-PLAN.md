# S04: Profile Switcher in Tray

**Goal:** Profile switcher submenu in tray with active checkmark
**Demo:** With hd2 profile installed: tray Profiles submenu shows 'hd2' with a checkmark. Selecting a different profile (if present) sends LOAD_PROFILE and the daemon logs the switch.

## Must-Haves

- Not provided.

## Proof Level

- This slice proves: Not provided.

## Integration Closure

Not provided.

## Verification

- Not provided.

## Tasks

- [x] **T01: Profile submenu** `est:1h`
  Read profiles from XDG config dir at menu-open time; active profile gets checkmark; selecting another fires SwitchProfile on OS thread
  - Files: `src/ui/tray.rs`
  - Verify: Profile submenu lists installed profiles; active profile shows checkmark; switching sends SwitchProfile

## Files Likely Touched

- src/ui/tray.rs
