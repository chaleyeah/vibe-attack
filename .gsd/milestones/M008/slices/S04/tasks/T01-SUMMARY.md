---
id: T01
parent: S04
milestone: M008
key_files:
  - tests/control_integration.rs
key_decisions:
  - SocketGuard uses place_runtime_file (server-side path) not find_runtime_file so path always matches the bound socket
  - Tests skip gracefully (return, not panic) when XDG_RUNTIME_DIR is absent, keeping CI green on bare environments
  - SetThreshold test asserts only the RuntimeCommand channel — dispatcher.threshold() is updated by coordinator drain, not the handler, so asserting it here would be wrong
duration: 
verification_result: passed
completed_at: 2026-04-28T02:02:47.934Z
blocker_discovered: false
---

# T01: Added tests/control_integration.rs with two serial tokio tests proving SetMode and SetThreshold round-trips over the real UDS socket

**Added tests/control_integration.rs with two serial tokio tests proving SetMode and SetThreshold round-trips over the real UDS socket**

## What Happened

Created `tests/control_integration.rs` as a new integration test file exercising the M008 control surface end-to-end over a live Unix-domain socket.

**Implementation decisions:**

1. **Socket path via `place_runtime_file`**: The cleanup `SocketGuard` resolves the path using the server-side `xdg::BaseDirectories::with_prefix("vibe-attack").place_runtime_file("vibe-attack.sock")` — the same function the server uses — so the guard always matches what was bound. `find_runtime_file` (the client path) would fail before bind succeeds.

2. **CI skip pattern**: Both tests call `socket_path()` first; if `XDG_RUNTIME_DIR` is absent (returns `None` from `place_runtime_file`), they print a skip message and `return` rather than panic. If the bind itself fails (e.g. the runtime dir exists but isn't writable), the `spawn_control_listener` error is caught and the test skips similarly.

3. **`spawn_blocking` for `send_command`**: The blocking std `UnixStream` client must not run on the tokio reactor, so all `send_command` calls are wrapped in `tokio::task::spawn_blocking`.

4. **`make_handle_with_runtime_tx` helper**: Mirrors the `make_handle` pattern from `tests/control_protocol.rs:154` but additionally calls `.with_runtime_tx(rt_tx)` so `SetMode` and `SetThreshold` handlers can forward `RuntimeCommand`s without returning `Error{pipeline not running}`.

5. **Threshold channel assert only**: For `set_threshold_via_socket_updates_dispatcher`, the test only asserts that `RuntimeCommand::SetThreshold(0.6)` appeared on the channel — it does not assert `dispatcher.threshold() == 0.6` because the dispatcher is updated by the coordinator drain loop, not by the handler directly. This matches the plan's note and avoids a spurious race.

6. **SetMode cache assertion**: For `set_mode_round_trip_via_socket`, the test asserts `*handle.active_mode.read().unwrap() == ActivationMode::Wake` after the socket round-trip, confirming the handler writes the cache before forwarding. A follow-up `Status` query over the wire confirms the cache is reflected back.

Both tests ran locally against a real `XDG_RUNTIME_DIR` and passed in 0.08s. Full suite remained green with no regressions.

## Verification

Ran `cargo test --test control_integration -- --test-threads=1` — 2 passed, 0 failed, finished in 0.08s. Confirmed `test result: ok` in /tmp/s04-t01-test.log. Ran full `cargo test` — all test binaries passed, 0 failures, no regressions.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test control_integration -- --test-threads=1 2>&1 | tee /tmp/s04-t01-test.log` | 0 | ✅ pass | 1020ms |
| 2 | `grep -q 'test result: ok' /tmp/s04-t01-test.log` | 0 | ✅ pass | 5ms |
| 3 | `cargo test 2>&1 | tail -20` | 0 | ✅ pass | 4800ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `tests/control_integration.rs`
