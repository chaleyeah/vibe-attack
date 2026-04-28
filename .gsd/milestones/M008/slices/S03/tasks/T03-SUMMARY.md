---
id: T03
parent: S03
milestone: M008
key_files:
  - src/ui/tray.rs
key_decisions:
  - Extracted new_active_mode with status.as_ref().map() before status.and_then() consumes the value — preserves the single query_status() call per tick without cloning the whole struct.
  - Mode submenu uses the same fire-and-forget std::thread::spawn pattern as Mute/Profile items — ksni activate callbacks must not block the D-Bus event loop.
  - active_mode: Option<ActivationMode> in TrayState (not ActivationMode directly) so None cleanly represents daemon-not-running without a sentinel value.
duration: 
verification_result: passed
completed_at: 2026-04-28T01:53:33.915Z
blocker_discovered: false
---

# T03: Added active_mode to TrayState, polled it from query_status, and rendered a Mode submenu with PTT/Wake checkmark items that dispatch SetMode fire-and-forget

**Added active_mode to TrayState, polled it from query_status, and rendered a Mode submenu with PTT/Wake checkmark items that dispatch SetMode fire-and-forget**

## What Happened

Three changes to `src/ui/tray.rs`:

1. **Import**: Added `ActivationMode` to the existing `crate::control::protocol` import so the type is available in the tray module.

2. **TrayState field**: Extended the `TrayState` struct with `active_mode: Option<ActivationMode>` (derives `Default` → `None`, which represents daemon stopped or unknown — matching the plan's intent).

3. **Poll loop**: In the tokio poll task, `query_status()` is called once and held as `status`. `new_active_mode` is extracted with `status.as_ref().map(|s| s.active_mode.clone())` before the existing `status.and_then(|s| s.active_profile)` consumes the value. Added `|| s.active_mode != new_active_mode` to the `changed` guard, and `s.active_mode = new_active_mode.clone()` inside the `update` closure — exactly mirroring the pattern used for `active_profile`.

4. **Mode submenu**: After the Profiles `SubMenu` push and before the separator+Quit, added two locals `is_ptt` and `is_wake` (boolean comparisons against `state.active_mode`). Built `mode_submenu` as a `vec!` of two `CheckmarkItem`s — "Push-to-talk" (checked when `is_ptt`) and "Wake word" (checked when `is_wake`), both `enabled: daemon_running`. Each `activate` closure spawns a fresh thread and calls `send_command(ControlRequest::SetMode { mode: ... })` fire-and-forget, matching the established Mute/Profile pattern so ksni callbacks never block. Pushed as a `SubMenu { label: "Mode", ... }` followed by a `MenuItem::Separator` before Quit.

When `state.active_mode` is `None` (daemon not running) neither item is checked — the acceptable fallback noted in the plan. `daemon_running` is already `false` in that case so both items are greyed out.

## Verification

cargo test --features gui: 63 tests passed, 0 failed (5 ignored for privileged/heavy resources). cargo build (default features): clean compile. cargo build --features gui: clean compile. All three verification commands from the task plan passed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features gui` | 0 | ✅ pass | 7420ms |
| 2 | `cargo build` | 0 | ✅ pass | 80ms |
| 3 | `cargo build --features gui` | 0 | ✅ pass | 5590ms |

## Deviations

none

## Known Issues

none — menu layout is exercised manually in S04 UAT; D-Bus is not available in CI so no automated menu rendering test is possible.

## Files Created/Modified

- `src/ui/tray.rs`
