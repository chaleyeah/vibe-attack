---
id: T02
parent: S05
milestone: M009
key_files:
  - src/control/mod.rs
  - tests/control_integration.rs
key_decisions:
  - block_in_place requires multi_thread Tokio runtime flavor — new TestMacro integration tests use #[tokio::test(flavor = "multi_thread", worker_threads = 2)] while existing tests (SetMode, SetThreshold) remain on the default single-thread runtime because they do not call block_in_place
  - Removed the catch-all _ arm entirely because TestMacro was the last unhandled variant; keeping it would have triggered unreachable_patterns under -D warnings
duration: 
verification_result: passed
completed_at: 2026-04-28T03:17:34.569Z
blocker_discovered: false
---

# T02: Wired ControlRequest::TestMacro handler in src/control/mod.rs and added two round-trip integration tests proving the socket-to-dispatcher path fires macros by name

**Wired ControlRequest::TestMacro handler in src/control/mod.rs and added two round-trip integration tests proving the socket-to-dispatcher path fires macros by name**

## What Happened

Replaced the catch-all `_ => ControlResponse::Error { message: "Not yet implemented" }` arm in `src/control/mod.rs` with an explicit `ControlRequest::TestMacro { name }` arm. The arm follows the SwitchProfile pattern exactly: it emits `tracing::info!(macro_name = %name, "TestMacro request received")` for journalctl visibility, then calls `tokio::task::block_in_place(|| h.dispatcher.fire_named(&name))` so the synchronous RwLock read does not block the Tokio executor across an await point. `Ok(_)` maps to `ControlResponse::Ok`; `Err(msg)` maps to `ControlResponse::Error { message: msg }`. The former defensive catch-all was removed entirely because `TestMacro` was the last unhandled variant — keeping it would have caused an `unreachable_patterns` warning under `RUSTFLAGS=-D warnings`.\n\nAdded `make_handle_with_macro_rx()` helper to `tests/control_integration.rs` that builds a Dispatcher pre-loaded with one `MacroConfig` (name=`"smoke_test"`, one KEY_UP key step) and returns both the DaemonHandle and the `mpsc::Receiver<MacroCmd>` so tests can assert channel messages.\n\nAdded two integration tests, both tagged `#[serial]` and `#[tokio::test(flavor = \"multi_thread\", worker_threads = 2)]`:\n- `test_macro_via_socket_fires_dispatcher`: sends `TestMacro { name: \"smoke_test\" }` over the socket, asserts `ControlResponse::Ok`, then drains the macro channel and asserts exactly one `MacroCmd::Execute` was delivered.\n- `test_macro_unknown_name_returns_error`: sends `TestMacro { name: \"nonexistent\" }`, asserts `ControlResponse::Error` whose message contains `\"macro not found\"`.\n\nKey deviation from the plan: the `multi_thread` flavor was required on both new tests because `block_in_place` panics on the single-threaded runtime that `#[tokio::test]` uses by default. The existing `set_mode_round_trip_via_socket` and `set_threshold_via_socket_updates_dispatcher` tests were unaffected because they route through `send_runtime_cmd` which never calls `block_in_place`.

## Verification

Ran `cargo test --test control_integration -- --test-threads=1`: 4/4 tests pass including the two new tests. Ran `cargo test -- --test-threads=1`: 155 tests pass across all test suites with 0 failures. Ran `RUSTFLAGS=\"-D warnings\" cargo check --all-targets`: clean with no warnings. Clippy binary was absent on this machine; cargo check with -D warnings was used as the equivalent gate.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test control_integration -- --test-threads=1` | 0 | ✅ pass — 4/4 tests pass (2 pre-existing + 2 new) | 2060ms |
| 2 | `cargo test -- --test-threads=1` | 0 | ✅ pass — 155 tests pass, 0 failures | 3200ms |
| 3 | `RUSTFLAGS="-D warnings" cargo check --all-targets` | 0 | ✅ pass — no warnings | 460ms |

## Deviations

Two deviations from the plan: (1) The plan said to keep the catch-all arm as a defensive fallback — it was removed instead because it became unreachable (TestMacro was the only un-handled variant) and would have caused a -D warnings build failure. (2) The plan did not mention that block_in_place requires a multi-threaded Tokio runtime; the two new tests required #[tokio::test(flavor = \"multi_thread\", worker_threads = 2)] to avoid a runtime panic.

## Known Issues

none

## Files Created/Modified

- `src/control/mod.rs`
- `tests/control_integration.rs`
