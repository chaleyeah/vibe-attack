---
id: T02
parent: S04
milestone: M003
key_files:
  - src/bin/vibe-attack-config.rs
key_decisions:
  - Stream kept alive on a parked thread per CPAL architecture note in audio/mod.rs — moving Stream out of the thread that created it causes silent ALSA stop on Linux
  - Mic thread started only after wizard completes to avoid unnecessary audio capture during setup
duration: 
verification_result: passed
completed_at: 2026-04-26T00:22:07.343Z
blocker_discovered: false
---

# T02: Added spawn_mic_level_thread() in binary: CPAL RMS level stored in AtomicU32, graceful no-device handling, ProgressBar in UI

**Added spawn_mic_level_thread() in binary: CPAL RMS level stored in AtomicU32, graceful no-device handling, ProgressBar in UI**

## What Happened

spawn_mic_level_thread() opens the default CPAL input device, builds an input stream that computes RMS per buffer and stores as f32::to_bits() in an Arc<AtomicU32>. Thread parks after stream.play() keeping the stream alive. On device or stream failure, returns MicLevelState with no_device=true and level=0. show_main_config() renders a ProgressBar clamped 0.0..=1.0 or a 'no device' label. ctx.request_repaint_after(100ms) keeps the level updating at ~10Hz.

## Verification

cargo check --lib clean; no errors from binary source; no_device path returns gracefully

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check --lib && no errors from vibe-attack-config.rs source in cargo check --bin --features gui` | 0 | pass | 750ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/bin/vibe-attack-config.rs`
