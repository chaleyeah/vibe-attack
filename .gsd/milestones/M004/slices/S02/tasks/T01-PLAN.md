---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Static ksni tray icon

Wire ksni into vibe-attack-config; tray icon appears with Open Config and Quit menu entries

## Inputs

- `ksni crate`
- `DaemonHandle from S01`

## Expected Output

- `VibeTray struct with static menu`

## Verification

Tray icon appears on X11 and Wayland; Open Config raises egui window; Quit exits
