# S01: Control-protocol extensions

**Goal:** Extend the control protocol with five new ControlRequest variants (SetMode, SetThreshold, SetInputDevice, SetPttBinding, ReloadConfig) and wire a RuntimeCommand MPSC channel from the control server into the coordinator so the running daemon can honor mode and threshold changes between utterances without restarting the full pipeline.
**Demo:** cargo test passes including new control_protocol tests; round-trip serde test for each new request variant passes; coordinator accepts RuntimeCommand::SetMode and RuntimeCommand::SetThreshold without restarting the full pipeline

## Must-Haves

- After this: `cargo test --lib --tests` passes including new round-trip tests for each new ControlRequest variant; `cargo clippy --all-targets -- -D warnings` is clean for the default feature set; the coordinator drains a RuntimeCommand channel between utterance frames and applies SetMode (live activation-mode flip with PTT/wake state reset) and SetThreshold (live PhraseMatcher rebuild) without tearing down the pipeline. SetInputDevice/SetPttBinding return Ok with a tracing log noting restart-required (live application deferred to S02). ReloadConfig re-reads `config.yaml` and applies mode + threshold portions live. Threat Surface (Q3): control socket is 0600 user-only Unix domain socket — no new network attack surface; SetThreshold accepts an f32 that must be clamped to [0.0, 1.0] before use; SetPttBinding accepts an arbitrary string but is not applied this slice (returns Ok). Requirement Impact (Q4): touches ACT-03 (mode switch from UI — protocol substrate landed here) and STT-02/STT-03 (threshold tunable at runtime). Re-verify status round-trip and existing SwitchProfile after change. No prior decisions need reconsideration.

## Proof Level

- This slice proves: Contract + integration: protocol-layer serde round-trips prove wire-format correctness; a coordinator unit test (or focused integration test) proves a RuntimeCommand sent over the channel mutates dispatcher threshold and toggles activation mode without panicking. Real runtime not required for this slice — S04 owns the end-to-end UAT.

## Integration Closure

Upstream surfaces consumed: existing `ControlRequest` enum (`src/control/protocol.rs`), `DaemonHandle` (`src/control/mod.rs`), `Dispatcher` (`src/pipeline/dispatcher.rs`), `spawn_pipeline` (`src/pipeline/coordinator.rs`), and the `PhraseMatcher` constructor. New wiring introduced in this slice: `RuntimeCommand` enum (new module or inline in coordinator); `std::sync::mpsc::Sender<RuntimeCommand>` field on `DaemonHandle`; `runtime_rx` parameter on `spawn_pipeline`; `Dispatcher::update_threshold(f32)`; channel hookup in `src/main.rs`. What remains before the milestone is truly usable end-to-end: S02 (config UI surfaces these knobs and emits the new requests), S03 (tray icon state mapping + Mode submenu), S04 (UAT: voice-fired stratagem after mode flip, no restart).

## Verification

- Runtime signals: tracing::info! on each RuntimeCommand applied in the coordinator (e.g. `runtime_command_applied cmd=set_mode mode=wake`), tracing::info! on threshold change with old→new values, tracing::warn! when SetInputDevice/SetPttBinding are received (note: restart-required, not applied live this slice). Inspection surfaces: existing JSONL stdout stream is unchanged; control responses (Ok / Error{message}) are the synchronous inspection surface for the CLI sender. Failure visibility: if the RuntimeCommand channel is dropped (coordinator stopped), the control handler returns ControlResponse::Error{message} with a clear "pipeline not running" string. Redaction constraints: none — no PII or secrets flow through these commands.

## Tasks

- [x] **T01: Add ActivationMode enum and five new ControlRequest variants with round-trip tests** `est:45m`
  Extend the wire protocol to carry the new control requests M008/S02 will emit. Add an `ActivationMode { Ptt, Wake }` enum (serde rename_all=snake_case) and five new variants on `ControlRequest`: `SetMode { mode: ActivationMode }`, `SetThreshold { threshold: f32 }`, `SetInputDevice { device: Option<String> }`, `SetPttBinding { key: String }`, and unit variant `ReloadConfig`. Then add round-trip serde tests in `tests/control_protocol.rs` for each new variant (one test per variant) following the existing `status_request_roundtrip` pattern. Tests must assert both serialization to expected JSON shape and deserialization back into the matching variant. Important: the enum uses `#[serde(tag = "cmd", content = "args", rename_all = "snake_case")]` — adjacently tagged. Unit variants like `ReloadConfig` serialize as `{"cmd":"reload_config"}` (no `args` key). Confirm round-trip works for the unit variant (research flagged this as a potential pitfall). Do NOT remove the `_ => ControlResponse::Error { message: "Not yet implemented".into() }` arm in `control/mod.rs` — it stays as the forward-compatibility guard for `TestMacro` (out of scope this slice). This task does not modify any handler logic — pure type + test additions.

## Failure Modes

None — pure type additions and serde tests, no external dependencies.

## Negative Tests

- **Malformed inputs**: a deserialization test asserting `serde_json::from_str::<ControlRequest>("{\"cmd\":\"set_mode\",\"args\":{\"mode\":\"bogus\"}}")` returns `Err` (unknown ActivationMode value).
- **Unit variant**: explicit round-trip test for `ReloadConfig` proving the no-args form survives encode→decode.
  - Files: `src/control/protocol.rs`, `tests/control_protocol.rs`
  - Verify: cargo test --test control_protocol 2>&1 | tail -3 shows all tests pass (existing 11 + new round-trip tests for each of SetMode/SetThreshold/SetInputDevice/SetPttBinding/ReloadConfig + one negative malformed-mode test); `cargo clippy --all-targets -- -D warnings` clean.

- [x] **T02: Make Dispatcher threshold runtime-mutable via update_threshold()** `est:45m`
  Make the phrase-match confidence threshold mutable at runtime so the coordinator can apply SetThreshold without rebuilding the dispatcher. Wrap the `matcher: PhraseMatcher` field in `Dispatcher` (`src/pipeline/dispatcher.rs`) with `RwLock<PhraseMatcher>`. Update the constructor to wrap accordingly. Update `Dispatcher::process()` to acquire a read lock on the matcher before calling `find_best_match`. Add a new `pub fn update_threshold(&self, threshold: f32)` method that clamps the input to `[0.0, 1.0]`, acquires the write lock, replaces the inner `PhraseMatcher` with `PhraseMatcher::new(clamped)`, and emits `tracing::info!(old, new, "dispatcher threshold updated")`. Add a unit test in the existing `#[cfg(test)] mod tests` block (or a new one if none exists in dispatcher.rs) named `test_update_threshold_changes_match_behavior` that: constructs a Dispatcher with threshold 0.99 and a single macro phrase "eagle airstrike", asserts `process("eagal airstrike")` returns NoMatch (score below 0.99), then calls `update_threshold(0.5)` and asserts the same input now returns Fired. Existing matcher unit tests in `src/pipeline/matcher.rs` must continue to pass unchanged. The `unsafe impl Send/Sync for Dispatcher` blocks must remain (rodio constraint, MEM019); RwLock is Send+Sync so this does not affect that justification — keep the existing SAFETY comments intact.

## Failure Modes

| Dependency | On error | On timeout | On malformed response |
|------------|----------|-----------|----------------------|
| RwLock<PhraseMatcher> poison (only on panic during write) | propagate via `.unwrap()` consistent with existing `macros: Arc<RwLock<Vec<MacroConfig>>>` usage in this file | N/A (no timeout) | N/A |

## Load Profile

- **Shared resources**: one additional RwLock on the dispatcher hot path (acquired read-side per utterance).
- **Per-operation cost**: one read-lock acquisition per `process()` call (microseconds, uncontended).
- **10x breakpoint**: write-side calls (update_threshold) are rare (user-initiated); no breakpoint at expected load.

## Negative Tests

- **Boundary conditions**: `update_threshold(-0.5)` clamps to 0.0; `update_threshold(2.0)` clamps to 1.0; `update_threshold(f32::NAN)` clamps to 1.0 (or 0.0 — pick one and document; recommend NAN→0.0 via `.max(0.0).min(1.0)` semantics with NaN comparison short-circuit).
- **Behavior verification**: the test described above (threshold-flip changes match outcome) is the primary negative-path proof.
  - Files: `src/pipeline/dispatcher.rs`
  - Verify: cargo test --lib pipeline::dispatcher 2>&1 | tail -5 shows new `test_update_threshold_changes_match_behavior` passes; cargo test --lib pipeline::matcher remains green; `cargo clippy --all-targets -- -D warnings` clean.

- [ ] **T03: Wire RuntimeCommand channel from control handler into coordinator and apply live mode/threshold changes** `est:1h30m`
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
  - Files: `src/pipeline/coordinator.rs`, `src/control/mod.rs`, `src/control/protocol.rs`, `src/main.rs`, `tests/runtime_commands.rs`
  - Verify: cargo test --lib --tests 2>&1 | tail -5 — all tests pass including the new runtime_commands test (or coordinator unit test); `cargo clippy --all-targets -- -D warnings` clean; `cargo build --release` succeeds; manual smoke (optional): `cargo run --release` + `echo '{"cmd":"set_threshold","args":{"threshold":0.6}}' | nc -U $XDG_RUNTIME_DIR/vibe-attack/vibe-attack.sock` returns `{"status":"ok"}`.

## Files Likely Touched

- src/control/protocol.rs
- tests/control_protocol.rs
- src/pipeline/dispatcher.rs
- src/pipeline/coordinator.rs
- src/control/mod.rs
- src/main.rs
- tests/runtime_commands.rs
