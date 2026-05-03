---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Rewrite show_main_config() with rail nav and panes

Rewrite config_app.rs and vibe-attack-config.rs: rail navigation (Devices/Voice/Packs/Hotkeys/Advanced), themed panes, daemon-disconnected banner

## Inputs

- `src/ui/theme.rs`
- `src/ui/widgets.rs`

## Expected Output

- `All 5 nav panes render`
- `Banner appears when daemon not running`

## Verification

cargo build --features gui succeeds; config app launches
