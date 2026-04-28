# S04 — End-to-end UAT + headless integration test — Research

**Date:** 2026-04-27
**Slice:** S04 depends on S03 (complete)

## Summary

S04 is the final slice of M008. It has two deliverables:

1. **`tests/control_integration.rs`** — a `cargo test --test control_integration` headless integration test that spins up a real `spawn_control_listener` tokio task on a temp socket, sends `ControlRequest::SetMode` and `ControlRequest::SetThreshold` over the socket, and asserts the correct `ControlResponse` and side-effects.
2. **`S04-UAT.md`** — a manual UAT script documenting the steps a human must follow to verify: launch `vibe-attack`, open `vibe-attack-config`, switch mode PTT → Wake in the config window, speak a stratagem phrase, confirm it fires without daemon restart.

This is straightforward work. All subsystems it exercises already exist and have been unit-tested in S01–S03. The integration test is the only piece of new code; the UAT doc is a prose artifact with no build dependency.

**Recommendation:** Write the integration test first (one test file, two test functions covering SetMode round-trip and SetThreshold side-effect), then write the UAT script. No new library dependencies needed.

## Implementation Landscape

### Key Files

- `tests/control_integration.rs` — **does not exist yet; must be created**. The new integration test lives here. Follows the pattern established in `tests/runtime_commands.rs` and `tests/control_protocol.rs`.
- `src/control/mod.rs:104` — `pub async fn spawn_control_listener(handle: DaemonHandle)` — this is the tokio task the test must spin up. It binds to a UDS socket path from `xdg::BaseDirectories`.
- `src/control/client.rs:8` — `pub fn send_command(req: ControlRequest) -> Result<ControlResponse>` — blocking UDS client already written; the test uses this directly.
- `src/control/client.rs:37` — `pub fn is_daemon_running() -> bool` — socket-existence check; test can use to wait for ready.
- `src/control/protocol.rs` — `ControlRequest`, `ControlResponse`, `ActivationMode`, `DaemonStatus` — all serde-complete from S01.
- `src/pipeline/coordinator.rs:34` — `RuntimeCommand` enum — used to construct the MPSC channel the test attaches to `DaemonHandle`.
- `src/pipeline/dispatcher.rs:118` — `Dispatcher::update_threshold` and `Dispatcher::threshold()` — the test asserts threshold side-effects.
- `.gsd/milestones/M008/slices/S04/S04-UAT.md` — **does not exist yet; must be created** as a freestanding markdown file (not built by cargo).

### The Core Test Challenge: Socket Path Isolation

`spawn_control_listener` resolves the socket path via `xdg::BaseDirectories::with_prefix("vibe-attack")` → `place_runtime_file("vibe-attack.sock")`. The server-side path-resolver is a private function (`fn get_socket_path()` in `src/control/mod.rs`). The client-side resolver is also private (`fn get_socket_path()` in `src/control/client.rs`).

**This means the integration test cannot pass a temp socket path.** Both halves are hard-wired to the XDG runtime directory. The test must:
- Accept the real XDG path (works in any user session with `$XDG_RUNTIME_DIR` set)
- Clean up the socket file after the test
- Use `serial_test` crate (already in dev-deps) to serialize tests that touch the shared socket

The XDG runtime dir is always set in a real user session. In headless CI without a real user session, `$XDG_RUNTIME_DIR` may be absent and the test should skip gracefully (using `return` after checking `is_daemon_running()` or catching the bind error).

**Alternative approach (recommended):** Write the integration test to call `spawn_control_listener` on the real XDG socket, wait for the socket file to appear, then call `send_command` from the same process. This is exactly how the daemon and tray communicate in production.

### Build Order

1. Write `tests/control_integration.rs` with two tests:
   - `set_mode_round_trip_via_socket` — sends `SetMode { mode: Wake }`, asserts `ControlResponse::Ok`, reads back `Status` and asserts `active_mode == Wake`
   - `set_threshold_via_socket_updates_dispatcher` — sends `SetThreshold { threshold: 0.6 }`, asserts `Ok`, then calls `send_command(Status)` and asserts the response; **or** holds a `DaemonHandle` reference and checks `dispatcher.threshold()` directly (simpler and doesn't require another socket round-trip)
2. Write `S04-UAT.md` with the manual test steps.
3. Run `cargo test --test control_integration` to confirm both pass.
4. Run full `cargo test` to confirm no regressions.

### Verification Approach

```
cargo test --test control_integration
cargo test
cargo build
cargo build --features gui
```

S04-UAT.md manual steps should produce a log line like:
```
INFO runtime_command_applied cmd=set_mode mode=Wake
```
in the daemon's `RUST_LOG=info` output after the mode switch, with no daemon restart.

## Constraints

- **Socket path is not injectable** — both server and client resolve via `xdg::BaseDirectories`; the test must use the real path and clean up after itself.
- **`spawn_control_listener` is async** — the integration test needs a tokio runtime. tokio 1.x with `features = ["full"]` is already a normal (non-dev) dependency, so `#[tokio::test]` works without adding anything to `[dev-dependencies]`.
- **`serial_test` is already in dev-deps** — use `#[serial]` from `serial_test = "3"` to prevent parallel test runs from racing on the shared socket file.
- **The `spawn_control_listener` call consumes the handle** — make the handle before calling; keep `dispatcher` arc alive separately for side-effect assertions.
- **No new library dependencies needed** — all APIs are already public.

## Common Pitfalls

- **Race between socket bind and first `send_command`** — the control listener binds synchronously before spawning the tokio task, but the test must still wait for the socket file to appear. A short `std::thread::sleep` or poll loop on `is_daemon_running()` is sufficient.
- **Socket cleanup on test failure** — if the test panics before cleanup, the next run fails to bind. Wrap in a `defer`-style struct or use `std::fs::remove_file` in a `panic = unwind` guard. The server already removes a stale socket on startup (`std::fs::remove_file` at `spawn_control_listener:108`), so the next call to `spawn_control_listener` self-heals.
- **XDG_RUNTIME_DIR absent in bare CI** — the test must catch the bind error and call `return` (not `panic!`) to avoid marking the test as failed when the environment doesn't support it.
- **Tokio runtime nesting** — `spawn_control_listener` must be called from within a tokio runtime. `#[tokio::test]` provides one; do not manually create a `Runtime` inside the test.

## Patterns to Follow

- `tests/control_protocol.rs:154` — `fn make_handle()` helper that builds a `DaemonHandle` with a live `Dispatcher` and optional `RuntimeCommand` channel. Copy this; extend with `with_runtime_tx`.
- `tests/runtime_commands.rs:28` — `drain_into_dispatcher` simulates coordinator. The integration test doesn't need this — the socket server handles commands internally.
- All prior test files use `use vibe_attack::...` imports (crate re-exported from `src/lib.rs`).
