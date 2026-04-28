# S05: TriggerMacro control request + editor Test button

**Goal:** Wire a TestMacro control-plane request through the existing dispatcher and add a Test button (with 1-second confirmation) to the egui pack editor so the user can verify a macro's key sequence end-to-end without speaking the trigger phrase.
**Demo:** User opens editor, selects a macro, clicks Test; 1-second confirmation prompt; daemon fires the key sequence via uinput; dispatcher JSONL output shows the triggered macro

## Must-Haves

- User opens the editor, selects a macro, clicks Test, sees a 1-second countdown, then the daemon fires the macro's key sequence via uinput. cargo test (default features) and cargo build/clippy --features gui both pass with -D warnings.

## Proof Level

- This slice proves: integration — the slice proves the control-plane handler routes through the live dispatcher and emits a real MacroCmd::Execute. Real runtime required: yes for end-to-end visual demo, no for automated verification (channel observation suffices). Human/UAT required: no — automated tests cover the wire path; the 1-second UI countdown is verified visually during S06 UAT.

## Integration Closure

Upstream surfaces consumed: src/control/protocol.rs::ControlRequest::TestMacro (already declared in M008), src/control/mod.rs catch-all arm (currently returns "Not yet implemented"), src/pipeline/dispatcher.rs Dispatcher (MacroCmd sender, RwLock<Vec<MacroConfig>> registry), src/ui/pack_editor.rs::PackEditorState (selected_macro field), src/control/client.rs::send_command. New wiring: TestMacro handler routed to Dispatcher::fire_named via tokio::task::block_in_place; editor adds pending_test field and Test button rendered next to Update Macro / Remove Macro. What remains for milestone usability: S06 manual UAT confirming the full editor → daemon → uinput loop on a real desktop session.

## Verification

- Runtime signals: TestMacro emits a tracing::info!("Firing macro name=...") log via the existing Dispatcher::fire_named path so journalctl shows the trigger source. Inspection surfaces: the daemon's stdout JSONL stream emits {"event":"macro_fired","macro_id":"..."} for any test trigger (same shape as phrase-matched triggers, verified during S06 UAT). Failure visibility: when the named macro is not in the active profile, fire_named returns Err with a message naming the missing macro; the control handler propagates that as ControlResponse::Error so the editor surfaces it inline. Redaction constraints: none — macro names are not sensitive.

## Tasks

- [x] **T01: Add Dispatcher::fire_named with unit tests** `est:1h`
  Add `pub fn fire_named(&self, name: &str) -> Result<DispatchOutcome, String>` to `Dispatcher` in `src/pipeline/dispatcher.rs`. The method skips phrase matching: it acquires a read lock on `self.macros`, finds the first MacroConfig whose `name` equals the given name, plays the optional sound (same code path as `process()`), builds a `Vec<KeyStep>` via `KeyStep::from_config`, and sends `MacroCmd::Execute { keys, default_dwell_ms, default_gap_ms }` over `self.macro_tx`. On found-and-sent return `Ok(DispatchOutcome::Fired { macro_id: name.into(), score: 1.0 })` (score 1.0 marks a direct trigger as a deliberate convention). On not-found return `Err(format!("macro not found: {name}"))`. On `macro_tx.send` failure (receiver dropped) return `Err(format!("injection channel closed: {e}"))`. Add tracing::info!(macro_name=name, "Firing macro (direct)") before the send so journalctl distinguishes direct triggers from phrase matches.

Add two unit tests to the existing `mod tests` block, mirroring the `make_dispatcher` helper pattern:
- `fire_named_found_emits_execute`: build dispatcher with a known macro carrying two KeyAction entries; capture the receiver end of the macro channel; call `fire_named("eagle_airstrike")`; assert Ok(Fired { macro_id, score }) where macro_id == "eagle_airstrike" and (score - 1.0).abs() < 1e-6; assert exactly one `MacroCmd::Execute` was sent and its keys vec length matches the configured KeyAction count.
- `fire_named_missing_returns_err`: build dispatcher with one macro; call `fire_named("does_not_exist")`; assert Err whose Display contains "macro not found"; assert receiver got zero MacroCmd messages (use try_recv() and expect Err(TryRecvError::Empty)).

Do NOT touch `process()` or any other dispatcher behavior. Do NOT add a `category` argument — name lookup over the flat registry is sufficient.
  - Files: `src/pipeline/dispatcher.rs`
  - Verify: cargo test --lib pipeline::dispatcher -- --test-threads=1 && RUSTFLAGS="-D warnings" cargo check --all-targets

- [ ] **T02: Wire TestMacro control handler and add round-trip integration test** `est:1h30m`
  Replace the catch-all `_ => ControlResponse::Error { message: "Not yet implemented" }` arm in `src/control/mod.rs` with an explicit `ControlRequest::TestMacro { name }` arm. Mirror the `SwitchProfile` pattern exactly: use `tokio::task::block_in_place(|| h.dispatcher.fire_named(&name))` so the synchronous `RwLock` read does not block the Tokio executor across an await point. On `Ok(_)` return `ControlResponse::Ok`; on `Err(msg)` return `ControlResponse::Error { message: msg }`. Add `tracing::info!(macro_name = %name, "TestMacro request received")` immediately before the block_in_place call. Keep the catch-all arm (now redundant since TestMacro was the only un-handled variant) — replace it with `#[cfg(test)]` exhaustive coverage by adding a panicking arm only if a future variant slips through; otherwise leave as a defensive `_ =>` returning a clearly-named error.

Add a new integration test `test_macro_via_socket_fires_dispatcher` to `tests/control_integration.rs` modeled on `set_threshold_via_socket_updates_dispatcher`:
- Use `make_handle_with_runtime_tx`-like setup but build the Dispatcher with one MacroConfig (name="smoke_test", keys with one KEY_UP entry); capture the receiver end of the MacroCmd channel by adding a sibling helper `make_handle_with_macro_rx() -> (DaemonHandle, mpsc::Receiver<MacroCmd>)` (or extend the existing helper).
- Skip if `socket_path()` returns None or `spawn_control_listener` fails to bind (same skip pattern as existing tests).
- Send `ControlRequest::TestMacro { name: "smoke_test" }` via `spawn_blocking(|| send_command(...))`; assert `ControlResponse::Ok`.
- Drain the macro receiver with `try_recv` and assert exactly one `MacroCmd::Execute` was sent.
- Add a second negative test `test_macro_unknown_name_returns_error`: send `TestMacro { name: "nonexistent" }` and assert `ControlResponse::Error` whose message contains "macro not found".
- Use `#[serial]` on both tests to match the existing concurrency convention.

Do NOT add a `category` field to ControlRequest::TestMacro; the variant is already declared correctly in protocol.rs and macro names are unique within a profile.
  - Files: `src/control/mod.rs`, `tests/control_integration.rs`
  - Verify: cargo test --test control_integration -- --test-threads=1 && cargo test -- --test-threads=1 && RUSTFLAGS="-D warnings" cargo clippy --all-targets

- [ ] **T03: Add Test button with 1-second confirmation to pack editor** `est:2h`
  Add a Test button to the pack editor (gui-feature only) that fires the currently-selected macro via the daemon after a 1-second deliberate delay. The delay is a hard safety requirement: an accidental click during a live game session would waste a stratagem.

In `src/ui/pack_editor.rs::PackEditorState` (inside the `#[cfg(feature = "gui")] mod inner`):
- Add field `pub pending_test: Option<(String, std::time::Instant)>` initialized to None in `PackEditorState::new`.
- Add field `pub last_test_status: Option<String>` initialized to None (used to surface the daemon's response in the editor panel).

In `show_pack_editor` (the same file), in the right-hand edit form's button row alongside Add Macro / Update Macro:
- When `state.selected_macro.is_some()` and `state.pending_test.is_none()`, render a `Test` button. On click, set `state.pending_test = Some((selected_macro.clone(), Instant::now()))` and `state.last_test_status = None`.
- When `state.pending_test.is_some()`, render a non-clickable label `Firing in N.Ns... (Cancel)` where N.Ns counts down from 1.0 to 0.0 (compute with `1.0 - elapsed_secs`, clamp to >= 0.0). Render a Cancel button next to the countdown that clears `pending_test`.
- Each frame, if `pending_test.is_some()` and `elapsed >= Duration::from_secs(1)`, take the pending name out of the Option, then call `send_command(ControlRequest::TestMacro { name })`. On Ok(ControlResponse::Ok) set `last_test_status = Some(format!("Fired: {name}"))`. On Ok(ControlResponse::Error { message }) set `last_test_status = Some(format!("Test failed: {message}"))`. On Err(e) set `last_test_status = Some(format!("Daemon error: {e}"))`. Always log via `tracing::info!`/`tracing::warn!` with the macro name and outcome.
- Render `last_test_status` as a colored label (green for success, red for failure) below the inline `last_error` display.
- The Test button must be greyed out (use `ui.add_enabled(daemon_running, ...)`) when the daemon is not running. Pass `daemon_running: bool` into `show_pack_editor` as a new parameter; update the single call site in `src/bin/vibe-attack-config.rs` to pass `app.config.daemon_running`.
- Each frame the editor panel is visible AND `pending_test.is_some()`, request a fast repaint (`ui.ctx().request_repaint_after(Duration::from_millis(50))`) so the countdown animates smoothly.

No egui or eframe types in the default-feature build. The new fields are inside the `mod inner` (gui-gated) block. No `std::thread::sleep`, no `tokio::time::sleep` — use `Instant` polled from the eframe loop.

Document the safety rationale in a one-line comment above `pending_test`: `// 1-second confirmation guards against accidental fire — never use thread::sleep.`
  - Files: `src/ui/pack_editor.rs`, `src/bin/vibe-attack-config.rs`
  - Verify: RUSTFLAGS="-D warnings" cargo build --features gui --bin vibe-attack-config && RUSTFLAGS="-D warnings" cargo clippy --features gui --all-targets && RUSTFLAGS="-D warnings" cargo clippy --all-targets && cargo test -- --test-threads=1

## Files Likely Touched

- src/pipeline/dispatcher.rs
- src/control/mod.rs
- tests/control_integration.rs
- src/ui/pack_editor.rs
- src/bin/vibe-attack-config.rs
