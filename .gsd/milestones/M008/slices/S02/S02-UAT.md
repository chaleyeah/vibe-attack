# S02: ConfigApp state + egui config panel — UAT

**Milestone:** M008
**Written:** 2026-04-28T01:42:55.664Z

# S02 UAT: ConfigApp state + egui config panel

## Preconditions
- Build: `cargo build --features gui --bin vibe-attack-config` succeeds
- Config file at `~/.config/vibe-attack/config.yaml` exists (or will be created by wizard)
- vibe-attack daemon may be running or absent (both paths must be tested)

## Test Cases

### TC-01: Panel renders without crash (daemon absent)
1. Kill any running vibe-attack daemon
2. Run `cargo run --features gui --bin vibe-attack-config`
3. Complete (or skip) the first-run wizard
**Expected**: Main config panel appears. Status row shows amber "Daemon: not running (changes will save to disk only)". No panic, no blank window.

### TC-02: Threshold slider is readable and writable
1. With the config panel open, observe the "Confidence threshold (%)" slider
**Expected**: Slider shows a value between 0–100. Moving the slider updates the displayed integer value immediately.

### TC-03: Mode radio buttons reflect current state
1. Observe the "Mode:" row in the config panel
**Expected**: "Push-to-talk" and "Wake word" radio buttons present. Exactly one is selected. Selection changes when clicked.

### TC-04: Input device ComboBox lists system devices
1. Click the "Input device" ComboBox
**Expected**: Dropdown contains "<system default>" as the first option, followed by any detected ALSA/PulseAudio input devices. Selecting one updates the displayed selection.

### TC-05: Save with daemon absent — writes to disk, no panic
1. Ensure daemon is not running
2. Change the threshold slider to a new value
3. Click "Save"
**Expected**: Status message shows "Saved to disk — daemon not running, runtime changes skipped." Config file at `~/.config/vibe-attack/config.yaml` is updated (verify `stt.confidence_threshold` field). No crash.

### TC-06: Save with daemon running — dispatches control commands
1. Start vibe-attack daemon
2. Change mode to "Wake word"
3. Move threshold slider to 60%
4. Click "Save"
**Expected**: Status shows "Saved and applied." Daemon log shows `runtime_command_applied SetMode` and `dispatcher threshold updated` events (via the log scroll area or daemon stderr). Config YAML updated on disk.

### TC-07: Save failure on SetMode — status shows error, no panic
1. Start daemon then immediately kill it between clicking Save (simulate transient absence)
**Expected**: Status message contains the error text from `send_command` (e.g. "Failed to send SetMode: ..."). No panic.

### TC-08: Config round-trip preserves macros and other fields
1. Have a config.yaml with at least 2 macros defined
2. Open config panel, change threshold, click Save
3. Inspect saved config.yaml
**Expected**: Both macros are still present. Only `stt.confidence_threshold`, `audio.device`, `ptt.key` changed (if those were edited).

### TC-09: PTT key display is read-only
1. Observe the "PTT key:" label in the config panel
**Expected**: PTT key value is displayed as text (not an editable input). No crash when inspecting.

### TC-10: Daemon status indicator updates across frames
1. Open config panel with daemon absent (amber status)
2. Start the daemon in another terminal
3. Wait ~2 seconds
**Expected**: Status row updates to green "Daemon: running" on the next frame without restarting the config app.

## Edge Cases
- **No config.yaml on disk**: Panel shows status "Could not load config.yaml: ..." on startup; Save button is disabled (shows "No config loaded — cannot save.")
- **Threshold at boundary values (0% and 100%)**: Slider can be moved to extremes; Save writes 0.0 and 1.0 to `stt.confidence_threshold` respectively
- **device_names empty** (no input devices detected): ComboBox shows only "<system default>"; no panic

