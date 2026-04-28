# S04 UAT: Tray + Config Window + Daemon Control Surface — End-to-End Mode Switch

Manual test script for M008 Slice S04. Verifies that mode and threshold changes reach a running
daemon over the control socket without a daemon restart, and that the tray reflects those changes
in real time.

---

## Preconditions

- Linux desktop session with a system tray (KDE Plasma, GNOME + AppIndicator/Status Area extension,
  XFCE, i3 with a status-bar tray slot, etc.) and a working microphone.
- `cargo build --features gui` and `cargo build` both succeed with no errors.
- Microphone accessible to the user — no PipeWire or ALSA permission errors in
  `journalctl --user -b | grep -i audio`.
- At least one profile directory exists under `~/.config/vibe-attack/profiles/<name>/pack.yaml`
  (the HD2 pack is the reference; any pack with stratagem phrases works).
- Shell environment has `RUST_LOG=info` (or `RUST_LOG=debug` for maximum detail) set before
  launching the daemon so that `SetMode` and `runtime_command_applied` log lines are visible.

---

## Setup

1. Open **terminal A**. Export the log level and start the daemon, tee-ing output to a log file:

   ```
   export RUST_LOG=info
   cargo run --bin vibe-attack 2>&1 | tee /tmp/vibe-attack-uat.log
   ```

2. Wait for the line:

   ```
   Control channel listening on: /run/user/<UID>/vibe-attack/vibe-attack.sock
   ```

   Record the socket path — you may need it for manual `socat` inspection in later steps.

3. Open **terminal B**. Start the config window:

   ```
   cargo run --features gui --bin vibe-attack-config
   ```

4. Confirm the tray icon appears in the system tray — it should show the
   `audio-input-microphone` glyph (microphone icon, not muted).

5. Right-click the tray icon and confirm the context menu shows:
   - **Open Config**
   - **Mute**
   - **Profiles** submenu (lists available profiles)
   - **Mode** submenu (Push-to-talk / Wake word)
   - **Quit**

---

## Test 1: PTT → Wake mode switch via config window

1. In the config window, locate the **Mode** dropdown or radio buttons. It should show
   `Push-to-talk` as the current selection.
2. Change Mode to **Wake word**.
3. Click **Save**.
4. **Expected in terminal A** (within 1 s):

   ```
   SetMode: cached active_mode=Wake, forwarding to coordinator
   ```

   If `RUST_LOG=debug`, you will also see the coordinator acknowledge the `RuntimeCommand::SetMode`.
5. **Expected in tray** (within the 1 s poll interval): right-click → Mode → `Wake word` now has
   a checkmark; `Push-to-talk` checkmark is gone.
6. **Expected: no restart log lines** — `Pipeline shutting down`, `Spawning audio thread`, or
   `Control channel listening on:` must NOT appear after the SetMode log.

---

## Test 2: Speak a stratagem phrase in Wake mode

1. With Wake mode active (confirmed by Test 1), speak the wake word followed by a stratagem
   phrase from the active profile (e.g. "reinforce" or any phrase in `pack.yaml`).

   > **Note:** The default wake word is project-specific and may not yet be wired to a hot
   > classifier in M008. If wake-word detection is not yet functional end-to-end, use
   > Push-to-talk for phrase dispatch and note this in the UAT results.

2. **Expected in terminal A:**

   ```
   Dispatched macro: <name>
   ```

   If `RUST_LOG=debug`, you will also see the raw STT transcription and confidence score before
   the dispatch line.

3. **Expected behavior:** the configured key sequence is sent. Confirm by focusing a text editor
   or an `xev` window and verifying that the expected key events arrive.

---

## Test 3: Threshold change without restart

1. In the config window, adjust the **Threshold** slider (or numeric input) to a different value
   (e.g. move from 80% to 40%).
2. Click **Save**.
3. **Expected in terminal A** (within 1 s): a log line containing `SetThreshold` — for example,
   a debug line from the coordinator acknowledging `RuntimeCommand::SetThreshold(0.4)`.
4. **Expected: no restart messages.** `Pipeline shutting down` must NOT appear.
5. **Behavioral check:** speak a slightly mispronounced phrase that previously did not fire. With
   a lower threshold it should now dispatch; with a higher threshold a marginal phrase should fail.
   Note the result in the UAT log.

---

## Test 4: Tray Mode submenu round-trip

1. Right-click the tray icon → **Mode** → **Push-to-talk**.
2. **Expected in terminal A** (within 1 s):

   ```
   SetMode: cached active_mode=Ptt, forwarding to coordinator
   ```

3. **Expected in tray** (within 1 s poll cycle): `Push-to-talk` checkmark is active;
   `Wake word` checkmark is gone.
4. Config window does NOT need to be open for this test.
5. **Expected: no daemon restart.** `Control channel listening on:` must NOT reappear.

---

## Test 5: Tray icon state transitions

Observe the tray icon glyph as the daemon transitions between states.

| Daemon state            | Expected icon name                      | When to observe                                |
|-------------------------|-----------------------------------------|------------------------------------------------|
| Idle (Wake mode)        | `audio-input-microphone`                | Daemon running, no active session              |
| Listening (wake window) | `audio-input-microphone-high`           | Immediately after wake word is detected        |
| Recording (PTT held)    | `audio-input-microphone`                | While PTT key is held down                     |
| Muted                   | `audio-input-microphone-muted`          | After right-click → Mute                       |
| Daemon stopped          | `audio-input-microphone-muted`          | After killing the daemon process               |

To observe the Muted transition: right-click → **Mute**; the icon should change to the muted
glyph within 1 s. Right-click → **Unmute** to restore.

To observe the Listening state: trigger the wake word and watch for a brief icon change during
the listen window. If the window is too short to see visually, enable `RUST_LOG=debug` and grep
the log for `state=Listening`.

---

## Pass/fail Criteria

Record a result for each item:

- [ ] Mode switch (config window Save) triggers `SetMode: cached active_mode=Wake` log without
      any restart messages.
- [ ] Tray Mode submenu reflects current mode (checkmark on correct item) after mode switch,
      within the 1 s poll cycle.
- [ ] Tray Mode submenu dispatches SetMode over the socket (`SetMode:` log line) without restart.
- [ ] Threshold change (config window Save) triggers a `SetThreshold` log without any restart
      messages.
- [ ] A stratagem phrase fires (`Dispatched macro:` log) after a mode switch with no daemon
      restart in between.
- [ ] Tray icon visually changes between idle / listening / recording / muted as the daemon
      transitions (observable for at least the Idle → Muted → Unmuted cycle).

---

## Known UAT Limitations

- **Recording-state icon**: only visible while a PTT key is physically held down — not observable
  in passive dashboard-style checks. Enable `RUST_LOG=debug` and grep for `state=Recording` if
  you cannot hold the key and watch the tray simultaneously.
- **Wake-word listen window**: the window may be very brief (under 500 ms). Use `RUST_LOG=debug`
  and `grep 'state=Listening' /tmp/vibe-attack-uat.log` to confirm it fired if the visual icon
  change is not caught in time.
- **Mode persistence**: ActivationMode is runtime-only in M008 and is not saved to
  `~/.config/vibe-attack/config.yaml`. After a daemon restart, mode resets to `Push-to-talk`
  (the compiled default). This is expected behavior for this milestone.
- **Tray icon rendering quality**: depends on the distro's StatusNotifierItem / SNI implementation.
  KDE/Plasma is the reference target. GNOME requires the AppIndicator/gnome-shell-extension-appindicator
  extension; without it the tray icon will not appear at all.
- **SetInputDevice / SetPttBinding**: these commands require a daemon restart to take effect
  (S01 design decision); the log will show `requires daemon restart` — this is expected and is
  not a failure of this UAT.
