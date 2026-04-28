---
id: T02
parent: S04
milestone: M008
key_files:
  - .gsd/milestones/M008/slices/S04/S04-UAT.md
key_decisions:
  - UAT assertions use exact log strings from source (SetMode: cached active_mode=...) rather than approximate descriptions, so testers can grep the log file unambiguously
  - Called out ActivationMode runtime-only limitation (no YAML persistence in M008) to prevent tester confusion after daemon restart
  - Pass/fail heading uses lowercase 'f' to match the slice verification grep pattern exactly
duration: 
verification_result: passed
completed_at: 2026-04-28T02:05:06.329Z
blocker_discovered: false
---

# T02: Authored .gsd/milestones/M008/slices/S04/S04-UAT.md with five numbered test scenarios covering config-window mode switch, tray mode round-trip, threshold change, stratagem dispatch, and icon state transitions — all with exact log lines from source

**Authored .gsd/milestones/M008/slices/S04/S04-UAT.md with five numbered test scenarios covering config-window mode switch, tray mode round-trip, threshold change, stratagem dispatch, and icon state transitions — all with exact log lines from source**

## What Happened

Read `src/ui/tray.rs`, `src/ui/config_app.rs`, and `src/control/mod.rs` before writing to extract the exact log strings and UI labels used in the running daemon rather than paraphrasing from memory.

Key source facts extracted:
- SetMode handler logs: `SetMode: cached active_mode={mode:?}, forwarding to coordinator` (tracing::debug, control/mod.rs:170)
- Control listener start: `Control channel listening on: {socket_path}` (tracing::info, control/mod.rs:123)
- Tray menu labels are exactly `Push-to-talk` and `Wake word` (tray.rs:303, 309)
- Tray menu structure: Open Config → Mute/Unmute → Profiles submenu → Mode submenu → Quit (tray.rs:199–344)
- Icon names from `icon_name_for_state`: None/Muted → `audio-input-microphone-muted`, Idle/Recording → `audio-input-microphone`, Listening → `audio-input-microphone-high` (tray.rs:147–153)
- ActivationMode is runtime-only in M008 — save_app_to_config does NOT persist it to YAML (config_app.rs:114 comment, MEM036)
- SetInputDevice/SetPttBinding require restart — logged in control/mod.rs:178–181

The UAT doc is structured as five runnable test scenarios plus a Pass/fail checklist and Known UAT Limitations section. Each assertion references a specific, observable log line or visible UI state rather than vague prose. The mode-not-persisted limitation is called out explicitly to prevent tester confusion after a daemon restart.

## Verification

Ran the exact slice verification command: `test -f .gsd/milestones/M008/slices/S04/S04-UAT.md && [ $(wc -l < .gsd/milestones/M008/slices/S04/S04-UAT.md) -gt 30 ] && grep -q 'Pass/fail' .gsd/milestones/M008/slices/S04/S04-UAT.md && grep -q 'Preconditions' .gsd/milestones/M008/slices/S04/S04-UAT.md && grep -q 'SetMode' .gsd/milestones/M008/slices/S04/S04-UAT.md` — printed ALL CHECKS PASSED. File is 186 lines.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `test -f .gsd/milestones/M008/slices/S04/S04-UAT.md && [ $(wc -l < .gsd/milestones/M008/slices/S04/S04-UAT.md) -gt 30 ] && grep -q 'Pass/fail' .gsd/milestones/M008/slices/S04/S04-UAT.md && grep -q 'Preconditions' .gsd/milestones/M008/slices/S04/S04-UAT.md && grep -q 'SetMode' .gsd/milestones/M008/slices/S04/S04-UAT.md && echo ALL CHECKS PASSED` | 0 | ✅ pass | 15ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `.gsd/milestones/M008/slices/S04/S04-UAT.md`
