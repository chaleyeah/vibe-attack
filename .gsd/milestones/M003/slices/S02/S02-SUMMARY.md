---
id: S02
parent: M003
milestone: M003
provides:
  - ["src/ui/probe::run() -> FirstRunState", "Four check functions individually callable for per-step refresh in S03 wizard panels"]
requires:
  []
affects:
  []
key_files:
  - (none)
key_decisions:
  - ["serial_test used for env-var isolation — established pattern in this codebase", "O_NONBLOCK omitted for uinput check — read+write flags sufficient, avoids libc dep", "XDG_DATA_HOME resolved manually since xdg crate appends prefix to get_data_home() return"]
patterns_established:
  - ["probe::run() is the single FirstRunState constructor in production — wizard panels (S03) call it after each action to refresh state"]
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-26T00:15:47.398Z
blocker_discovered: false
---

# S02: Environment probe

**Real environment probe wired: src/ui/probe.rs with four hermetic-tested checks; vibe-attack-config.rs stub replaced with probe::run()**

## What Happened

Created src/ui/probe.rs with four check functions (config, model, uinput, ptt) and pub fn run() that constructs FirstRunState from real environment state. XDG path resolution uses the xdg crate. uinput check opens /dev/uinput with read+write flags. PTT check scans config lines. All checks emit tracing::warn on failure. Eight hermetic unit tests with #[serial] isolation all pass. vibe-attack-config.rs stub (from_checks(false,false,false,false)) replaced with probe::run().

## Verification

cargo test --lib ui::probe: 8/8 pass; grep confirms no stub in production code; cargo check --lib clean

## Requirements Advanced

None.

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

eframe/winit binary build fails on this headless kernel (pre-existing). Manual UAT for the GUI requires a machine with a display server.

## Follow-ups

None.

## Files Created/Modified

- `src/ui/probe.rs` — New: environment probe module with four checks and probe::run()
- `src/ui/mod.rs` — Added: pub mod probe
- `src/bin/vibe-attack-config.rs` — Replaced stub with probe::run() call
