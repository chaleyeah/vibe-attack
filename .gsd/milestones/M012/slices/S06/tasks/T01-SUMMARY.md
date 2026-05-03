---
id: T01
parent: S06
milestone: M012
key_files:
  - src/ui/tray.rs
key_decisions:
  - (none)
duration: 
verification_result: passed
completed_at: 2026-05-03T20:39:18.423Z
blocker_discovered: false
---

# T01: Rewrote icon_pixmap() to generate palette-matched ARGB32 status dots

**Rewrote icon_pixmap() to generate palette-matched ARGB32 status dots**

## What Happened

Rewrote icon_pixmap() in tray.rs to generate 16x16 ARGB32 bitmaps programmatically using the theme palette color values. Green dot for running/active state, amber for muted/PTT-held, red for error, gray for disconnected. Colors are pulled directly from theme::STATUS_GREEN, STATUS_AMBER, STATUS_RED, STATUS_GRAY constants so tray icons automatically stay in sync if the palette changes.

## Verification

cargo build: 0 errors (tray doesn't require --features gui)

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build` | 0 | pass | 40000ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/ui/tray.rs`
