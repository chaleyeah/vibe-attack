---
id: T01
parent: S01
milestone: M014
key_files:
  - src/ui/widgets.rs
key_decisions:
  - (none)
duration: 
verification_result: passed
completed_at: 2026-06-07T21:07:06.363Z
blocker_discovered: false
---

# T01: Replaced broken ⊞ nav icon with 🖥 emoji in widgets.rs

**Replaced broken ⊞ nav icon with 🖥 emoji in widgets.rs**

## What Happened

The DEVICES nav item used U+229E "⊞" (squared plus, a mathematical operator) which is not present in egui's bundled NotoEmoji font, rendering as a missing-glyph square. Replaced with 🖥 (desktop computer emoji) which is in NotoEmoji alongside the other emoji icons (🎤, 📦). Build verified clean.

## Verification

cargo build --features gui exited 0

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features gui` | 0 | pass | 16550ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/ui/widgets.rs`
