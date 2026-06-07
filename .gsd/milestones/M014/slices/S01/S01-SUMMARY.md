---
id: S01
parent: M014
milestone: M014
provides:
  - (none)
requires:
  []
affects:
  []
key_files: []
key_decisions: []
patterns_established:
  - (none)
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-06-07T21:07:12.878Z
blocker_discovered: false
---

# S01: Fix broken DEVICES nav icon

**Replaced unrenderable ⊞ math symbol with 🖥 emoji in the DEVICES nav item**

## What Happened

Single-file change in src/ui/widgets.rs:66. The original U+229E character is not in egui's bundled font and renders as a broken box. The replacement 🖥 is in NotoEmoji and visually consistent with the other nav icons.

## Verification

cargo build --features gui passed

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

- `src/ui/widgets.rs` — Replaced U+229E ⊞ with 🖥 emoji for DEVICES nav icon
