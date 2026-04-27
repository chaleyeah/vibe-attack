---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Poll loop and live tray state

Add TrayState behind tokio Mutex; 1s poll loop queries DaemonHandle and updates icon/tooltip; Mute/Unmute item conditional on daemon running

## Inputs

- `VibeTray from S02`
- `DaemonHandle from S01`

## Expected Output

- `Live icon/tooltip driven by DaemonState`

## Verification

Mute/unmute toggle verified; tray updates within ~1s of daemon state change
