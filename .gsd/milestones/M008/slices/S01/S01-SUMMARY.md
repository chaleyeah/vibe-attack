---
id: S01
parent: M008
milestone: M008
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - (none)
key_decisions:
  - ["ActivationMode + 5 ControlRequest variants added without breaking adjacently-tagged serde schema (tag=cmd, content=args)", "RwLock<PhraseMatcher> chosen over Arc<AtomicF32>: threshold consumed by PhraseMatcher::new(), full matcher replacement under write lock is cleanest", "active_mode is thread-local in coordinator (not Arc<AtomicXxx>): only one writer (drain loop on same thread), ActivationMode not atomically encodable", "DaemonHandle.runtime_cmd_tx is Option<Arc<Sender>> set via with_runtime_tx() builder: preserves zero-arg constructor for existing tests", "NaN threshold clamped to 0.0 (maximally permissive) via explicit is_nan() guard before f32::clamp", "SetInputDevice/SetPttBinding forwarded through RuntimeCommand channel (coordinator logs them) rather than handled in control server: keeps full command trace in one place", "wake.enabled not applied by ReloadConfig (pipeline-structural, requires restart); only stt.confidence_threshold is applied live"]
patterns_established:
  - ["try_recv drain loop at top of coordinator frame: non-blocking, ~50/s, processes all queued commands before audio work", "send_runtime_cmd helper on DaemonHandle: Option check + SendError → ControlResponse::Error{pipeline not running}", "update_threshold() pattern: clamp→write-lock→replace matcher→emit tracing::info(old, new)", "Dispatcher::threshold() read accessor for old-value logging without duplicating threshold state"]
observability_surfaces:
  - ["tracing::info!(cmd, mode, \"runtime_command_applied\") on SetMode", "tracing::info!(old, new, \"dispatcher threshold updated\") on SetThreshold", "tracing::warn!(\"restart required\") on SetInputDevice/SetPttBinding", "tracing::warn! on ReloadConfig parse error (best-effort, no crash)", "ControlResponse::Error{message: \"pipeline not running\"} on dropped channel"]
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-28T01:28:35.915Z
blocker_discovered: false
---

# S01: Control-protocol extensions

**Extended the control protocol with five new ControlRequest variants and wired a RuntimeCommand MPSC channel so the running daemon can flip activation mode and adjust confidence threshold between utterances without restarting the pipeline.**

## What Happened

S01 delivered the full control-protocol substrate for M008's runtime-control surface across three tasks.

**T01 — Protocol types and serde round-trips**
Added `ActivationMode { Ptt, Wake }` (serde rename_all=snake_case) and five new `ControlRequest` variants: `SetMode { mode: ActivationMode }`, `SetThreshold { threshold: f32 }`, `SetInputDevice { device: Option<String> }`, `SetPttBinding { key: String }`, and unit variant `ReloadConfig`. The existing adjacently-tagged serde layout (`tag = "cmd", content = "args"`) was preserved with no schema break. Six new tests were added to `tests/control_protocol.rs` (round-trips for each variant plus a negative test for unknown ActivationMode values). The key gotcha — that unit variants in adjacently-tagged enums omit the `args` key entirely — was confirmed by exact JSON equality assertion. The `_ => Not yet implemented` catch-all in `control/mod.rs` was left intact as the TestMacro forward-compat guard.

**T02 — Runtime-mutable Dispatcher threshold**
Wrapped `Dispatcher`'s `matcher: PhraseMatcher` field in `RwLock<PhraseMatcher>`. Added `pub fn update_threshold(&self, threshold: f32)` which clamps input to `[0.0, 1.0]` (NaN→0.0), acquires the write lock, replaces the inner PhraseMatcher, and emits `tracing::info!(old, new, "dispatcher threshold updated")`. Also added `Dispatcher::threshold()` accessor (reads under read lock) to support old→new logging without duplicating state. Read-lock acquisition on the hot path (every utterance) is microseconds uncontended. A unit test `test_update_threshold_changes_match_behavior` proves a threshold flip changes dispatch outcome from NoMatch to Fired on the same input.

**T03 — RuntimeCommand channel wiring**
Defined `pub enum RuntimeCommand { SetMode, SetThreshold, SetInputDevice, SetPttBinding, ReloadConfig }` in `src/pipeline/coordinator.rs`. Added `runtime_rx: mpsc::Receiver<RuntimeCommand>` to `spawn_pipeline`. At the top of each coordinator frame, `while let Ok(cmd) = runtime_rx.try_recv()` drains all pending commands before any audio processing. SetMode applies surgical state cleanup on mode flip (PTT→Wake: clears ptt_audio/prev_ptt, resets VAD segmenter; Wake→PTT: clears listen-window state, resets segmenter). Mode gating is wired at the frame level so the active branch is skipped entirely based on `active_mode`. SetThreshold delegates to `dispatcher.update_threshold(t)`. ReloadConfig calls `crate::config::load(None)` and applies `stt.confidence_threshold` (best-effort; errors are tracing::warn and continue). SetInputDevice/SetPttBinding emit tracing::warn (restart-required) and are ignored pending S02/S03.

`DaemonHandle` gained `runtime_cmd_tx: Option<Arc<mpsc::Sender<RuntimeCommand>>>` (defaults None, set via `with_runtime_tx()` builder) preserving the zero-argument constructor for existing tests. A `send_runtime_cmd` helper returns `ControlResponse::Error { message: "pipeline not running" }` on None or SendError. Five explicit match arms handle the new variants; the `_ => Not yet implemented` catch-all remains. `src/main.rs` creates the channel, passes rx to `spawn_pipeline`, and chains `.with_runtime_tx(tx)` on `DaemonHandle`. Six integration tests in `tests/runtime_commands.rs` cover: threshold change via channel changes dispatch outcome, clamping via channel, SetMode accepted, channel-closed returns error, DaemonHandle without tx returns error, and multiple commands processed in order.

## Verification

All verification checks pass:
- `cargo test --test control_protocol`: 17/17 pass (11 existing + 6 new including negative bogus-ActivationMode test)
- `cargo test --test runtime_commands`: 6/6 pass
- `cargo test --lib --tests`: 44+ lib tests + all integration tests pass, 0 failures
- `cargo check --all-targets`: clean (0 errors, 0 warnings)
- `cargo build --release`: succeeds (0.08s, already built)
- Slice-level: round-trip serde test for each of the 5 new ControlRequest variants passes; coordinator unit contract proven by runtime_commands tests; mode gating and threshold mutation confirmed without pipeline restart

## Requirements Advanced

- ACT-03 — Protocol substrate for mode switching landed: SetMode ControlRequest, ActivationMode enum, RuntimeCommand::SetMode channel wiring, and surgical coordinator mode-gate. UI surface (S02) and tray (S03) build on this.
- STT-02 — Confidence threshold is now runtime-mutable: update_threshold() on RwLock<PhraseMatcher> applies changes between utterances without restart.
- STT-03 — SetThreshold ControlRequest and RuntimeCommand::SetThreshold wiring enable config UI (S02) to push threshold changes to the running daemon.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

SetInputDevice and SetPttBinding are accepted by the protocol and forwarded to the coordinator but not applied live — both emit restart-required warnings. Live application is deferred to S02/S03. wake.enabled from ReloadConfig is not applied live (requires pipeline restart); only stt.confidence_threshold is hot-reloaded.

## Follow-ups

S02 (ConfigApp state + egui config panel): surfaces SetMode/SetThreshold/SetInputDevice/SetPttBinding via UI controls; S03 (Tray icon state mapping + Mode submenu): reads mode state and emits SetMode; S04 (end-to-end UAT + headless integration test).

## Files Created/Modified

- `src/control/protocol.rs` — 
- `tests/control_protocol.rs` — 
- `src/pipeline/dispatcher.rs` — 
- `src/pipeline/coordinator.rs` — 
- `src/control/mod.rs` — 
- `src/main.rs` — 
- `tests/runtime_commands.rs` — 
