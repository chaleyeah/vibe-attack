---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Profile submenu

Read profiles from XDG config dir at menu-open time; active profile gets checkmark; selecting another fires SwitchProfile on OS thread

## Inputs

- `VibeTray from S03`
- `DaemonHandle SwitchProfile command`

## Expected Output

- `Profiles submenu with active checkmark`

## Verification

Profile submenu lists installed profiles; active profile shows checkmark; switching sends SwitchProfile
