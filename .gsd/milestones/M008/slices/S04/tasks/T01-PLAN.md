---
estimated_steps: 31
estimated_files: 6
skills_used: []
---

# T01: Write tests/control_integration.rs covering SetMode and SetThreshold round-trips over the real UDS socket

Create tests/control_integration.rs as a new integration test file that proves the M008 control surface works end-to-end across the real Unix-domain socket. The test spins up spawn_control_listener on a tokio runtime, waits for the socket file to appear at the XDG runtime path, sends ControlRequest::SetMode and ControlRequest::SetThreshold via the blocking send_command client, and asserts both the wire-level response and the in-process side-effects on the shared DaemonHandle/Dispatcher.

Socket path is NOT injectable — both server (place_runtime_file) and client (find_runtime_file) hard-wire to xdg::BaseDirectories::with_prefix("vibe-attack"). The test must use the real XDG path and serialize with #[serial_test::serial] to prevent races against any other test that touches the same socket. If XDG_RUNTIME_DIR is absent (bare CI), the bind will fail — catch the error and `return` (do not panic) so the test is skipped gracefully.

Two test functions:

1. `set_mode_round_trip_via_socket` (#[tokio::test] #[serial]):
   - Build a DaemonHandle with a Dispatcher and a runtime_cmd_tx (mpsc channel) attached via with_runtime_tx — this is required because SetMode forwards a RuntimeCommand and would return Error{message:"pipeline not running"} otherwise. Hold the receiver alive in the test so try_send succeeds.
   - Confirm initial active_mode is Ptt via *handle.active_mode.read().unwrap().
   - Call spawn_control_listener(handle.clone()).await — bail out gracefully if the bind errors (XDG_RUNTIME_DIR absent in CI).
   - Poll vibe_attack::control::client::is_daemon_running() with a short sleep loop (e.g. up to 50 iterations of 20ms) until true; otherwise skip.
   - Call tokio::task::spawn_blocking to invoke send_command(ControlRequest::SetMode { mode: ActivationMode::Wake }) — send_command is blocking std UnixStream and must not run on the tokio reactor. Await the JoinHandle. Assert response is ControlResponse::Ok.
   - Drain the receiver to confirm exactly one RuntimeCommand::SetMode(Wake) was forwarded.
   - Assert *handle.active_mode.read().unwrap() == ActivationMode::Wake (the SetMode handler caches the mode before forwarding).
   - Call spawn_blocking(send_command(ControlRequest::Status)) and assert the StatusData payload's active_mode == Wake — proves the cache is reflected back over the wire.
   - Clean up: std::fs::remove_file(socket_path) in a guard struct (Drop impl) so the file goes away even if assertions panic.

2. `set_threshold_via_socket_updates_dispatcher` (#[tokio::test] #[serial]):
   - Build the same handle with a fresh Dispatcher created with starting threshold 0.5 and a runtime_cmd_tx attached. Hold the receiver in the test scope.
   - Spawn the listener; wait for the socket to be ready (same skip-if-no-XDG_RUNTIME_DIR pattern).
   - send_command(ControlRequest::SetThreshold { threshold: 0.6 }) via spawn_blocking; assert ControlResponse::Ok.
   - Drain the rx channel and assert exactly one RuntimeCommand::SetThreshold(0.6) was forwarded — this proves the socket → handle → coordinator path. Do NOT assert dispatcher.threshold() == 0.6 directly: the dispatcher is updated by the coordinator when it drains the channel, not by the SetThreshold handler. The handler forwards the command and returns Ok; coordinator integration is already covered by tests/runtime_commands.rs.
   - Clean up the socket file via the same Drop guard.

Helper function `make_handle_with_runtime_tx() -> (DaemonHandle, mpsc::Receiver<RuntimeCommand>)` near the top of the file — copy the make_handle pattern from tests/control_protocol.rs:154 but extend with the runtime channel. Use `vibe_attack::pipeline::dispatcher::Dispatcher::new(0.5, vec![], dispatch_tx, 50, 30)` for the dispatcher and a separate `mpsc::channel::<vibe_attack::macros::MacroAction>` (or whatever Dispatcher::new expects — verify from tests/control_protocol.rs:159) for the dispatch channel.

Drop guard struct example:
```rust
struct SocketGuard(std::path::PathBuf);
impl Drop for SocketGuard {
    fn drop(&mut self) { let _ = std::fs::remove_file(&self.0); }
}
```
Resolve the socket path inside the test the same way the server does: `xdg::BaseDirectories::with_prefix("vibe-attack").get_runtime_file("vibe-attack.sock")` — `get_runtime_file` is read-only and won't error pre-bind; use it only to build the cleanup path after the bind succeeds.

Add `use serial_test::serial;` and `use vibe_attack::control::{client::{send_command, is_daemon_running, query_status}, protocol::{ActivationMode, ControlRequest, ControlResponse}, spawn_control_listener, DaemonHandle};` plus the dispatcher and RuntimeCommand imports.

Failure modes to handle: bind error when XDG_RUNTIME_DIR is unset (skip via early return), socket-already-in-use leftover from a prior crashed test (server's spawn_control_listener already removes the stale socket on bind, so this self-heals), parallel test interference (mitigated by #[serial]).

No new dependencies needed — tokio with `full` is a normal dep and serial_test is in dev-deps already.

## Inputs

- ``src/control/mod.rs``
- ``src/control/client.rs``
- ``src/control/protocol.rs``
- ``src/pipeline/dispatcher.rs``
- ``src/pipeline/coordinator.rs``
- ``tests/control_protocol.rs``
- ``tests/runtime_commands.rs``
- ``Cargo.toml``

## Expected Output

- ``tests/control_integration.rs``

## Verification

cargo test --test control_integration -- --test-threads=1 2>&1 | tee /tmp/s04-t01-test.log && grep -q 'test result: ok' /tmp/s04-t01-test.log && cargo test 2>&1 | tail -20 | grep -q 'test result: ok'

## Observability Impact

Test asserts on ControlResponse values returned over the wire AND on the cached active_mode field of the shared DaemonHandle — both are runtime inspection surfaces. If the test fails, the failure message includes the actual response variant and the cached mode value, giving a future agent a direct signal about whether the regression is in the wire codec, the handler ordering, or the cache write. Test uses RUST_LOG-aware tracing via the existing daemon code path; running with RUST_LOG=debug surfaces the existing `SetMode: cached active_mode=...` log for diagnostic context.
