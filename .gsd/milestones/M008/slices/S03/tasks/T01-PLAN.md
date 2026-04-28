---
estimated_steps: 2
estimated_files: 3
skills_used: []
---

# T01: Expose active_mode through DaemonStatus and DaemonHandle

Plumb the daemon's current ActivationMode through the control protocol so the tray can read it. Add `active_mode: ActivationMode` to `DaemonStatus`. Derive `Default` for `ActivationMode` returning `Ptt` so `#[serde(default)]` on the new field keeps existing JSON readable. Add `active_mode: Arc<RwLock<ActivationMode>>` to `DaemonHandle`, initialise it to `Ptt` in `DaemonHandle::new`, expose it from `DaemonHandle::status()`, and update the `SetMode` handler in `spawn_control_listener` to write the new value into the handle BEFORE forwarding `RuntimeCommand::SetMode` to the coordinator (matches the active_profile pattern). Update `tests/control_protocol.rs::status_data_response_roundtrip` to construct/assert the new field. Add a new test `daemon_handle_active_mode_defaults_to_ptt`. Per project convention (MEM002), every new pub item gets a `///` doc comment.

Failure modes are limited to RwLock poisoning — follow the existing `active_profile.write().unwrap()` pattern (a poisoned lock here means the process is already toast).

## Inputs

- ``src/control/protocol.rs``
- ``src/control/mod.rs``
- ``tests/control_protocol.rs``

## Expected Output

- ``src/control/protocol.rs` — adds Default for ActivationMode and active_mode field on DaemonStatus with #[serde(default)]`
- ``src/control/mod.rs` — adds active_mode: Arc<RwLock<ActivationMode>> to DaemonHandle, initializes in new(), exposes via status(), and writes it in the SetMode handler before forwarding to coordinator`
- ``tests/control_protocol.rs` — updates status_data_response_roundtrip to set/assert active_mode and adds daemon_handle_active_mode_defaults_to_ptt test`

## Verification

cargo test --test control_protocol && cargo build && cargo build --features gui

## Observability Impact

Adds active_mode to the existing tracing::debug for SetMode handling so JSONL/log readers can correlate a SetMode request with the cached value used by Status responses.
