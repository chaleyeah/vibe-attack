---
estimated_steps: 9
estimated_files: 2
skills_used: []
---

# T02: Wire TestMacro control handler and add round-trip integration test

Replace the catch-all `_ => ControlResponse::Error { message: "Not yet implemented" }` arm in `src/control/mod.rs` with an explicit `ControlRequest::TestMacro { name }` arm. Mirror the `SwitchProfile` pattern exactly: use `tokio::task::block_in_place(|| h.dispatcher.fire_named(&name))` so the synchronous `RwLock` read does not block the Tokio executor across an await point. On `Ok(_)` return `ControlResponse::Ok`; on `Err(msg)` return `ControlResponse::Error { message: msg }`. Add `tracing::info!(macro_name = %name, "TestMacro request received")` immediately before the block_in_place call. Keep the catch-all arm (now redundant since TestMacro was the only un-handled variant) — replace it with `#[cfg(test)]` exhaustive coverage by adding a panicking arm only if a future variant slips through; otherwise leave as a defensive `_ =>` returning a clearly-named error.

Add a new integration test `test_macro_via_socket_fires_dispatcher` to `tests/control_integration.rs` modeled on `set_threshold_via_socket_updates_dispatcher`:
- Use `make_handle_with_runtime_tx`-like setup but build the Dispatcher with one MacroConfig (name="smoke_test", keys with one KEY_UP entry); capture the receiver end of the MacroCmd channel by adding a sibling helper `make_handle_with_macro_rx() -> (DaemonHandle, mpsc::Receiver<MacroCmd>)` (or extend the existing helper).
- Skip if `socket_path()` returns None or `spawn_control_listener` fails to bind (same skip pattern as existing tests).
- Send `ControlRequest::TestMacro { name: "smoke_test" }` via `spawn_blocking(|| send_command(...))`; assert `ControlResponse::Ok`.
- Drain the macro receiver with `try_recv` and assert exactly one `MacroCmd::Execute` was sent.
- Add a second negative test `test_macro_unknown_name_returns_error`: send `TestMacro { name: "nonexistent" }` and assert `ControlResponse::Error` whose message contains "macro not found".
- Use `#[serial]` on both tests to match the existing concurrency convention.

Do NOT add a `category` field to ControlRequest::TestMacro; the variant is already declared correctly in protocol.rs and macro names are unique within a profile.

## Inputs

- ``src/pipeline/dispatcher.rs` — Dispatcher::fire_named added in T01 (callable from the control handler)`
- ``src/control/mod.rs` — existing match arm structure for ControlRequest variants; SwitchProfile arm at lines 156-163 is the template (block_in_place pattern)`
- ``src/control/protocol.rs` — ControlRequest::TestMacro { name: String } variant already declared (no protocol change needed)`
- ``tests/control_integration.rs` — existing `make_handle_with_runtime_tx`, `socket_path`, `SocketGuard`, and `set_threshold_via_socket_updates_dispatcher` test as templates`

## Expected Output

- ``src/control/mod.rs` — explicit ControlRequest::TestMacro arm replacing the catch-all-only handling for that variant`
- ``tests/control_integration.rs` — two new tests `test_macro_via_socket_fires_dispatcher` and `test_macro_unknown_name_returns_error`, plus a helper that exposes the MacroCmd receiver`

## Verification

cargo test --test control_integration -- --test-threads=1 && cargo test -- --test-threads=1 && RUSTFLAGS="-D warnings" cargo clippy --all-targets
