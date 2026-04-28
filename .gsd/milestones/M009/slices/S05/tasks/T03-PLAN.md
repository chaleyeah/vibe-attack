---
estimated_steps: 13
estimated_files: 2
skills_used: []
---

# T03: Add Test button with 1-second confirmation to pack editor

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

## Inputs

- ``src/ui/pack_editor.rs` — PackEditorState struct, show_pack_editor function, existing Add Macro / Update Macro / Remove Macro button rows in the right-hand edit form`
- ``src/bin/vibe-attack-config.rs` — show_main_config function which calls show_pack_editor and already exposes app.config.daemon_running`
- ``src/control/client.rs` — send_command(ControlRequest) -> Result<ControlResponse> (synchronous, blocking; one round-trip is fast enough to call directly from the eframe loop)`
- ``src/control/protocol.rs` — ControlRequest::TestMacro { name: String } and ControlResponse variants`
- ``src/control/mod.rs` — TestMacro handler wired in T02 (must be present so the editor's send_command actually returns Ok or a typed Error rather than the old "Not yet implemented")`

## Expected Output

- ``src/ui/pack_editor.rs` — pending_test and last_test_status fields on PackEditorState; Test/Cancel button + countdown rendering; show_pack_editor signature gains daemon_running: bool parameter`
- ``src/bin/vibe-attack-config.rs` — call site updated to pass app.config.daemon_running into show_pack_editor`

## Verification

RUSTFLAGS="-D warnings" cargo build --features gui --bin vibe-attack-config && RUSTFLAGS="-D warnings" cargo clippy --features gui --all-targets && RUSTFLAGS="-D warnings" cargo clippy --all-targets && cargo test -- --test-threads=1

## Observability Impact

Adds tracing::info! on Test click (macro_name) and tracing::info!/warn! on Test outcome (success/error message) so journalctl shows the editor's trigger trail. last_test_status surfaces the daemon's response inline in the editor panel for failure visibility. No new files, no metrics, no JSONL change — the daemon still emits the same `Firing macro` log it does for phrase matches (T01).
