---
id: S01
parent: M003
milestone: M003
provides:
  - ["scripts/setup.sh with step names: copy_config, install_model, setup_uinput, validate", "Step names are the canonical identifiers S02 probe checks and S03 wizard panels will reference"]
requires:
  []
affects:
  []
key_files:
  - (none)
key_decisions:
  - ["--step=NAME flag added for S03 wizard panels to shell out to individual steps", "Used 'input' group (not 'uinput') consistent with docs/uinput-setup.md and systemd v258+ behaviour", "PTT key detection in validate uses grep -qE for 'key: KEY_*' pattern matching config.example.yaml format"]
patterns_established:
  - ["run_cmd wrapper: routes commands through dry-run print or real execution — reuse this pattern in any future scripts that need --dry-run support", "ask_confirm: respects --yes flag and prompts otherwise — same pattern for any interactive script step"]
observability_surfaces:
  - ["Each step prints '[step_name] ...' status prefix", "Failures print step name and reason to stderr", "validate prints a pass/fail table before exit code"]
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-26T00:11:33.773Z
blocker_discovered: false
---

# S01: Setup script

**Delivered scripts/setup.sh — a self-contained, idempotent setup script covering config copy, model download, uinput setup, and validation with --yes/--dry-run/--step support**

## What Happened

Created scripts/setup.sh from scratch with bash strict mode. Implements four ordered steps: copy_config (XDG-aware, idempotent), install_model (curl download with size check skip), setup_uinput (modprobe + input group, systemd v258+ note), and validate (four-check summary table with named failures). CLI flags: --yes (non-interactive), --dry-run (preview via run_cmd wrapper), --step=NAME (single-step dispatch for wizard integration). All sudo commands route through ask_confirm and run_cmd so they respect both flags. Colour output degrades gracefully on non-TTY stdout.

## Verification

bash -n exits 0; --help exits 0; copy_config idempotent across two runs in temp XDG dir; dry-run creates no files; validate shows correct per-check pass/fail with named failure reasons; uinput step skips cleanly when prerequisites already met

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

- `scripts/setup.sh` — New: complete setup script with all four steps
