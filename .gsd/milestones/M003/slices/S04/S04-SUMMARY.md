---
id: S04
parent: M003
milestone: M003
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - (none)
key_decisions:
  - ["CPAL stream kept alive on parked thread per audio/mod.rs architecture note — moving Stream out crashes ALSA silently on Linux", "Profiles and mic deferred to post-wizard to avoid unnecessary resource acquisition during setup"]
patterns_established:
  - (none)
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-26T00:22:32.227Z
blocker_discovered: false
---

# S04: Main config app wired up

**Main config app populated from real data: profiles from XDG, live CPAL mic level with graceful no-device fallback, log lines via tracing channel layer with auto-scroll**

## What Happened

load_profiles() reads *.yaml stems from XDG profiles dir. spawn_mic_level_thread() opens the default CPAL device and computes RMS into an AtomicU32 (or returns no_device=true gracefully). ChannelLayer tracing subscriber feeds log lines to an mpsc channel drained each egui frame. ScrollArea stick_to_bottom auto-scrolls on new lines. Mic and profiles are deferred until wizard completes. ctx.request_repaint_after(100ms) keeps mic level live at ~10Hz.

## Verification

cargo check --lib clean; 8 probe unit tests pass; no errors from binary source files

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

Manual UAT requires display server; eframe/winit binary build fails on headless kernel (pre-existing). CPAL mic level accuracy depends on device buffer sizes — at 44.1kHz with typical 1024-sample buffers, updates are ~23ms per RMS reading.

## Follow-ups

None.

## Files Created/Modified

- `src/ui/config_app.rs` — Added load_profiles(), mic_no_device field, tracing import
- `src/bin/vibe-attack-config.rs` — Added ChannelLayer, MicLevelState, spawn_mic_level_thread(), show_main_config(), log/mic wiring in update()
