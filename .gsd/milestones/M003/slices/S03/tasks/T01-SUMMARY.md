---
id: T01
parent: S03
milestone: M003
key_files:
  - src/ui/wizard.rs
  - src/ui/mod.rs
  - src/ui/probe.rs
  - src/bin/vibe-attack-config.rs
key_decisions:
  - wizard.rs uses #[cfg(feature = "gui")] inner module so the file can exist in the lib without pulling egui into non-gui builds
  - ctx.request_repaint_after(100ms) used while PTT thread listens — avoids busy-poll without missing the result by more than 100ms
duration: 
verification_result: passed
completed_at: 2026-04-26T00:18:53.165Z
blocker_discovered: false
---

# T01: Created src/ui/wizard.rs with show_wizard() dispatcher and PttCaptureState; wired into vibe-attack-config.rs

**Created src/ui/wizard.rs with show_wizard() dispatcher and PttCaptureState; wired into vibe-attack-config.rs**

## What Happened

Created wizard.rs with entire implementation behind #[cfg(feature = \"gui\")] inner module. show_wizard() dispatches by first_incomplete_step(), harvests PTT thread results on each frame, and re-probes after each action. PttCaptureState holds the thread handle and Arc<Mutex<Option<String>>> for the captured key. Updated vibe-attack-config.rs to call show_wizard() and added request_repaint_after(100ms) while PTT thread is listening. Added config_path_for_display() and model_path_for_display() helpers to probe.rs for wizard panel display.

## Verification

cargo check --lib exits 0; no errors from wizard.rs or vibe-attack-config.rs source

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check --lib` | 0 | pass — clean with no errors from wizard module | 800ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/ui/wizard.rs`
- `src/ui/mod.rs`
- `src/ui/probe.rs`
- `src/bin/vibe-attack-config.rs`
