---
id: T03
parent: S01
milestone: M008
key_files:
  - src/pipeline/coordinator.rs
  - src/pipeline/dispatcher.rs
  - src/control/mod.rs
  - src/main.rs
  - tests/runtime_commands.rs
key_decisions:
  - dispatcher_for_pipeline is a separate Arc::clone from dispatcher_for_thread — two distinct clones are required because the dispatcher thread closure moves its clone before the pipeline thread closure can be spawned
  - active_mode state is local to the pipeline thread (not an Arc<AtomicXxx>) because ActivationMode is not atomically encodable as a single integer and the only writer is the drain loop on the same thread
  - DaemonHandle.runtime_cmd_tx is Option<Arc<Sender>> so DaemonHandle::new() remains zero-argument (no required runtime channel) — the field defaults to None and is set via with_runtime_tx() builder, preserving compatibility with existing tests
  - SetInputDevice and SetPttBinding are forwarded through the channel as RuntimeCommand variants (so the coordinator can log them) rather than handled entirely in the control server — this keeps the full command trace in one place (coordinator stderr) for observability
duration: 
verification_result: passed
completed_at: 2026-04-28T01:26:18.976Z
blocker_discovered: false
---

# T03: feat: wire RuntimeCommand mpsc channel from DaemonHandle into spawn_pipeline coordinator and apply live SetMode/SetThreshold/ReloadConfig changes between utterances

**feat: wire RuntimeCommand mpsc channel from DaemonHandle into spawn_pipeline coordinator and apply live SetMode/SetThreshold/ReloadConfig changes between utterances**

## What Happened

Closed the S01 control-protocol loop by defining a `RuntimeCommand` enum in `src/pipeline/coordinator.rs` (`SetMode`, `SetThreshold`, `SetInputDevice`, `SetPttBinding`, `ReloadConfig`) and adding a `std::sync::mpsc::Receiver<RuntimeCommand>` parameter to `spawn_pipeline`. Inside the pipeline thread, a `try_recv` drain loop runs at the top of each `while !pipeline_shutdown.is_cancelled()` iteration, before any audio processing.

SetMode applies live activation-mode switching with surgical state cleanup: PTT→Wake clears `ptt_audio`/`prev_ptt` and resets the VAD segmenter; Wake→PTT clears the listen-window (`listening_until`, `listening_started_at`, `wake_preroll_len`) and rebuilds the segmenter. Both branches emit `tracing::info!(cmd, mode, "runtime_command_applied")`. Mode gating was also wired at the frame-processing level: the PTT capture branch now checks `active_mode == ActivationMode::Ptt`, and the wake-word detection block requires `active_mode == ActivationMode::Wake`, so a mode flip is surgical with no restart.

SetThreshold calls `dispatcher_for_pipeline.update_threshold(t)` (the pipeline's Arc clone of Dispatcher) and logs old→new values. ReloadConfig calls `crate::config::load(None)` and applies `stt.confidence_threshold`; on error it `tracing::warn!` and continues. SetInputDevice and SetPttBinding emit `tracing::warn!(restart-required)` and are ignored (deferred to S02/S03).

`DaemonHandle` in `src/control/mod.rs` gained a `runtime_cmd_tx: Option<Arc<mpsc::Sender<RuntimeCommand>>>` field, a `with_runtime_tx` builder, and a private `send_runtime_cmd` helper that returns `ControlResponse::Error{message: "pipeline not running"}` on either `None` or `SendError`. Five explicit match arms replace the `_ => Not yet implemented` catch-all for SetMode/SetThreshold/SetInputDevice/SetPttBinding/ReloadConfig. The `_ => Not yet implemented` catch-all is preserved as the TestMacro/unknown guard.

`src/main.rs` creates `let (runtime_tx, runtime_rx) = mpsc::channel::<RuntimeCommand>()` before `spawn_pipeline`, passes `runtime_rx` into the call, and chains `.with_runtime_tx(runtime_tx)` onto `DaemonHandle::new(...)`.

`Dispatcher` gained a `threshold()` accessor (reads from the `RwLock<PhraseMatcher>`) to enable old-value logging in the drain loop.

`tests/runtime_commands.rs` contains 6 integration tests: happy-path channel drain changing dispatch outcome, threshold clamping via channel, SetMode accepted by drain, channel-closed returns SendError, DaemonHandle without runtime_tx returns pipeline-not-running, and multiple commands processed in order with last-write winning.

## Verification

cargo check --lib --tests: clean (0 errors). cargo test --test runtime_commands: 6/6 pass. cargo test --test control_protocol: 17/17 pass. cargo test --lib dispatcher: 4/4 pass. cargo check --all-targets: clean. cargo build --release: clean. The flaky pack::tests::test_pack_export_import_with_sounds failure is pre-existing (passes in isolation, intermittent under parallel test load).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check --lib --tests` | 0 | ✅ pass | 580ms |
| 2 | `cargo test --test runtime_commands` | 0 | ✅ pass — 6 tests | 100ms |
| 3 | `cargo test --test control_protocol` | 0 | ✅ pass — 17 tests | 40ms |
| 4 | `cargo test --lib dispatcher` | 0 | ✅ pass — 4 tests | 30ms |
| 5 | `cargo check --all-targets` | 0 | ✅ pass | 800ms |
| 6 | `cargo build --release` | 0 | ✅ pass | 0ms |

## Deviations

Added `Dispatcher::threshold()` accessor (not in plan) — required because the drain loop logs old→new threshold values and the plan assumed the accessor already existed. Minor: the ReloadConfig hot-reload applies `stt.confidence_threshold` only when non-zero (avoids clobbering a runtime-adjusted threshold with a zero default); plan said apply `wake.enabled` and `stt.confidence_threshold` portions — wake.enabled is pipeline-structural (requires restart to take effect) and was omitted.

## Known Issues

none

## Files Created/Modified

- `src/pipeline/coordinator.rs`
- `src/pipeline/dispatcher.rs`
- `src/control/mod.rs`
- `src/main.rs`
- `tests/runtime_commands.rs`
