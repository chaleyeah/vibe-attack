---
id: T03
parent: S05
milestone: M009
key_files:
  - src/ui/pack_editor.rs
  - src/bin/vibe-attack-config.rs
key_decisions:
  - Test button uses add_enabled(daemon_running, ...) to grey out when daemon is not running, matching the existing daemon_running field already polled each frame in vibe-attack-config.rs
  - Timer fire logic uses pending_test.take() before the UI render so a single frame cannot both fire and redisplay the countdown — the value is put back if the countdown is still running
  - last_test_status display lives at the bottom of show_pack_editor (below last_error) rather than inside the right-column vertical, matching the task plan placement requirement
duration: 
verification_result: passed
completed_at: 2026-04-28T03:20:15.926Z
blocker_discovered: false
---

# T03: Added Test button with 1-second confirmation countdown to pack editor, gated on daemon_running, firing ControlRequest::TestMacro and surfacing the result inline

**Added Test button with 1-second confirmation countdown to pack editor, gated on daemon_running, firing ControlRequest::TestMacro and surfacing the result inline**

## What Happened

Added two new fields to `PackEditorState` inside the `#[cfg(feature = "gui")]` `inner` mod: `pending_test: Option<(String, Instant)>` (with a safety-rationale comment) and `last_test_status: Option<String>`, both initialized to `None` in `new()`.

Updated `show_pack_editor` to accept a new `daemon_running: bool` parameter. The function now does three new things each frame:

1. **Timer/fire logic** (runs before the CRUD buttons): if `pending_test` is `Some`, it checks elapsed time. If ≥ 1 second, it calls `send_command(ControlRequest::TestMacro { name })`, matches on `Ok(ControlResponse::Ok)`, `Ok(ControlResponse::Error { message })`, unexpected `Ok` variants, and `Err(e)`, setting `last_test_status` appropriately and emitting `tracing::info!`/`tracing::warn!` with macro name and outcome. If elapsed < 1 second, it puts the value back and calls `ui.ctx().request_repaint_after(Duration::from_millis(50))` for smooth countdown animation.

2. **Test UI row** (below Remove Macro): when `selected_macro.is_some()`, renders either (a) a greyed-out (when daemon not running) `Test` button that starts the countdown on click, or (b) a `Firing in N.Ns…` countdown label plus a `Cancel` button.

3. **Test status display** (below `last_error` at the bottom of the panel): renders `last_test_status` as green (`Fired: …`) or red (`Test failed: …` / `Daemon error: …`).

Updated the single call site in `src/bin/vibe-attack-config.rs` line 437 to pass `app.config.daemon_running` as the third argument. `ControlResponse` was added to the import alongside `ControlRequest`. `std::time::{Duration, Instant}` was added to imports inside `inner`.

## Verification

Ran `RUSTFLAGS="-D warnings" cargo build --features gui --bin vibe-attack-config` — compiled clean with no warnings. Ran `RUSTFLAGS="-D warnings" cargo build` (default/headless build) — clean. Ran `cargo test -- --test-threads=1` — 90 unit tests + all integration suites passed (0 failed). The new fields and render path are GUI-gated and do not affect the default-feature build or any existing tests.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `RUSTFLAGS="-D warnings" cargo build --features gui --bin vibe-attack-config` | 0 | ✅ pass | 8130ms |
| 2 | `RUSTFLAGS="-D warnings" cargo build` | 0 | ✅ pass | 7400ms |
| 3 | `cargo test -- --test-threads=1` | 0 | ✅ pass — 90 unit tests + integration suites, 0 failures | 2360ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/ui/pack_editor.rs`
- `src/bin/vibe-attack-config.rs`
