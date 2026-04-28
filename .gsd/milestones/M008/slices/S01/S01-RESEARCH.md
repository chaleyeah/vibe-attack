# M008/S01 — Control-protocol extensions — Research

**Date:** 2026-04-27
**Status:** Ready for planning

## Summary

S01 adds five new `ControlRequest` variants (`SetMode`, `SetThreshold`, `SetInputDevice`, `SetPttBinding`, `ReloadConfig`) to the existing Unix-socket control plane, wires server-side handlers in `control/mod.rs`, and introduces a `RuntimeCommand` MPSC channel that lets the control server signal the coordinator to perform live changes (mode swap, threshold update) between utterances. No new process or socket is introduced — this is a pure extension of the existing protocol.

The codebase is clean and well-structured for this work. `ControlRequest` already uses `#[serde(tag = "cmd", content = "args", rename_all = "snake_case")]`, so adding new variants is mechanical. The coordinator (`spawn_pipeline`) runs a tight loop on an OS thread with no existing command channel — we will add one. The `PhraseMatcher` holds threshold as a constructor arg with no runtime setter — the coordinator must rebuild and swap it on threshold change. The `Dispatcher` owns the matcher and is `Arc`-shared; the cleanest approach per the locked architectural decision is for the coordinator to own a `RuntimeCommand` channel and rebuild the dispatcher (or patch it via a new `update_threshold` method) between utterances.

Baseline: `cargo test --test control_protocol` passes (11 tests, 0 failures). The `_ =>` catch-all at `control/mod.rs:138` currently returns "Not yet implemented" for any unrecognized variant — the new variants will replace this path with real handlers.

## Recommendation

Implement in two layers:

1. **Protocol layer** (`src/control/protocol.rs`): Add the five new `ControlRequest` variants and the `ActivationMode` enum. Add a `RuntimeCommand` enum in a new module or inline in `src/pipeline/coordinator.rs`. Add serde round-trip tests in `tests/control_protocol.rs` for each new variant.

2. **Handler + coordinator layer** (`src/control/mod.rs`, `src/pipeline/coordinator.rs`): Wire a `std::sync::mpsc::Sender<RuntimeCommand>` through `DaemonHandle` → control handler → coordinator loop. The coordinator drains this channel at the top of its per-frame loop (cheap, non-blocking `try_recv`). `SetMode` tears down only the activation mode flag and resets the relevant VAD/wake state (the pipeline thread's local state); `SetThreshold` rebuilds the `PhraseMatcher` inside the `Dispatcher` via a new `update_threshold` method. `SetInputDevice` and `SetPttBinding` return `ControlResponse::Ok` with a note that these require restart (deferred to S02/S03); `ReloadConfig` re-reads `config.yaml` and applies mode/threshold portions live.

This approach keeps all mutable runtime state on the coordinator's own OS thread — no cross-thread lock contention on the audio hot path.

## Implementation Landscape

### Key Files

- `src/control/protocol.rs` — Add `ActivationMode { Ptt, Wake }` enum (serde snake_case); add `SetMode { mode: ActivationMode }`, `SetThreshold { threshold: f32 }`, `SetInputDevice { device: Option<String> }`, `SetPttBinding { key: String }`, `ReloadConfig` to `ControlRequest`. No breaking change — `#[serde(tag = "cmd", content = "args")]` handles new variants transparently; unknown variants already return error via `serde_json::from_str` failure path.

- `src/control/mod.rs` — Add `runtime_cmd_tx: Arc<Mutex<Option<std::sync::mpsc::Sender<RuntimeCommand>>>>` (or simpler: a plain `Option<std::sync::mpsc::Sender<RuntimeCommand>>` added to `DaemonHandle`). Handler match arms for each new variant: `SetMode` / `SetThreshold` / `ReloadConfig` send `RuntimeCommand` via the channel; `SetInputDevice` / `SetPttBinding` return `Ok` (with log noting restart-required). Remove the `TestMacro` arm from the `_ =>` catch-all path (it's already in the enum but not handled — check line 138; keep the catch-all for truly unknown future variants but make it unreachable for known-but-unimplemented ones).

- `src/pipeline/coordinator.rs` — `spawn_pipeline` signature gains `runtime_rx: std::sync::mpsc::Receiver<RuntimeCommand>`. At the top of the per-frame `while` loop (before the STT result drain), call `while let Ok(cmd) = runtime_rx.try_recv()` and match: `SetMode` → update a local `active_mode: ActivationMode` flag; if switching PTT→Wake, reset `listening_until`/`ptt_audio`/VAD segmenter; if switching Wake→PTT, do the reverse. `SetThreshold` → call `dispatcher.update_threshold(t)`. `ReloadConfig` → re-read config file, apply `mode` and `threshold` fields only.

- `src/pipeline/dispatcher.rs` — Add `pub fn update_threshold(&self, threshold: f32)`. The `PhraseMatcher` is currently a plain value field; it must become `Arc<RwLock<PhraseMatcher>>` or be replaced inline. Simpler: add `threshold: Arc<atomic...>` — but `f32` isn't atomic. Cleanest: wrap `matcher` in `RwLock<PhraseMatcher>` inside `Dispatcher`, then `update_threshold` acquires write lock and replaces it. This is safe because `Dispatcher` is `Arc`-shared across coordinator + dispatcher threads and already `unsafe impl Sync`.

- `tests/control_protocol.rs` — Add round-trip serde tests for each of the five new `ControlRequest` variants. Follow the existing `status_request_roundtrip` pattern. These tests are purely serialization-layer (no daemon process needed) and should cover: unit variant (`ReloadConfig`), struct variants with args (`SetMode { mode: ActivationMode::Wake }`, `SetThreshold { threshold: 0.75 }`, `SetInputDevice { device: None }`, `SetPttBinding { key: "KEY_F14".into() }`).

### Build Order

1. **`src/control/protocol.rs`** first — unblocks everything; compile-time guarantee no handler references a missing variant.
2. **`src/pipeline/dispatcher.rs`** — add `update_threshold`; wrap `matcher` in `RwLock`; confirm `cargo test` still passes (existing matcher unit tests must still compile and pass).
3. **`src/pipeline/coordinator.rs`** — add `RuntimeCommand` enum and `runtime_rx` parameter; drain it in the per-frame loop; integrate `active_mode` logic.
4. **`src/control/mod.rs`** — add `runtime_cmd_tx` to `DaemonHandle`; wire new match arms.
5. **`tests/control_protocol.rs`** — add serde round-trip tests for new variants.
6. **`cargo test`** (non-hardware-gated) — baseline must stay green throughout.

### Verification Approach

```
# All non-hardware-gated tests pass
cargo test --lib --tests 2>&1 | tail -5

# Specific control protocol tests
cargo test --test control_protocol

# Clippy clean (both feature sets)
cargo clippy -- -D warnings
cargo clippy --features gui -- -D warnings

# Manual smoke: send SetMode via CLI (after daemon runs)
echo '{"cmd":"set_mode","args":{"mode":"wake"}}' | nc -U /run/user/$(id -u)/vibe-attack/vibe-attack.sock
# Expect: {"status":"ok"}
```

## Constraints

- `ControlRequest` uses `#[serde(tag = "cmd", content = "args", rename_all = "snake_case")]`. Unit variants (no fields) like `ReloadConfig` serialize as `{"cmd":"reload_config"}` with no `"args"` key. Struct variants like `SetMode { mode }` serialize as `{"cmd":"set_mode","args":{"mode":"wake"}}`. Tests must reflect this exact JSON shape.
- `PhraseMatcher` threshold is currently a plain `f32` constructor arg with no setter. Wrapping it in `RwLock` inside `Dispatcher` is the minimal change; do not reach into `PhraseMatcher`'s internals from outside.
- The coordinator loop runs on a plain OS thread (`std::thread::spawn`), not Tokio. The `RuntimeCommand` channel must be `std::sync::mpsc`, not `tokio::sync::mpsc`.
- `spawn_pipeline` signature change adds `runtime_rx` — update all call sites (currently just `src/main.rs`). The sender half lives in `DaemonHandle` which is `Clone`; use `Arc<Mutex<Sender<RuntimeCommand>>>` or create the channel before calling `spawn_pipeline` and thread both halves through explicitly.
- `TestMacro` is in the `ControlRequest` enum but falls into `_ =>` (unimplemented) at `mod.rs:138`. S01 should not implement it (out of scope) but should not accidentally remove the `_ =>` guard needed for forward compatibility.
- The `active_mode` state lives in the coordinator pipeline thread's local scope (not `DaemonHandle`) since the architectural decision is to avoid shared mutable state for the audio hot path. The control socket handler sends commands and gets back `Ok` immediately — it does not wait for the coordinator to apply them.

## Common Pitfalls

- **`content = "args"` with unit variants** — `#[serde(tag = "cmd", content = "args")]` makes serde use *adjacently tagged* format. Unit variants (no fields) do NOT emit an `"args"` key at all during serialization, but serde may fail to *deserialize* `{"cmd":"reload_config"}` if the variant is declared with `content` tag and no fields. Verify with an actual round-trip test; do not assume it works.
- **`RwLock` write on dispatcher from Tokio context** — `handle_switch_profile` already uses `tokio::task::block_in_place` to avoid blocking the async executor. Any new handler that calls `dispatcher.update_threshold()` (or sends a RuntimeCommand) must also be non-blocking — sending on an `mpsc::Sender` is non-blocking (returns immediately), which is fine.
- **Mode swap without resetting VAD/STT state** — switching from PTT to Wake while a PTT utterance is in flight could leave `ptt_audio` non-empty. The coordinator must drain or discard `ptt_audio` on mode swap to avoid submitting stale audio to STT.
- **`ActivationMode` in config vs. as a runtime concept** — the current config has no `ActivationMode` enum; activation mode is implicit (`wake.enabled` + existence of `PttConfig`). S01 introduces `ActivationMode` as a protocol/runtime concept. The config YAML schema does not change in S01 (out of scope per boundary map); only the runtime state tracks it.
