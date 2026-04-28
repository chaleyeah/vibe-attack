---
estimated_steps: 6
estimated_files: 2
skills_used: []
---

# T01: Add ActivationMode enum and five new ControlRequest variants with round-trip tests

Extend the wire protocol to carry the new control requests M008/S02 will emit. Add an `ActivationMode { Ptt, Wake }` enum (serde rename_all=snake_case) and five new variants on `ControlRequest`: `SetMode { mode: ActivationMode }`, `SetThreshold { threshold: f32 }`, `SetInputDevice { device: Option<String> }`, `SetPttBinding { key: String }`, and unit variant `ReloadConfig`. Then add round-trip serde tests in `tests/control_protocol.rs` for each new variant (one test per variant) following the existing `status_request_roundtrip` pattern. Tests must assert both serialization to expected JSON shape and deserialization back into the matching variant. Important: the enum uses `#[serde(tag = "cmd", content = "args", rename_all = "snake_case")]` — adjacently tagged. Unit variants like `ReloadConfig` serialize as `{"cmd":"reload_config"}` (no `args` key). Confirm round-trip works for the unit variant (research flagged this as a potential pitfall). Do NOT remove the `_ => ControlResponse::Error { message: "Not yet implemented".into() }` arm in `control/mod.rs` — it stays as the forward-compatibility guard for `TestMacro` (out of scope this slice). This task does not modify any handler logic — pure type + test additions.

## Failure Modes

None — pure type additions and serde tests, no external dependencies.

## Negative Tests

- **Malformed inputs**: a deserialization test asserting `serde_json::from_str::<ControlRequest>("{\"cmd\":\"set_mode\",\"args\":{\"mode\":\"bogus\"}}")` returns `Err` (unknown ActivationMode value).
- **Unit variant**: explicit round-trip test for `ReloadConfig` proving the no-args form survives encode→decode.

## Inputs

- ``src/control/protocol.rs` — existing ControlRequest/ControlResponse/DaemonState/DaemonStatus definitions`
- ``tests/control_protocol.rs` — existing round-trip test pattern (status_request_roundtrip, mute_request_roundtrip)`

## Expected Output

- ``src/control/protocol.rs` — adds `pub enum ActivationMode { Ptt, Wake }` with serde derives, and 5 new variants on `ControlRequest``
- ``tests/control_protocol.rs` — adds 5 round-trip tests (one per new variant) plus 1 malformed-mode negative test`

## Verification

cargo test --test control_protocol 2>&1 | tail -3 shows all tests pass (existing 11 + new round-trip tests for each of SetMode/SetThreshold/SetInputDevice/SetPttBinding/ReloadConfig + one negative malformed-mode test); `cargo clippy --all-targets -- -D warnings` clean.
