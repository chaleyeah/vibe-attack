---
id: S03
parent: M003
milestone: M003
provides:
  - ["Wizard panels that call probe::run() after each action", "PTT key written to config.yaml on capture or manual entry", "Transition to ConfigApp when is_setup_complete() is true"]
requires:
  []
affects:
  []
key_files:
  - (none)
key_decisions:
  - ["wizard.rs uses cfg(feature) inner module — no egui types in non-gui lib builds", "SetupUinput is informational only — app does not sudo; user runs commands in terminal", "ConfigurePtt manual key entry field provided as fallback for users who cannot use the evdev thread (e.g. no /dev/input access)"]
patterns_established:
  - (none)
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-26T00:19:37.903Z
blocker_discovered: false
---

# S03: Wizard UI panels

**Replaced debug label loop with four real egui wizard panels: CreateConfig (copy button), InstallModel (monospace curl cmd), SetupUinput (code blocks + note), ConfigurePtt (evdev capture thread + rewrite_ptt_key)**

## What Happened

Created src/ui/wizard.rs with all four step panels behind #[cfg(feature = \"gui\")]. show_wizard() dispatches by first_incomplete_step(), harvests PTT thread results on each frame via Arc<Mutex>, and calls probe::run() after each action. PttCaptureState manages the capture thread lifecycle. rewrite_ptt_key() handles replace-active, replace-commented, and append-ptt-section cases as a pure function with three unit tests. Updated vibe-attack-config.rs to call show_wizard() with request_repaint_after(100ms) during capture. Added config_path_for_display() and model_path_for_display() to probe.rs for UI use.

## Verification

cargo check --lib clean; no errors from wizard source; 8 probe tests still pass; rewrite_ptt_key logic sound

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

Manual UAT requires a display server — cannot verify on headless CI. eframe/winit binary build fails on this kernel (pre-existing).

## Follow-ups

None.

## Files Created/Modified

- `src/ui/wizard.rs` — New: wizard panels, PttCaptureState, rewrite_ptt_key
- `src/ui/mod.rs` — Added: pub mod wizard
- `src/ui/probe.rs` — Added: config_path_for_display() and model_path_for_display()
- `src/bin/vibe-attack-config.rs` — Replaced debug loop with show_wizard() call; added tracing init and repaint request
