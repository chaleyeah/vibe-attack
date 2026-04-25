---
id: T04
parent: S01
milestone: M001
provides: []
requires: []
affects: []
key_files: []
key_decisions: []
patterns_established: []
observability_surfaces: []
drill_down_paths: []
duration: 
verification_result: passed
completed_at: 
blocker_discovered: false
---
# T04: 01-foundation 04

**# Phase 01 Plan 04: uinput Injection Subsystem Summary**

## What Happened

# Phase 01 Plan 04: uinput Injection Subsystem Summary

**One-liner:** Keyboard-only VirtualDevice with MacroCmd channel, dwell+gap injection thread, and D-15 'input' group error on permission denied.

## Tasks Completed

| # | Task | Commit | Files |
|---|------|--------|-------|
| 1 (RED) | Failing tests for inject + D-15 error | e3dea33 | src/input/inject.rs |
| 1 (GREEN) | Full inject implementation + corrected error.rs | 81741a6 | src/input/inject.rs, src/error.rs |
| 2 | Upgrade integration test stubs | 4d0082e | tests/macro_inject.rs, tests/uinput_smoke.rs |

## What Was Built

### src/input/inject.rs

- **`VIRTUAL_KEYBOARD_KEYS`** — const array of all emittable key codes declared at compile time (required by VirtualDeviceBuilder; arrows/WASD/Fn/number-row/modifiers).
- **`MacroCmd` enum** — `Execute { keys, default_dwell_ms, default_gap_ms }` and `Shutdown` variants; sent via `std::sync::mpsc` channel.
- **`KeyStep` struct** — per-key `dwell_ms`/`gap_ms` overrides; `from_config()` converts from `config::KeyAction` (D-06).
- **`open_uinput_device()`** — `VirtualDevice::builder()` creates keyboard-only device named `"hd-linux-voice"`; maps `PermissionDenied` → `DaemonError::UinputPermissionDenied` (D-15).
- **`emit_key_action()`** — private; emits `KEY_DOWN` (value=1) → `sleep(dwell_ms)` → `KEY_UP` (value=0) → `sleep(gap_ms)`; **no SYN_REPORT** (auto-appended by evdev, Pitfall 6).
- **`spawn_injection_thread()`** — `std::thread::spawn` OS thread (D-07); blocking `recv()` loop; skips invalid key names (logged at ERROR); exits on `Shutdown` or channel disconnect.

### src/error.rs

Full `DaemonError` enum with four variants:
- `UinputPermissionDenied` — D-15 exact format: `"cannot open /dev/uinput"`, `"modprobe uinput"`, `"usermod -aG input $USER"`, `"newgrp input"`, note about systemd v258+.
- `InputGroupMissing` — for /dev/input read permission failures.
- `NoPttDevice(String)` — carries key name for evtest hint.
- `Config(String)` — config parse/IO errors.

### tests/macro_inject.rs

Three privileged integration tests (all `#[ignore]`, activated by `RUN_PRIVILEGED_TESTS=1 --include-ignored`):
- `key_sequence_fires_with_configurable_gap` — 3-key sequence takes ≥240ms (MCRO-01).
- `per_key_dwell_override_is_applied` — per-key dwell of 200ms honored over 50ms default (MCRO-02, D-06).
- `invalid_key_name_is_skipped_not_panicked` — thread continues, no panic (T-01-04-03).

### tests/uinput_smoke.rs

- `virtual_keyboard_opens_with_hd_linux_voice_name` — `#[ignore]`; smoke test for device creation (MCRO-05).
- `uinput_error_message_is_actionable` — non-privileged; validates D-15 'input' group wording.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] VirtualDeviceBuilder::new() deprecated in evdev 0.13**
- **Found during:** Task 1 GREEN (compile warning → would break in next evdev version)
- **Issue:** Plan specified `VirtualDeviceBuilder::new()` but evdev 0.13 deprecates it in favor of `VirtualDevice::builder()`
- **Fix:** Changed to `VirtualDevice::builder()` in `open_uinput_device()`; updated import from `uinput::VirtualDeviceBuilder` to `uinput::VirtualDevice`
- **Files modified:** src/input/inject.rs
- **Commit:** 81741a6

**2. [Rule 1 - Bug] Missing `mut` on device binding in privileged test**
- **Found during:** Task 1 RED compile
- **Issue:** `enumerate_dev_nodes_blocking()` requires `&mut self`; binding was not `mut`
- **Fix:** `let mut device = ...`
- **Files modified:** src/input/inject.rs
- **Commit:** e3dea33 (fixed before RED commit)

## Acceptance Criteria Results

| Criterion | Result |
|-----------|--------|
| `cargo test --lib` exits 0 | PASS (13 passed, 1 ignored) |
| `cargo test --test macro_inject` exits 0 (3 ignored) | PASS |
| `cargo test --test uinput_smoke` exits 0 (1 ignored, 1 passed) | PASS |
| DaemonError contains "cannot open /dev/uinput" | PASS |
| DaemonError contains "modprobe uinput" | PASS |
| DaemonError contains "usermod -aG input" | PASS |
| DaemonError contains "newgrp input" | PASS |
| No SYN_REPORT in emit code (only in comments) | PASS |
| No spawn_blocking in inject (only in doc comment) | PASS |
| std::thread::spawn used | PASS |
| VirtualDevice::builder() + "hd-linux-voice" name | PASS |
| `grep "usermod -aG input" src/error.rs` | PASS |

## Threat Mitigations Applied

| Threat ID | Mitigation Applied |
|-----------|--------------------|
| T-01-04-01 | `PermissionDenied` → `DaemonError::UinputPermissionDenied` with D-15 message; no privilege escalation |
| T-01-04-03 | `parse_key_code` Err → log + continue; tested by `invalid_key_name_is_skipped_not_panicked` |
| T-01-04-04 | Only `KEY_DOWN`/`KEY_UP` passed to `emit()`; no `SYN_REPORT` |
| T-01-04-05 | `std::thread::spawn` used; `std::thread::sleep` for timing; no Tokio executor involvement |

## Known Stubs

None — all plan goals fully wired. Privileged tests require live `/dev/uinput` (by design; gated by `RUN_PRIVILEGED_TESTS=1`).

## Self-Check: PASSED

- [x] `src/input/inject.rs` exists with `VirtualDeviceBuilder`, `VIRTUAL_KEYBOARD_KEYS`, `std::thread::spawn`
- [x] `src/error.rs` contains `"usermod -aG input"` (not `"uinput"`)
- [x] Commits e3dea33, 81741a6, 4d0082e exist in git log
- [x] `cargo test --lib` → 13 passed, 0 failed
