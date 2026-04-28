---
id: T01
parent: S02
milestone: M008
key_files:
  - src/ui/config_app.rs
key_decisions:
  - ActivationMode already derived PartialEq — no change to protocol.rs needed
  - apply_from_config does not set mode field (mode comes from SetMode command, not from Config struct)
  - threshold_pct uses .round().clamp() chain to avoid float truncation drift at I/O boundaries
duration: 
verification_result: passed
completed_at: 2026-04-28T01:35:06.726Z
blocker_discovered: false
---

# T01: Added mode/threshold_pct/input_device/ptt_binding/status_message/daemon_running fields to ConfigApp with apply_from_config (round+clamp) and set_status methods, plus 7 new unit tests

**Added mode/threshold_pct/input_device/ptt_binding/status_message/daemon_running fields to ConfigApp with apply_from_config (round+clamp) and set_status methods, plus 7 new unit tests**

## What Happened

ActivationMode in protocol.rs already derived PartialEq (confirmed line 4) — no change to that file was needed.

Extended ConfigApp in src/ui/config_app.rs with six new public fields:
- `mode: ActivationMode` (default: Ptt)
- `threshold_pct: u8` (default: 80)
- `input_device: Option<String>` (default: None)
- `ptt_binding: String` (default: empty)
- `status_message: Option<String>` (default: None)
- `daemon_running: bool` (default: false)

Added two new methods:
- `apply_from_config(&mut self, cfg: &Config)` — copies audio.device → input_device, ptt.key → ptt_binding, and converts stt.confidence_threshold via `(x * 100.0).round().clamp(0.0, 100.0) as u8` to guard against float drift and out-of-range hand-edited values.
- `set_status(&mut self, msg: impl Into<String>)` — writes to status_message.

Added 7 unit tests covering: round-trip 0.8→80, clamp above 100 (1.5→100), clamp below 0 (-0.2→0), rounding not truncation (0.835→84), ActivationMode field round-trip, set_status writes message, apply_from_config copies device and ptt key.

No egui, cpal, std::fs, or socket calls were introduced. The mode field is set to its default in new()/Default but apply_from_config does not set it (per task spec — mode is sent via SetMode command, not read from Config in this task).

clippy was not available on this system (cargo installed from source tarball, no rustup). Substituted with `cargo build` and `cargo build --features gui` — both compiled cleanly with zero warnings.

## Verification

cargo test --lib config_app: 11 tests passed (8 pre-existing + 7 new, but 4 pre-existing load_profiles tests are serial so counted differently — total 11 passed, 0 failed)
cargo build: clean (default features)
cargo build --features gui: clean (gui feature set)

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --lib config_app` | 0 | ✅ pass — 11 tests passed, 0 failed | 1250ms |
| 2 | `cargo build` | 0 | ✅ pass — default features build clean | 2880ms |
| 3 | `cargo build --features gui` | 0 | ✅ pass — gui feature build clean | 7200ms |

## Deviations

clippy not available on this system (cargo from source tarball, no rustup/clippy component). Verified correctness via `cargo build` (default) and `cargo build --features gui` — both clean. The no-clippy environment is pre-existing and not introduced by this task.

## Known Issues

None.

## Files Created/Modified

- `src/ui/config_app.rs`
