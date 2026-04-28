---
id: S05
parent: M009
milestone: M009
provides:
  - ["Dispatcher::fire_named — direct macro trigger bypassing phrase matching", "ControlRequest::TestMacro handler — socket-to-dispatcher round trip", "Pack editor Test button — 1-second safety countdown with cancel, status feedback, and daemon-running gate"]
requires:
  []
affects:
  []
key_files:
  - (none)
key_decisions:
  - ["score=1.0 in fire_named marks direct control-plane triggers vs fuzzy phrase-matched scores for JSONL event consumer disambiguation", "block_in_place requires multi_thread Tokio flavor — TestMacro integration tests use #[tokio::test(flavor = \"multi_thread\", worker_threads = 2)]", "Catch-all match arm removed (not kept) when TestMacro became the last unhandled variant — unreachable_patterns is an error under -D warnings", "Test button uses Instant polling from eframe loop (never sleep) with request_repaint_after(50ms) for smooth countdown animation", "daemon_running: bool threaded into show_pack_editor as a new parameter to gate Test button via add_enabled"]
patterns_established:
  - ["Control-plane fire-by-name path: send_command(TestMacro{name}) → Unix socket → handler block_in_place → Dispatcher::fire_named → MacroCmd::Execute → uinput", "UI safety countdown: pending_test: Option<(String, Instant)> with Instant polling, no sleep, request_repaint_after(50ms)", "Test helper pattern: make_dispatcher_with_keys returns (Dispatcher, Receiver<MacroCmd>) for channel-assertion tests"]
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-28T03:22:19.211Z
blocker_discovered: false
---

# S05: TriggerMacro control request + editor Test button

**Wired Dispatcher::fire_named through the TestMacro control-plane handler and added a 1-second safety-countdown Test button to the pack editor, enabling end-to-end macro testing without speaking a phrase.**

## What Happened

S05 delivered three coordinated changes that together close the loop from editor UI to uinput key injection via the control plane.

**T01 — Dispatcher::fire_named (src/pipeline/dispatcher.rs)**
Added `pub fn fire_named(&self, name: &str) -> Result<DispatchOutcome, String>` to the Dispatcher. The method bypasses phrase matching entirely: it acquires a read lock on `self.macros`, locates the first MacroConfig whose name matches the argument, plays the optional sound (same code path as `process()`), builds a `Vec<KeyStep>` via `KeyStep::from_config`, and sends `MacroCmd::Execute` over `self.macro_tx`. On success it returns `Ok(DispatchOutcome::Fired { macro_id, score: 1.0 })` — score 1.0 is a deliberate convention distinguishing direct control-plane triggers from fuzzy phrase matches in downstream JSONL event consumers. On name-not-found it returns `Err("macro not found: {name}")`. A `tracing::info!(macro_name, "Firing macro (direct)")` log before the send makes the trigger source visible in journalctl. Two unit tests were added using a new `make_dispatcher_with_keys` test helper that exposes the MacroCmd receiver (unlike the existing `make_dispatcher` which discards `_rx`): `fire_named_found_emits_execute` asserts one MacroCmd::Execute with matching keys length; `fire_named_missing_returns_err` asserts zero channel messages on name miss.

**T02 — TestMacro handler + integration tests (src/control/mod.rs, tests/control_integration.rs)**
Replaced the catch-all `_ => ControlResponse::Error { message: "Not yet implemented" }` arm with an explicit `ControlRequest::TestMacro { name }` arm. The arm mirrors the SwitchProfile pattern: emits a tracing::info! log for journalctl visibility, then calls `tokio::task::block_in_place(|| h.dispatcher.fire_named(&name))` so the synchronous RwLock read does not block the Tokio executor. Ok maps to ControlResponse::Ok; Err maps to ControlResponse::Error. The catch-all was removed entirely because TestMacro was the last unhandled variant — keeping it would have triggered unreachable_patterns under -D warnings. A key deviation from the plan: the two new integration tests (`test_macro_via_socket_fires_dispatcher` and `test_macro_unknown_name_returns_error`) required `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]` because `block_in_place` panics on the default single-threaded runtime.

**T03 — Test button with 1-second countdown (src/ui/pack_editor.rs, src/bin/vibe-attack-config.rs)**
Added `pending_test: Option<(String, Instant)>` and `last_test_status: Option<String>` fields to `PackEditorState` inside the gui-gated `mod inner`. The Test button renders alongside Update Macro / Remove Macro when a macro is selected and no test is pending; clicking it sets `pending_test = Some((name, Instant::now()))`. While pending, a non-clickable countdown label ("Firing in N.Ns...") and a Cancel button replace the Test button. Each frame the editor panel is visible with a pending test, `request_repaint_after(50ms)` drives smooth animation. When `elapsed >= 1s`, the pending name is taken out of the Option and `send_command(ControlRequest::TestMacro { name })` is called; the result (Ok/Error/network failure) is stored in `last_test_status` and rendered as a colored label below the inline error display. The Test button is gated by `ui.add_enabled(daemon_running, ...)` — when the daemon is not running the button is greyed out. The `daemon_running: bool` parameter was threaded into `show_pack_editor` and the single call site in `vibe-attack-config.rs` was updated to pass `app.config.daemon_running`. No egui types leak into the default-feature build.

## Verification

1. `cargo test --lib pipeline::dispatcher -- --test-threads=1` → 6/6 tests pass (4 pre-existing threshold tests + 2 new fire_named tests). Exit 0.
2. `cargo test --test control_integration -- --test-threads=1` → 4/4 tests pass (2 pre-existing + 2 new TestMacro tests). Exit 0.
3. `cargo test -- --test-threads=1` → all test suites pass with 0 failures. Exit 0.
4. `RUSTFLAGS="-D warnings" cargo check --all-targets` → no warnings, no errors. Exit 0.
5. `RUSTFLAGS="-D warnings" cargo build --features gui --bin vibe-attack-config` → builds cleanly. Exit 0.

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

["Catch-all match arm removed rather than kept — unreachable_patterns would fail -D warnings build gate", "TestMacro integration tests require multi_thread Tokio flavor due to block_in_place — not documented in original plan"]

## Known Limitations

None.

## Follow-ups

None.

## Files Created/Modified

- `src/pipeline/dispatcher.rs` — 
- `src/control/mod.rs` — 
- `tests/control_integration.rs` — 
- `src/ui/pack_editor.rs` — 
- `src/bin/vibe-attack-config.rs` — 
