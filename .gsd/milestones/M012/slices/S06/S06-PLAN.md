# S06: Tray Icon Update

**Goal:** Update tray.rs to return palette-matched ARGB32 status dots via icon_pixmap()
**Demo:** System tray icon changes color correctly when daemon state changes.

## Must-Haves

- Not provided.

## Proof Level

- This slice proves: Not provided.

## Integration Closure

Not provided.

## Verification

- Not provided.

## Tasks

- [x] **T01: Implement palette-matched status dot tray icons** `est:1h`
  Rewrite icon_pixmap() in tray.rs to generate 16x16 ARGB32 status dots in green/amber/red/gray from theme palette constants
  - Files: `src/ui/tray.rs`
  - Verify: cargo build succeeds

## Files Likely Touched

- src/ui/tray.rs
