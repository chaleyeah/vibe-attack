# S03: Live Daemon State in Tray

**Goal:** Live daemon state reflected in tray icon/tooltip with mute/unmute toggle
**Demo:** With daemon running: tray icon turns green when wake-word or PTT is active, red when muted via tray click. With daemon stopped: tray shows grey 'Not running' tooltip.

## Must-Haves

- Not provided.

## Proof Level

- This slice proves: Not provided.

## Integration Closure

Not provided.

## Verification

- Not provided.

## Tasks

- [x] **T01: Poll loop and live tray state** `est:2h`
  Add TrayState behind tokio Mutex; 1s poll loop queries DaemonHandle and updates icon/tooltip; Mute/Unmute item conditional on daemon running
  - Files: `src/ui/tray.rs`
  - Verify: Mute/unmute toggle verified; tray updates within ~1s of daemon state change

## Files Likely Touched

- src/ui/tray.rs
