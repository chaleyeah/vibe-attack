---
estimated_steps: 51
estimated_files: 6
skills_used: []
---

# T02: Author S04-UAT.md manual test script for tray/config end-to-end mode switch

Write .gsd/milestones/M008/slices/S04/S04-UAT.md as a freestanding manual test script. The doc walks a human through verifying that M008's tray + config window + daemon control surface works end-to-end on a real Linux desktop session, with no daemon restart required for mode/threshold changes.

Structure the doc as a runnable checklist with these sections:

1. **Preconditions** — bullet list:
   - Linux desktop session with system tray (KDE Plasma, GNOME with extension, XFCE, etc.) and a working microphone.
   - `cargo build --features gui` and `cargo build` both succeed.
   - Microphone accessible to the user (no PipeWire permission errors in journalctl).
   - HD2 pack profile loaded (or any profile with at least one stratagem phrase).
   - `RUST_LOG=info` or `RUST_LOG=debug` set in the shell that launches the daemon, so SetMode and runtime_command_applied log lines are visible.

2. **Setup** — numbered steps with shell commands:
   - `cargo run --bin vibe-attack 2>&1 | tee /tmp/vibe-attack-uat.log` in terminal A.
   - Wait for `Control channel listening on:` log line — record the socket path.
   - In terminal B: `cargo run --features gui --bin vibe-attack-config` to open the config window.
   - Confirm the tray icon appears in the system tray (audio-input-microphone glyph).

3. **Test 1: PTT → Wake mode switch via config window** — numbered steps + expected:
   - In the config window, change Mode dropdown from 'Push-to-talk' to 'Wake word'.
   - Click Save.
   - **Expected in terminal A:** within 1s, a log line `SetMode: cached active_mode=Wake` and `runtime_command_applied cmd=set_mode mode=Wake` (or equivalent — exact wording from the daemon).
   - **Expected in tray:** Mode submenu now shows 'Wake word' checkmarked.
   - **Expected in daemon:** no restart log lines (no 'Pipeline shutting down' or 'Spawning audio thread' messages after the SetMode).

4. **Test 2: Speak a stratagem phrase** — numbered steps:
   - With Wake mode active, speak the wake word (whatever the project default is — note as TBD if not yet wired) followed by a stratagem phrase from the active profile.
   - **Expected in terminal A:** STT transcription log + macro fired log (`Dispatched macro: <name>`).
   - **Expected behavior:** the configured key sequence is sent (verify with a focused text editor or `xev` window if not in-game).

5. **Test 3: Threshold change without restart** — numbered steps:
   - In the config window, drag threshold slider to a different value (e.g. 0.4 if it was 0.5).
   - Click Save.
   - **Expected in terminal A:** SetThreshold log line; no restart messages.
   - **Expected behavior:** subsequent phrase matches use the new threshold (test by speaking a slightly mispronounced phrase that previously did not fire).

6. **Test 4: Tray Mode submenu round-trip** — numbered steps:
   - Right-click tray icon → Mode → 'Push-to-talk'.
   - **Expected in terminal A:** SetMode log line with mode=Ptt.
   - **Expected in tray:** checkmark moves to 'Push-to-talk'; config window does NOT need to be open.
   - **Expected: no daemon restart.**

7. **Test 5: Tray icon state transitions** — bulleted observation list:
   - At rest: audio-input-microphone (idle).
   - During wake-word listen window: audio-input-microphone-high.
   - During PTT recording: audio-input-microphone.
   - When muted (right-click → Mute): audio-input-microphone-muted.
   - Daemon stopped: audio-input-microphone-muted (TrayState.daemon_running=false).

8. **Pass/fail criteria** — explicit checklist:
   - [ ] Mode switch in config triggers SetMode log without restart.
   - [ ] Tray Mode submenu reflects current mode and dispatches SetMode without restart.
   - [ ] Threshold change in config triggers SetThreshold log without restart.
   - [ ] Stratagem phrase fires after a mode switch with no daemon restart in between.
   - [ ] Tray icon visually changes between idle / listening / recording / muted as the daemon transitions.

9. **Known UAT limitations** — short bullet list:
   - Recording-state icon is only visible during a live PTT hold — not observable in dashboard-style passive checks.
   - Wake-word listen window may be very short; capture by enabling RUST_LOG=debug and grepping the log.
   - Tray icon rendering quality depends on the distro's StatusNotifierItem implementation; KDE/Plasma is the reference target.

Keep the doc terse and verifiable — every assertion should be a log line, a visible UI element, or a key event the tester can confirm with their own eyes. Do not include any code that needs to be compiled — this file is consumed by humans only.

Write the file directly with the Write tool. The .gsd/ directory is gitignored, so no commit is needed.

## Inputs

- ``.gsd/milestones/M008/M008-ROADMAP.md``
- ``.gsd/milestones/M008/slices/S04/S04-RESEARCH.md``
- ``.gsd/milestones/M008/slices/S03/S03-SUMMARY.md``
- ``src/ui/tray.rs``
- ``src/ui/config_app.rs``
- ``src/control/mod.rs``

## Expected Output

- ``.gsd/milestones/M008/slices/S04/S04-UAT.md``

## Verification

test -f .gsd/milestones/M008/slices/S04/S04-UAT.md && [ $(wc -l < .gsd/milestones/M008/slices/S04/S04-UAT.md) -gt 30 ] && grep -q 'Pass/fail' .gsd/milestones/M008/slices/S04/S04-UAT.md && grep -q 'Preconditions' .gsd/milestones/M008/slices/S04/S04-UAT.md && grep -q 'SetMode' .gsd/milestones/M008/slices/S04/S04-UAT.md
