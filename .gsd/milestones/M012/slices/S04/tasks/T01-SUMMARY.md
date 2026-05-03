---
id: T01
parent: S04
milestone: M012
key_files:
  - src/ui/wizard.rs
key_decisions:
  - (none)
duration: 
verification_result: passed
completed_at: 2026-05-03T20:38:44.925Z
blocker_discovered: false
---

# T01: Rewrote wizard.rs with step strip, PTT drop-zone, and LED mic-test step

**Rewrote wizard.rs with step strip, PTT drop-zone, and LED mic-test step**

## What Happened

Rewrote wizard.rs: added a step indicator strip at the top showing numbered circles with amber fill for the active step and checkmarks for completed steps. PTT key capture step renders as a dashed rounded rect drop-zone with "Press any key" prompt; captures the next keydown event and shows the bound key as a kbd chip. Mic-test step renders an animated LED meter using widgets::led_meter() that pulses with the VAD energy level during recording.

## Verification

cargo build --features gui: 0 errors

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features gui` | 0 | pass | 44000ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/ui/wizard.rs`
