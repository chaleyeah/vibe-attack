---
estimated_steps: 22
estimated_files: 5
skills_used: []
---

# T03: Wire RuntimeCommand channel from control handler into coordinator and apply live mode/threshold changes

Close the slice: define a `RuntimeCommand` enum, plumb a `std::sync::mpsc::Sender<RuntimeCommand>` from `DaemonHandle` to the control handler arms, give `spawn_pipeline` the `Receiver<RuntimeCommand>`, drain it on the per-frame coordinator loop, and apply SetMode/SetThreshold/ReloadConfig live.

Concrete steps:

1. In `src/pipeline/coordinator.rs`, add `pub enum RuntimeCommand { SetMode(ActivationMode), SetThreshold(f32), SetInputDevice(Option<String>), SetPttBinding(String), ReloadConfig }` (re-export `ActivationMode` from `crate::control::protocol`). Add a new param `runtime_rx: std::sync::mpsc::Receiver<RuntimeCommand>` to `spawn_pipeline`.
2. Inside the pipeline thread closure, declare `let mut active_mode = ActivationMode::Ptt;` (default; later overridable) BEFORE the `while !pipeline_shutdown.is_cancelled()` loop. At the TOP of that loop (before STT-results drain), add `while let Ok(cmd) = runtime_rx.try_recv() { match cmd { ... } }`. Match arms: `SetMode(m)` → set `active_mode = m`; on PTT→Wake transition clear `ptt_audio` and reset `prev_ptt = false` and rebuild `seg = VadSegmenter::new(seg_cfg.clone())`; on Wake→PTT transition clear `listening_until = None; listening_started_at = None; wake_preroll_len = 0;` and rebuild seg. `SetThreshold(t)` → call `dispatcher.update_threshold(t)` (the dispatcher Arc is in scope). `ReloadConfig` → re-read config via `crate::config::load(None)` (best-effort; on Err just `tracing::warn!` and continue) and apply only `wake.enabled`/`stt.confidence_threshold` portions: send corresponding internal RuntimeCommand applies inline. `SetInputDevice` and `SetPttBinding` → `tracing::warn!("command requires daemon restart in S01")` and ignore (live application deferred to S02/S03).
3. **Important — also gate wake/PTT on `active_mode`**: when `active_mode == ActivationMode::Ptt`, skip the wake-word branch entirely (currently runs whenever `wake` is `Some`); when `active_mode == ActivationMode::Wake`, ignore PTT rising/falling edges (don't enter the `ptt` capture branch). This is what makes mode-flip actually surgical without restart.
4. In `src/control/mod.rs`: add `pub runtime_cmd_tx: std::sync::mpsc::Sender<RuntimeCommand>` to `DaemonHandle` (re-export RuntimeCommand from crate root or `pub use crate::pipeline::coordinator::RuntimeCommand;`). Update `DaemonHandle::new` signature to accept the sender, OR add a builder method `with_runtime_tx`. Replace the `_ =>` catch-all is preserved (TestMacro stays unimplemented). Add explicit handler arms for `SetMode { mode }` (forward `RuntimeCommand::SetMode(mode)` and return Ok), `SetThreshold { threshold }` (forward, return Ok), `SetInputDevice { device }` (forward, return Ok with tracing::warn note), `SetPttBinding { key }` (forward, return Ok), `ReloadConfig` (forward, return Ok). On send error (`SendError`), return `ControlResponse::Error { message: "pipeline not running".into() }`.
5. In `src/main.rs`: create the channel `let (runtime_tx, runtime_rx) = std::sync::mpsc::channel::<RuntimeCommand>();` BEFORE calling `spawn_pipeline`; pass `runtime_rx` into `spawn_pipeline`; pass `runtime_tx` into `DaemonHandle::new` (or builder).
6. Add a focused integration-style test as `tests/runtime_commands.rs`: spawn a thread that simulates the coordinator drain loop (reuse a small reproducer that constructs a Dispatcher + RuntimeCommand channel, sends `SetThreshold(0.5)`, applies it via `dispatcher.update_threshold`, and asserts subsequent `process()` outcome differs). This proves the contract end-to-end without a full audio pipeline. Alternatively put the same coverage in `src/pipeline/coordinator.rs` as a `#[cfg(test)]` test if a unit-level approach is simpler — pick whichever isolates the channel-drain behavior cleanly.

Do not modify the existing `_ => ControlResponse::Error { message: "Not yet implemented".into() }` arm — keep it as the TestMacro forward-compat guard.

## Failure Modes

| Dependency | On error | On timeout | On malformed response |
|------------|----------|-----------|----------------------|
| std::sync::mpsc::Sender (control → coordinator) | return ControlResponse::Error { message: "pipeline not running" } | N/A (try_recv non-blocking) | N/A |
| crate::config::load on ReloadConfig | tracing::warn! + ignore (do not crash coordinator) | N/A | N/A |

## Load Profile

- **Shared resources**: one additional `mpsc::Receiver` polled per coordinator frame (every audio frame, ~50/s). `try_recv()` is non-blocking and cheap.
- **Per-operation cost**: one extra `try_recv()` per frame; near-zero when channel is empty.
- **10x breakpoint**: control commands are user-initiated (rare). No load concern.

## Negative Tests

- **Channel closed**: control handler with a dropped receiver returns `ControlResponse::Error{message}` instead of panicking.
- **Mode flip mid-PTT**: simulated SetMode(Wake) while `prev_ptt == true` — verify `ptt_audio` is cleared (or document that the in-flight PTT utterance is discarded). The test should construct the relevant local state and call the drain logic.
- **Threshold clamp**: SetThreshold(1.5) ends up clamped (covered by T02's update_threshold contract — re-asserted indirectly here).

## Inputs

- ``src/control/protocol.rs` — ActivationMode + new ControlRequest variants from T01`
- ``src/pipeline/dispatcher.rs` — Dispatcher::update_threshold from T02`
- ``src/pipeline/coordinator.rs` — existing spawn_pipeline signature and per-frame loop`
- ``src/control/mod.rs` — existing DaemonHandle struct and handler match arms`
- ``src/main.rs` — existing spawn_pipeline call site (line ~249) and DaemonHandle construction (line ~266)`

## Expected Output

- ``src/pipeline/coordinator.rs` — new RuntimeCommand enum; spawn_pipeline gains runtime_rx parameter; per-frame try_recv drain loop; active_mode local state gating wake vs PTT branches`
- ``src/control/mod.rs` — DaemonHandle gains runtime_cmd_tx field; explicit handler arms for SetMode/SetThreshold/SetInputDevice/SetPttBinding/ReloadConfig forwarding via the channel`
- ``src/control/protocol.rs` — minor: ensure ActivationMode is `pub use`-exposable from coordinator module (no API change otherwise)`
- ``src/main.rs` — RuntimeCommand channel created before spawn_pipeline; runtime_rx passed into spawn_pipeline; runtime_tx threaded into DaemonHandle`
- ``tests/runtime_commands.rs` — integration test proving SetThreshold applied via the channel changes dispatcher match outcome`

## Verification

cargo test --lib --tests 2>&1 | tail -5 — all tests pass including the new runtime_commands test (or coordinator unit test); `cargo clippy --all-targets -- -D warnings` clean; `cargo build --release` succeeds; manual smoke (optional): `cargo run --release` + `echo '{"cmd":"set_threshold","args":{"threshold":0.6}}' | nc -U $XDG_RUNTIME_DIR/vibe-attack/vibe-attack.sock` returns `{"status":"ok"}`.

## Observability Impact

Signals added: `tracing::info!` on each RuntimeCommand applied (cmd kind, old/new value where applicable); `tracing::warn!` for SetInputDevice/SetPttBinding noting restart-required; `tracing::warn!` on ReloadConfig load failure. How a future agent inspects this: run with `RUST_LOG=info` (logs go to stderr), send a control command, inspect stderr for the matching tracing line. Failure state exposed: ControlResponse::Error{message} returned synchronously to the CLI sender when the channel is dropped (pipeline gone) — callers can detect this without polling.
