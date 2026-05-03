---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Implement palette-matched status dot tray icons

Rewrite icon_pixmap() in tray.rs to generate 16x16 ARGB32 status dots in green/amber/red/gray from theme palette constants

## Inputs

- `src/ui/theme.rs`

## Expected Output

- `icon_pixmap() returns correct color for each daemon state`

## Verification

cargo build succeeds
