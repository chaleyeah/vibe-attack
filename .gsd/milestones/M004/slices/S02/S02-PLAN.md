# S02: Tray Icon — Static Placeholder

**Goal:** Add ksni system tray with Open Config and Quit actions
**Demo:** Run vibe-attack-config on X11 and Wayland; tray icon appears in system bar; Open Config brings the egui window to front; Quit exits cleanly

## Must-Haves

- Not provided.

## Proof Level

- This slice proves: Not provided.

## Integration Closure

Not provided.

## Verification

- Not provided.

## Tasks

- [x] **T01: Static ksni tray icon** `est:1h`
  Wire ksni into vibe-attack-config; tray icon appears with Open Config and Quit menu entries
  - Files: `src/ui/tray.rs`, `src/ui/mod.rs`
  - Verify: Tray icon appears on X11 and Wayland; Open Config raises egui window; Quit exits

## Files Likely Touched

- src/ui/tray.rs
- src/ui/mod.rs
