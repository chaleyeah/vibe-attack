# S03: Tray icon state mapping + Mode submenu — UAT

**Milestone:** M008
**Written:** 2026-04-28T01:55:38.378Z

# S03: Tray icon state mapping + Mode submenu — UAT

**Milestone:** M008
**Written:** 2026-04-27

## UAT Type

- UAT mode: mixed (artifact-driven for icon mapping; live-runtime for Mode submenu and state transitions)
- Why this mode is sufficient: Icon mapping is fully covered by 5 unit tests (no D-Bus needed). Mode submenu and live state transitions require a running daemon and D-Bus session — exercised manually in this UAT. Automated menu rendering is not possible in CI (no D-Bus).

## Preconditions

1. Build the project: `cargo build --features gui`
2. A valid `config.yaml` exists (run `vibe-attack-config` once to generate if needed)
3. A D-Bus session is active (standard desktop login session)
4. No existing `vibe-attack` daemon is running: `pkill vibe-attack || true`

## Smoke Test

Run `cargo test --features gui` — all tests must pass (63 pass, 0 fail expected). This confirms icon mapping and control protocol plumbing are correct before touching live hardware.

## Test Cases

### 1. Tray icon reflects Idle state on daemon start

1. Start the daemon: `./target/debug/vibe-attack`
2. Observe the system tray icon.
3. **Expected:** Tray icon shows `audio-input-microphone` (standard microphone icon) — daemon is Idle.

### 2. Tray icon changes to Listening when audio capture begins

1. With daemon running in PTT mode, press and hold the configured PTT key.
2. Observe the tray icon while key is held.
3. **Expected:** Icon changes to `audio-input-microphone-high` (high/active microphone) during PTT hold.
4. Release PTT key.
5. **Expected:** Icon returns to `audio-input-microphone` (Idle).

### 3. Tray icon reflects Muted state

1. Right-click the tray icon and select "Mute".
2. **Expected:** Tray icon changes to `audio-input-microphone-muted`.
3. Right-click and select "Unmute".
4. **Expected:** Icon returns to `audio-input-microphone`.

### 4. Mode submenu shows current mode checkmarked

1. With daemon running in PTT mode, right-click the tray icon.
2. Hover over the "Mode" submenu.
3. **Expected:** "Push-to-talk" has a checkmark; "Wake word" does not.
4. Stop the daemon and restart it with wake-word mode configured.
5. Right-click tray → Mode submenu.
6. **Expected:** "Wake word" has a checkmark; "Push-to-talk" does not.

### 5. Selecting mode from submenu dispatches SetMode without restart

1. With daemon running in PTT mode, note the daemon PID: `pgrep vibe-attack`
2. Right-click tray → Mode → "Wake word".
3. Wait 1–2 seconds for the daemon to process.
4. **Expected:** Daemon log shows `SetMode { mode: Wake }` received. PID unchanged (no restart).
5. Right-click tray → Mode submenu.
6. **Expected:** "Wake word" is now checkmarked; "Push-to-talk" is unchecked.

### 6. Mode switch works in reverse (Wake → PTT)

1. With daemon running in wake-word mode, right-click tray → Mode → "Push-to-talk".
2. **Expected:** Daemon log shows `SetMode { mode: Ptt }`. PID unchanged.
3. Right-click tray → Mode submenu.
4. **Expected:** "Push-to-talk" is checkmarked.

## Edge Cases

### Mode submenu when daemon is not running

1. Stop the daemon: `pkill vibe-attack`
2. Right-click the tray icon → Mode submenu.
3. **Expected:** Both "Push-to-talk" and "Wake word" are greyed out (disabled) and neither is checkmarked.

### Rapid mode switching

1. With daemon running, right-click → Mode → "Wake word", then immediately right-click → Mode → "Push-to-talk".
2. **Expected:** No crash, no panic. Daemon ends up in PTT mode. Tray eventually reflects PTT.

## Failure Signals

- Tray icon does not change with daemon state → icon_name_for_state mapping broken or poll loop not updating TrayState
- Mode submenu items are missing from right-click menu → T03 changes not compiled or wrong feature flag
- Mode submenu items are always greyed out even when daemon is running → daemon_running logic or active_mode propagation broken
- Selecting a mode causes daemon to restart (PID changes) → SetMode handler incorrectly restarting pipeline
- Checkmark does not update after switching mode → active_mode not being written back to TrayState in poll loop

## Not Proven By This UAT

- Recording state icon (requires a phrase match to fire during a live session)
- Mode switch triggering a live stratagem recognition in the new mode (covered by S04 end-to-end UAT)
- Behavior under concurrent rapid SetMode calls from config window and tray simultaneously

## Notes for Tester

The icon names (`audio-input-microphone`, `audio-input-microphone-high`, `audio-input-microphone-muted`) are freedesktop.org icon theme names — their visual appearance depends on the desktop theme installed. On systems without those icon names the tray may fall back to a generic icon; this is a theme gap, not a code bug. The Mode submenu separator appears between Profiles and Quit in the context menu.
