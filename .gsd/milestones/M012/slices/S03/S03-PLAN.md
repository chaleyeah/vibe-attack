# S03: ConfigApp Rewrite

**Goal:** Rewrite ConfigApp (config_app.rs and vibe-attack-config.rs) using theme and widget primitives
**Demo:** Config app launches, all 5 panes navigable, disconnected banner appears when daemon is not running.

## Must-Haves

- Not provided.

## Proof Level

- This slice proves: Not provided.

## Integration Closure

Not provided.

## Verification

- Not provided.

## Tasks

- [x] **T01: Rewrite show_main_config() with rail nav and panes** `est:3h`
  Rewrite config_app.rs and vibe-attack-config.rs: rail navigation (Devices/Voice/Packs/Hotkeys/Advanced), themed panes, daemon-disconnected banner
  - Files: `src/ui/config_app.rs`, `src/bin/vibe-attack-config.rs`
  - Verify: cargo build --features gui succeeds; config app launches

## Files Likely Touched

- src/ui/config_app.rs
- src/bin/vibe-attack-config.rs
