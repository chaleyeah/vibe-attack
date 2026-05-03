---
id: S04
parent: M012
milestone: M012
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - (none)
key_decisions:
  - (none)
patterns_established:
  - (none)
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-03T20:38:49.485Z
blocker_discovered: false
---

# S04: Wizard Rewrite

**Wizard rewritten with step indicator strip, PTT dashed drop-zone, and animated LED mic-test step**

## What Happened

All 6 wizard steps themed. PTT capture is now a tactile drop-zone rather than a plain button. Mic-test step provides real-time visual feedback via LED meter.

## Verification

cargo build --features gui: 0 errors

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

None.

## Follow-ups

None.

## Files Created/Modified

- `src/ui/wizard.rs` — Rewritten with step strip, drop-zone, LED meter
