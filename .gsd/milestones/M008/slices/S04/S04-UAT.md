# S04: End-to-end UAT + headless integration test — UAT

**Milestone:** M008
**Written:** 2026-04-28T02:07:05.190Z

# S04 UAT: M008 End-to-End Control Surface

## Preconditions

- Linux desktop session with system tray (KDE Plasma, GNOME with AppIndicator/SNI extension, XFCE, or equivalent) and a working microphone.
- `cargo build --features gui` and `cargo build` both succeed without errors.
- Microphone accessible to the user (no PipeWire/ALSA permission errors in `journalctl -f`).
- HD2 pack profile loaded (or any profile with at least one stratagem phrase).
- `RUST_LOG=info` set in the shell that launches the daemon so SetMode and threshold log lines are visible.

## Setup

1. In **terminal A**, run:
   ```
   RUST_LOG=info cargo run --bin vibe-attack 2>&1 | tee /tmp/vibe-attack-uat.log
   ```
2. Wait for the log line: `Control channel listening on: /run/user/<uid>/vibe-attack/vibe-attack.sock` — note the socket path.
3. In **terminal B**, run:
   ```
   cargo run --features gui --bin vibe-attack-config
   ```
4. Confirm the tray icon (audio-input-microphone glyph) appears in the system tray.

## Test 1: PTT → Wake mode switch via config window

1. In the config window, change the **Mode** dropdown from `Push-to-talk` to `Wake word`.
2. Click **Save**.

**Expected in terminal A (within 1s):**
- Log line containing `SetMode: cached active_mode=Wake`
- Log line containing `runtime_command_applied cmd=set_mode mode=Wake`

**Expected in tray:** Right-click → Mode → `Wake word` is now checkmarked.

**Expected: no restart.** No `Pipeline shutting down` or `Spawning audio thread` log lines should appear after the SetMode.

- [ ] PASS / FAIL

## Test 2: Speak a stratagem phrase after mode switch

1. With Wake mode active, speak the configured wake word (check `config.yaml` for the phrase — e.g. `hey vibe`) followed by a stratagem phrase from the active profile.

**Expected in terminal A:**
- STT transcription log line (e.g. `transcript: "eagle storm"`)
- Macro dispatch log line: `Dispatched macro: <name>`

**Expected behavior:** The configured key sequence fires. Verify with a focused text editor or an `xev` window if not in-game.

- [ ] PASS / FAIL

## Test 3: Threshold change without restart

1. In the config window, drag the **Threshold** slider to a different value (e.g. 0.4 if it was 0.5).
2. Click **Save**.

**Expected in terminal A:**
- Log line containing `SetThreshold` with the new value (e.g. `runtime_command_applied cmd=set_threshold threshold=0.4`)
- No restart log lines.

**Expected behavior:** Subsequent phrase matches use the new threshold. Test by speaking a slightly mispronounced phrase that previously did not fire.

- [ ] PASS / FAIL

## Test 4: Tray Mode submenu round-trip

1. Right-click the tray icon → **Mode** → `Push-to-talk`.

**Expected in terminal A:**
- Log line `SetMode: cached active_mode=Ptt` (or equivalent)
- No daemon restart messages.

**Expected in tray:** Checkmark moves to `Push-to-talk`. Config window does NOT need to be open for this to work.

- [ ] PASS / FAIL

## Test 5: Tray icon state transitions

Observe the tray icon while the daemon is running and transitioning:

| State | Expected icon |
|-------|--------------|
| At rest (idle) | `audio-input-microphone` |
| During wake-word listen window | `audio-input-microphone-high` |
| During PTT recording | `audio-input-microphone` (active) |
| When muted (right-click → Mute) | `audio-input-microphone-muted` |
| Daemon stopped / socket absent | `audio-input-microphone-muted` |

- [ ] PASS / FAIL

## Pass/fail criteria

- [ ] Mode switch in config triggers SetMode log without daemon restart.
- [ ] Tray Mode submenu reflects current mode and dispatches SetMode without restart.
- [ ] Threshold change in config triggers SetThreshold log without restart.
- [ ] Stratagem phrase fires after a mode switch with no daemon restart in between.
- [ ] Tray icon visually changes between idle / listening / recording / muted as the daemon transitions.

## Known UAT limitations

- **Recording-state icon** is only visible during a live PTT hold — not observable in passive checks; use RUST_LOG=debug and grep for `state=Recording` in the log.
- **Wake-word listen window** may be very short; use RUST_LOG=debug and grep for `Listening` state transitions to confirm it fired.
- **Tray icon rendering** depends on the distro's StatusNotifierItem implementation; KDE/Plasma is the reference target — GNOME may require the AppIndicator extension.
- **ActivationMode is runtime-only in M008** — it is not persisted to `config.yaml`. After a daemon restart, the mode reverts to whatever `config.yaml` specifies. Mode persistence (write-back) is deferred to a future milestone.
