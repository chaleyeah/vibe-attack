# S03: Tray icon state mapping + Mode submenu — Research

**Date:** 2026-04-27
**Status:** Targeted research — known patterns, straightforward extension of existing tray code

## Summary

S03 is the lowest-risk slice in M008. The tray (`src/ui/tray.rs`) already polls daemon state every second via `query_status()` (UDS control socket), updates `TrayState`, and calls `poll_handle.update()` only on change. Icon names and tooltips are already partially mapped for all four `DaemonState` variants — the tooltip is complete but the icon mapping collapses Idle/Listening/Recording into one icon name. The work here is: (1) add per-state icon names, (2) add `active_mode: Option<ActivationMode>` to `TrayState` and `DaemonStatus`, (3) add a Mode submenu to `menu()`, and (4) add unit tests for the icon mapping.

**Key constraint discovered:** `DaemonStatus` (protocol.rs) and `DaemonHandle` (control/mod.rs) do not currently track `active_mode`. The coordinator tracks it as a local `let mut active_mode` inside the pipeline thread, but it is not exposed back to the control layer. To display the current mode in the tray submenu with the correct checkmark, S03 must add `active_mode: Option<ActivationMode>` to `DaemonStatus` and wire `DaemonHandle` to expose it via an `Arc<RwLock<ActivationMode>>` (matching the existing `active_profile` pattern).

The roadmap says "tray reads JSONL stdout" for state, but the tray already uses `query_status()` via control socket for all existing state. The architectural decision for JSONL was specifically to avoid extending the control protocol for push-state, but since `active_mode` needs to be readable (not just writable), adding it to the existing `Status` response is the minimal-impact path — no new JSONL parsing infrastructure required.

## Recommendation

Extend `DaemonStatus` with `active_mode: ActivationMode` (default PTT), add `Arc<RwLock<ActivationMode>>` to `DaemonHandle` (updated by control server on `SetMode`), then update tray to: read `active_mode` from polled status, render per-state icon names, and add Mode submenu with CheckmarkItems. Write a unit test that constructs a `VibeTray`-equivalent function mapping `DaemonState → icon_name` for each variant.

Do NOT attempt JSONL stdout parsing in the tray — the control socket poll already gives everything needed and adding a subprocess stdout parser would be a large scope increase.

## Implementation Landscape

### Key Files

- `src/control/protocol.rs` — Add `active_mode: ActivationMode` field to `DaemonStatus`. This is a breaking serde change but only affects internal consumers (all in the same crate). Default to `ActivationMode::Ptt` for `DaemonHandle::status()`.
- `src/control/mod.rs` — Add `active_mode: Arc<RwLock<ActivationMode>>` to `DaemonHandle`. Initialize to `Ptt`. Update it in the `SetMode` handler (before forwarding to coordinator via `send_runtime_cmd`). Expose via `status()`.
- `src/ui/tray.rs` — (1) Add `active_mode: Option<ActivationMode>` to `TrayState`. (2) Poll it from `query_status()` result. (3) Expand `icon_name()` match to return distinct icon per state. (4) Add Mode submenu in `menu()` with two `CheckmarkItem`s (PTT / Wake word), checkmarked by `active_mode`. (5) Unit tests for icon mapping.
- `tests/control_protocol.rs` — Update `status_data_response_roundtrip` to include `active_mode` field. No other tests break.

### Build Order

1. **`protocol.rs` first** — add `active_mode` to `DaemonStatus`. Everything else depends on this field existing.
2. **`control/mod.rs` second** — add `Arc<RwLock<ActivationMode>>` to `DaemonHandle`, wire into `SetMode` handler and `status()`. Fixes the compile break from step 1.
3. **`tray.rs` third** — add `active_mode` to `TrayState`, update poll loop, add per-state icon mapping, add Mode submenu, add unit tests.
4. **`tests/control_protocol.rs` last** — update the `status_data_response_roundtrip` test to include the new field; verify all tests pass.

### Verification Approach

```
cargo test                   # all non-hardware-gated tests pass
cargo build                  # default features, clean
cargo build --features gui   # gui feature, clean
```

Unit tests to add in `tray.rs` (pure functions, no D-Bus required):
- `icon_name_for_none_is_muted` — `None` daemon state → `audio-input-microphone-muted`
- `icon_name_for_idle` — `Some(Idle)` → `audio-input-microphone`
- `icon_name_for_listening` — `Some(Listening)` → `audio-input-microphone-high` (or chosen name)
- `icon_name_for_recording` — `Some(Recording)` → `audio-input-microphone` (active capture)
- `icon_name_for_muted` — `Some(Muted)` → `audio-input-microphone-muted`

Extract the match into a free function `icon_name_for_state(state: Option<&DaemonState>) -> &'static str` so tests can call it without spawning a tray.

## Constraints

- `ksni` callbacks (`activate` closures in `menu()`) must not block — existing pattern uses `std::thread::spawn` fire-and-forget for `send_command`. Mode submenu follows the same pattern.
- The `try_lock()` in `current_state()` can return the mutex poisoned / contended; the `unwrap_or_default()` fallback is already in place and correct — `TrayState::default()` gives `active_mode: None` which renders PTT as unchecked (acceptable fallback).
- `ActivationMode` already derives `PartialEq` (confirmed in S02 summary) — no change needed for checkmark comparison.
- Clippy not installed on this build system (pre-existing environment constraint from S02); substitute `cargo build` for `cargo clippy` gates throughout.

## Common Pitfalls

- **`DaemonStatus` serde break** — Adding `active_mode` to `DaemonStatus` without a `#[serde(default)]` will break deserialization of any JSON that lacks the field. Add `#[serde(default)]` on the field (with `Default` on `ActivationMode` defaulting to `Ptt`) so existing CLI clients are not broken. Alternatively derive `Default` for `ActivationMode` to make `#[serde(default)]` trivially work.
- **Mode submenu enabled state** — The Mode submenu should be `enabled: daemon_running` (same as the mute button), not always enabled. Sending `SetMode` when no daemon is running returns an error; disable submenu items when `daemon_running` is false.
- **Do not read `active_mode` from coordinator** — The coordinator owns `active_mode` as a local variable inside the pipeline thread. Do not try to add an `Arc<AtomicU8>` in the coordinator for this; instead track it in `DaemonHandle` via `Arc<RwLock<ActivationMode>>` updated by the control server's `SetMode` handler (before forwarding the command). This is the `active_profile` pattern already in the codebase.
