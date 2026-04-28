---
id: T01
parent: S03
milestone: M008
key_files:
  - src/control/protocol.rs
  - src/control/mod.rs
  - tests/control_protocol.rs
key_decisions:
  - Used Arc<RwLock<ActivationMode>> on DaemonHandle (not AtomicXxx) — ActivationMode is not atomically encodable; matches MEM030 noting pipeline-local copy stays unshared while the handle holds the readable snapshot.
  - SetMode handler writes active_mode before forwarding to coordinator, matching the active_profile write pattern, so Status responses are always coherent with the last SetMode request.
duration: 
verification_result: passed
completed_at: 2026-04-28T01:49:23.331Z
blocker_discovered: false
---

# T01: Exposed active_mode through DaemonStatus and DaemonHandle with serde default, SetMode cache-write, and 5 new tests

**Exposed active_mode through DaemonStatus and DaemonHandle with serde default, SetMode cache-write, and 5 new tests**

## What Happened

Added `Default for ActivationMode` (returns `Ptt`) and `active_mode: ActivationMode` with `#[serde(default)]` to `DaemonStatus` in `src/control/protocol.rs`. Added `active_mode: Arc<RwLock<ActivationMode>>` to `DaemonHandle`, initialized to `ActivationMode::Ptt` in `DaemonHandle::new()`, and exposed it from `DaemonHandle::status()`. Updated the `SetMode` handler in `spawn_control_listener` to write the new value into the handle before forwarding `RuntimeCommand::SetMode` to the coordinator (matching the `active_profile` pattern). Added a `tracing::debug!` line logging the cached mode value for observability. Updated `status_data_response_roundtrip` to construct and assert `active_mode: ActivationMode::Wake`. Added four new tests: `daemon_handle_active_mode_defaults_to_ptt`, `daemon_handle_active_mode_updates_on_write`, `status_active_mode_serializes_snake_case`, and `daemon_status_backward_compat_no_active_mode_field` (verifying old JSON without the field deserializes correctly via serde default).

## Verification

Ran `cargo test --test control_protocol` (21/21 pass), `cargo build` (clean), `cargo build --features gui` (clean). All three verification commands from the task plan passed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test control_protocol` | 0 | ✅ pass — 21 tests passed | 2890ms |
| 2 | `cargo build` | 0 | ✅ pass | 2560ms |
| 3 | `cargo build --features gui` | 0 | ✅ pass | 6070ms |

## Deviations

Added three extra tests beyond the two specified (daemon_handle_active_mode_updates_on_write, status_active_mode_serializes_snake_case, daemon_status_backward_compat_no_active_mode_field) to cover the serde default path and the tray-relevant snake_case serialization — these were implied by the task narrative but not explicitly named.

## Known Issues

none

## Files Created/Modified

- `src/control/protocol.rs`
- `src/control/mod.rs`
- `tests/control_protocol.rs`
