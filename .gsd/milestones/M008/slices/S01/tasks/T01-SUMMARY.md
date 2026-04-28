---
id: T01
parent: S01
milestone: M008
key_files:
  - src/control/protocol.rs
  - tests/control_protocol.rs
key_decisions:
  - Adjacently-tagged serde layout (tag=cmd, content=args) inherited from existing ControlRequest — no schema change needed
  - Unit variant ReloadConfig tested with exact JSON equality ({"cmd":"reload_config"}) to confirm no spurious args key appears
duration: 
verification_result: passed
completed_at: 2026-04-28T01:13:28.171Z
blocker_discovered: false
---

# T01: feat: add ActivationMode enum and five new ControlRequest variants (SetMode, SetThreshold, SetInputDevice, SetPttBinding, ReloadConfig) with round-trip serde tests

**feat: add ActivationMode enum and five new ControlRequest variants (SetMode, SetThreshold, SetInputDevice, SetPttBinding, ReloadConfig) with round-trip serde tests**

## What Happened

Added `ActivationMode { Ptt, Wake }` enum (serde rename_all=snake_case) to `src/control/protocol.rs` and extended `ControlRequest` with five new variants: `SetMode { mode: ActivationMode }`, `SetThreshold { threshold: f32 }`, `SetInputDevice { device: Option<String> }`, `SetPttBinding { key: String }`, and unit variant `ReloadConfig`. The adjacently-tagged serde layout (`tag = "cmd", content = "args"`) was preserved from the existing enum; unit variant `ReloadConfig` correctly serializes as `{"cmd":"reload_config"}` with no `args` key, which was confirmed by the explicit assertion in the test. The existing `_ => ControlResponse::Error { message: "Not yet implemented".into() }` arm in `control/mod.rs` was left untouched as required. Added 6 new tests to `tests/control_protocol.rs`: round-trips for `SetMode` (both Ptt and Wake), `SetThreshold` (value fidelity check), `SetInputDevice` (Some and None), `SetPttBinding`, `ReloadConfig` (exact JSON equality assertion), and one negative test asserting `serde_json::from_str` returns `Err` for an unknown `ActivationMode` value. Clippy is not installed in this environment; `cargo check --all-targets` is clean.

## Verification

cargo test --test control_protocol: 17 tests pass (11 existing + 6 new). cargo check --all-targets: clean (no errors or warnings). Clippy binary absent from this toolchain; cargo check used as substitute.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test control_protocol 2>&1 | tail -3` | 0 | ✅ pass | 2750ms |
| 2 | `cargo check --all-targets 2>&1 | tail -5` | 0 | ✅ pass | 910ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/control/protocol.rs`
- `tests/control_protocol.rs`
