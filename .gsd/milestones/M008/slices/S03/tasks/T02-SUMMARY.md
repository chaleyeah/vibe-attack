---
id: T02
parent: S03
milestone: M008
key_files:
  - src/ui/tray.rs
key_decisions:
  - Collapsed Idle and Recording into a single match arm since both map to 'audio-input-microphone' — avoids redundancy while remaining exhaustive.
  - Placed icon_name_for_state above the VibeTray struct rather than as a method so it can be called from tests without constructing any tray state.
duration: 
verification_result: passed
completed_at: 2026-04-28T01:50:43.316Z
blocker_discovered: false
---

# T02: Extracted icon_name_for_state free function in tray.rs with per-state icon mapping and 5 unit tests covering all DaemonState variants

**Extracted icon_name_for_state free function in tray.rs with per-state icon mapping and 5 unit tests covering all DaemonState variants**

## What Happened

Extracted the inline icon-name match from `VibeTray::icon_name` into a new `pub(crate) fn icon_name_for_state(state: Option<&DaemonState>) -> &'static str` free function placed immediately above the tray impl block. The mapping follows the task spec exactly: `None` and `Some(Muted)` → `"audio-input-microphone-muted"`, `Some(Idle)` and `Some(Recording)` → `"audio-input-microphone"`, `Some(Listening)` → `"audio-input-microphone-high"`. The `Idle` and `Recording` arms share a single match branch since they map to the same icon. `VibeTray::icon_name` was updated to a one-liner calling `icon_name_for_state(self.current_state().daemon_state.as_ref()).into()`. Added a `#[cfg(test)] mod tests` block at the end of `tray.rs` with five unit tests (`icon_name_for_none_is_muted`, `icon_name_for_idle`, `icon_name_for_listening`, `icon_name_for_recording`, `icon_name_for_muted`), all calling the free function directly with no D-Bus, no async, and no ksni dependencies. Added a `///` doc comment on `icon_name_for_state` per MEM002 convention explaining the None-as-Muted rationale and the Listening distinction.

## Verification

Ran `cargo test --features gui icon_name_for` — 5/5 tests pass. Ran `cargo build` — clean. Ran `cargo build --features gui` — clean.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features gui icon_name_for` | 0 | ✅ pass — 5 tests passed (icon_name_for_none_is_muted, icon_name_for_idle, icon_name_for_listening, icon_name_for_recording, icon_name_for_muted) | 7660ms |
| 2 | `cargo build` | 0 | ✅ pass | 80ms |
| 3 | `cargo build --features gui` | 0 | ✅ pass | 4150ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/ui/tray.rs`
