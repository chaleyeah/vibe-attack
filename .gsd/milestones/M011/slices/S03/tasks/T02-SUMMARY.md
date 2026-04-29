---
id: T02
parent: S03
milestone: M011
key_files:
  - src/ui/tray.rs
  - src/bin/vibe-attack-config.rs
key_decisions:
  - quit_window uses the same Arc<AtomicBool> + take pattern as open_window so the eframe loop owns the poll — no D-Bus or blocking calls in the shutdown path
  - tooltip_description_for extracted as free pub(crate) fn (not a method) per MEM045 convention — testable without constructing tray or D-Bus
  - Early return after ViewportCommand::Close prevents the rest of the frame from drawing after a quit request
duration: 
verification_result: passed
completed_at: 2026-04-29T01:43:33.772Z
blocker_discovered: false
---

# T02: Fixed tray Quit bypass (process::exit → flag), added mode-aware tooltip via tooltip_description_for free function, and wired take_quit_request into the eframe loop

**Fixed tray Quit bypass (process::exit → flag), added mode-aware tooltip via tooltip_description_for free function, and wired take_quit_request into the eframe loop**

## What Happened

Two tray-side bugs from S03-RESEARCH were addressed.

**Bug 1 — Quit bypassed eframe shutdown via std::process::exit(0):**
Added `quit_window: Arc<AtomicBool>` to both `TrayHandle` (public field, line 38) and `VibeTray` (private field, line 191). `TrayHandle::spawn()` initializes it alongside `open_window` and clones it into the `VibeTray` construction. The Quit menu item's `activate` closure now captures `Arc::clone(&self.quit_window)` as `quit_flag` and calls `quit_flag.store(true, Ordering::Release)` with a `tracing::info!("Tray quit requested")` log line — mirroring the open_window pattern exactly. `pub fn take_quit_request(&self) -> bool` was added to `TrayHandle` mirroring `take_open_request`. In `vibe-attack-config.rs`, two lines were added immediately after the existing `take_open_request` block: the guard calls `take_quit_request()` and sends `ViewportCommand::Close` with an early `return` to skip the rest of the frame.

**Bug 2 — Tooltip ignored active_mode:**
The hardcoded `"Idle — listening for wake word"` string in `tool_tip()` was wrong in PTT mode. Extracted `pub(crate) fn tooltip_description_for(state: Option<&DaemonState>, mode: Option<&ActivationMode>) -> String` (lines 170–185), following the MEM045 free-function convention established by `icon_name_for_state`. The Idle arm branches on mode: PTT → "Idle — waiting for PTT key"; Wake → "Idle — listening for wake word"; None → "Idle". All other states (Muted, Listening, Recording) are unaffected by mode. `tool_tip()` now calls the free function with `s.daemon_state.as_ref()` and `s.active_mode.as_ref()`.

Five new unit tests were added to the existing `#[cfg(test)] mod tests` block: `tooltip_description_idle_ptt`, `tooltip_description_idle_wake`, `tooltip_description_idle_unknown`, `tooltip_description_recording_unaffected_by_mode`, and `tray_handle_take_quit_request_clears_flag`. The last test directly verifies the `swap(false, AcqRel)` take pattern without spawning any real tray or D-Bus connection.

## Verification

Ran `cargo test --features gui --lib ui::tray:: -- --test-threads=1`: all 10 tests pass (5 pre-existing icon_name tests + 5 new tooltip/quit tests). Ran `cargo build --release --features gui --bin vibe-attack-config`: clean build in 6.25s, no warnings or errors.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features gui --lib ui::tray:: -- --test-threads=1` | 0 | ✅ pass — 10/10 tests ok | 2910ms |
| 2 | `cargo build --release --features gui --bin vibe-attack-config` | 0 | ✅ pass — clean release build | 6250ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/ui/tray.rs`
- `src/bin/vibe-attack-config.rs`
