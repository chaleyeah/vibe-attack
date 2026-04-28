# S04: End-to-end UAT + headless integration test

**Goal:** Prove the M008 control surface end-to-end: a headless integration test exercises the real Unix-socket round-trip for SetMode and SetThreshold against a live spawn_control_listener, and a manual UAT script captures the full tray → daemon mode-switch flow without restart.
**Demo:** cargo test --test control_integration passes; S04-UAT.md manual steps produce 'mode changed, stratagem fired by voice' result without daemon restart

## Must-Haves

- cargo test --test control_integration passes both test functions; full cargo test (default + --features gui) is green with no regressions; S04-UAT.md exists at .gsd/milestones/M008/slices/S04/S04-UAT.md and walks a human through launching vibe-attack, switching mode in vibe-attack-config, and firing a stratagem by voice without daemon restart; cargo clippy -D warnings clean for default and gui feature sets.

## Proof Level

- This slice proves: final-assembly — this slice closes M008 by exercising the real spawn_control_listener over a real UDS socket (test) and the real tray + config UI (UAT). Real runtime required: yes (test spins up a tokio runtime and binds the XDG socket; UAT requires the user's session). Human/UAT required: yes (S04-UAT.md is part of the deliverable).

## Integration Closure

Upstream surfaces consumed: src/control/mod.rs::spawn_control_listener (server), src/control/client.rs::send_command + query_status (client), src/control/protocol.rs ControlRequest/ControlResponse/DaemonStatus, src/pipeline/dispatcher.rs::Dispatcher::threshold, src/pipeline/coordinator.rs::RuntimeCommand. New wiring introduced: none — the test only composes existing public APIs. After this slice, M008 is end-to-end usable: tray reflects state, config window edits mode/threshold, daemon honors changes live. Remaining before milestone-truly-usable: nothing inside M008's scope.

## Verification

- Integration test asserts on ControlResponse values returned over the socket and on Dispatcher::threshold() side-effect via the shared Arc — both are inspection surfaces that future agents can reuse to debug the control plane. UAT script instructs the user to run with RUST_LOG=info so the daemon emits the existing `SetMode: cached active_mode=...` debug line and the runtime_command_applied log, giving a paper trail of the mode switch. No new logs introduced — the slice consumes existing observability surfaces from S01–S03.

## Tasks

- [x] **T01: Write tests/control_integration.rs covering SetMode and SetThreshold round-trips over the real UDS socket** `est:1h`
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
  - Files: `tests/control_integration.rs`, `src/control/mod.rs`, `src/control/client.rs`, `src/control/protocol.rs`, `tests/control_protocol.rs`, `tests/runtime_commands.rs`
  - Verify: cargo test --test control_integration -- --test-threads=1 2>&1 | tee /tmp/s04-t01-test.log && grep -q 'test result: ok' /tmp/s04-t01-test.log && cargo test 2>&1 | tail -20 | grep -q 'test result: ok'

- [ ] **T02: Author S04-UAT.md manual test script for tray/config end-to-end mode switch** `est:30m`
  Write .gsd/milestones/M008/slices/S04/S04-UAT.md as a freestanding manual test script. The doc walks a human through verifying that M008's tray + config window + daemon control surface works end-to-end on a real Linux desktop session, with no daemon restart required for mode/threshold changes.

Structure the doc as a runnable checklist with these sections:

1. **Preconditions** — bullet list:
   - Linux desktop session with system tray (KDE Plasma, GNOME with extension, XFCE, etc.) and a working microphone.
   - `cargo build --features gui` and `cargo build` both succeed.
   - Microphone accessible to the user (no PipeWire permission errors in journalctl).
   - HD2 pack profile loaded (or any profile with at least one stratagem phrase).
   - `RUST_LOG=info` or `RUST_LOG=debug` set in the shell that launches the daemon, so SetMode and runtime_command_applied log lines are visible.

2. **Setup** — numbered steps with shell commands:
   - `cargo run --bin vibe-attack 2>&1 | tee /tmp/vibe-attack-uat.log` in terminal A.
   - Wait for `Control channel listening on:` log line — record the socket path.
   - In terminal B: `cargo run --features gui --bin vibe-attack-config` to open the config window.
   - Confirm the tray icon appears in the system tray (audio-input-microphone glyph).

3. **Test 1: PTT → Wake mode switch via config window** — numbered steps + expected:
   - In the config window, change Mode dropdown from 'Push-to-talk' to 'Wake word'.
   - Click Save.
   - **Expected in terminal A:** within 1s, a log line `SetMode: cached active_mode=Wake` and `runtime_command_applied cmd=set_mode mode=Wake` (or equivalent — exact wording from the daemon).
   - **Expected in tray:** Mode submenu now shows 'Wake word' checkmarked.
   - **Expected in daemon:** no restart log lines (no 'Pipeline shutting down' or 'Spawning audio thread' messages after the SetMode).

4. **Test 2: Speak a stratagem phrase** — numbered steps:
   - With Wake mode active, speak the wake word (whatever the project default is — note as TBD if not yet wired) followed by a stratagem phrase from the active profile.
   - **Expected in terminal A:** STT transcription log + macro fired log (`Dispatched macro: <name>`).
   - **Expected behavior:** the configured key sequence is sent (verify with a focused text editor or `xev` window if not in-game).

5. **Test 3: Threshold change without restart** — numbered steps:
   - In the config window, drag threshold slider to a different value (e.g. 0.4 if it was 0.5).
   - Click Save.
   - **Expected in terminal A:** SetThreshold log line; no restart messages.
   - **Expected behavior:** subsequent phrase matches use the new threshold (test by speaking a slightly mispronounced phrase that previously did not fire).

6. **Test 4: Tray Mode submenu round-trip** — numbered steps:
   - Right-click tray icon → Mode → 'Push-to-talk'.
   - **Expected in terminal A:** SetMode log line with mode=Ptt.
   - **Expected in tray:** checkmark moves to 'Push-to-talk'; config window does NOT need to be open.
   - **Expected: no daemon restart.**

7. **Test 5: Tray icon state transitions** — bulleted observation list:
   - At rest: audio-input-microphone (idle).
   - During wake-word listen window: audio-input-microphone-high.
   - During PTT recording: audio-input-microphone.
   - When muted (right-click → Mute): audio-input-microphone-muted.
   - Daemon stopped: audio-input-microphone-muted (TrayState.daemon_running=false).

8. **Pass/fail criteria** — explicit checklist:
   - [ ] Mode switch in config triggers SetMode log without restart.
   - [ ] Tray Mode submenu reflects current mode and dispatches SetMode without restart.
   - [ ] Threshold change in config triggers SetThreshold log without restart.
   - [ ] Stratagem phrase fires after a mode switch with no daemon restart in between.
   - [ ] Tray icon visually changes between idle / listening / recording / muted as the daemon transitions.

9. **Known UAT limitations** — short bullet list:
   - Recording-state icon is only visible during a live PTT hold — not observable in dashboard-style passive checks.
   - Wake-word listen window may be very short; capture by enabling RUST_LOG=debug and grepping the log.
   - Tray icon rendering quality depends on the distro's StatusNotifierItem implementation; KDE/Plasma is the reference target.

Keep the doc terse and verifiable — every assertion should be a log line, a visible UI element, or a key event the tester can confirm with their own eyes. Do not include any code that needs to be compiled — this file is consumed by humans only.

Write the file directly with the Write tool. The .gsd/ directory is gitignored, so no commit is needed.
  - Files: `.gsd/milestones/M008/slices/S04/S04-UAT.md`, `.gsd/milestones/M008/M008-ROADMAP.md`, `.gsd/milestones/M008/slices/S04/S04-RESEARCH.md`, `.gsd/milestones/M008/slices/S03/S03-SUMMARY.md`, `src/ui/tray.rs`, `src/ui/config_app.rs`
  - Verify: test -f .gsd/milestones/M008/slices/S04/S04-UAT.md && [ $(wc -l < .gsd/milestones/M008/slices/S04/S04-UAT.md) -gt 30 ] && grep -q 'Pass/fail' .gsd/milestones/M008/slices/S04/S04-UAT.md && grep -q 'Preconditions' .gsd/milestones/M008/slices/S04/S04-UAT.md && grep -q 'SetMode' .gsd/milestones/M008/slices/S04/S04-UAT.md

## Files Likely Touched

- tests/control_integration.rs
- src/control/mod.rs
- src/control/client.rs
- src/control/protocol.rs
- tests/control_protocol.rs
- tests/runtime_commands.rs
- .gsd/milestones/M008/slices/S04/S04-UAT.md
- .gsd/milestones/M008/M008-ROADMAP.md
- .gsd/milestones/M008/slices/S04/S04-RESEARCH.md
- .gsd/milestones/M008/slices/S03/S03-SUMMARY.md
- src/ui/tray.rs
- src/ui/config_app.rs
