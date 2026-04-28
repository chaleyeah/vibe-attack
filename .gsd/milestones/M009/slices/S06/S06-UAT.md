# S06: UAT — full pack lifecycle and editor flow — UAT

**Milestone:** M009
**Written:** 2026-04-28T03:33:09.400Z

# S06: UAT — full pack lifecycle and editor flow

**Milestone:** M009
**Written:** 2026-04-27

## UAT Type

- UAT mode: mixed (artifact-driven for automated suites + human-experience for GUI scenarios)
- Why this mode is sufficient: Five manual scenarios cover GUI interactions that cannot be exercised headlessly (editor CRUD, file picker, countdown UI, malformed import); automated cargo test suites provide hermetic coverage of the underlying pack logic and control-plane routing.

## Preconditions

| Prerequisite | How to confirm |
|---|---|
| `cargo build --features gui` exits 0 | Run command; check exit code |
| Daemon binary `vibe-attack` built and launchable | `target/debug/vibe-attack --help` exits 0 |
| Config binary `vibe-attack-config` built | `target/debug/vibe-attack-config --help` exits 0 |
| HD2 profile exists | `test -f profiles/hd2/pack.yaml` |
| Hardware desktop session (X11 or Wayland) | Running inside a display session, not SSH-only |
| `/tmp` writable | `touch /tmp/hd2-uat.hdpack && rm /tmp/hd2-uat.hdpack` |

## Smoke Test

Run `cargo test -- --test-threads=1` from repo root. All `test result:` lines must show `0 failed`. This confirms the automated pack lifecycle and control integration layers are intact before any manual scenario.

## Test Cases

### Scenario 1: HD2 voice stratagem fire

**Prerequisites:** Daemon running with HD2 profile active; audio input device connected.

1. Launch `target/debug/vibe-attack` with HD2 profile selected.
2. Confirm daemon stdout shows `Listening` state in JSONL output.
3. Speak the phrase **"railgun"** clearly into the microphone.
4. **Expected:** Daemon JSONL stdout shows a `DispatchOutcome::Fired` event for the railgun macro (Patriotic Administration Center category). Key sequence is injected via uinput.
5. Repeat for one phrase from each remaining category: **"orbital airburst strike"** (Orbital Cannons), **"eagle strafing run"** (Hangar), **"resupply"** (Bridge), **"guard dog"** (Engineering Bay), **"gatling sentry"** (Robotics Workshop).
6. **Expected:** Each spoken phrase produces exactly one `Fired` event with the correct macro name; no `Unmatched` events for these phrases.

### Scenario 2: Add macro and fire via editor

**Prerequisites:** Config window launchable; daemon running with any writable profile.

1. Open `target/debug/vibe-attack-config`.
2. Navigate to the Pack Editor panel.
3. Add a new macro: phrase **"extra ammo"**, key sequence **`KEY_DOWN,KEY_DOWN,KEY_UP,KEY_RIGHT`**, assigned to any existing category.
4. Click **Save**.
5. Confirm `pack.yaml` on disk contains the new macro entry (open the file or reload the editor).
6. Speak **"extra ammo"** into the microphone.
7. **Expected:** Daemon JSONL stdout shows a `DispatchOutcome::Fired` event for the "extra ammo" macro. Key sequence `KEY_DOWN,KEY_DOWN,KEY_UP,KEY_RIGHT` is injected via uinput.

### Scenario 3: Test button 1-second countdown

**Prerequisites:** Config window open; daemon running; macro selected in editor.

1. Open the Pack Editor and select any macro.
2. Click the **Test** button.
3. **Expected:** A countdown label appears and decrements visibly from 1 to 0 (approximately 1 second).
4. While the countdown is active, click **Cancel**.
5. **Expected:** Countdown stops immediately; no `MacroCmd::Execute` appears in daemon JSONL; no key sequence injected.
6. Repeat steps 2–3 without clicking Cancel; let the countdown complete.
7. **Expected:** Daemon JSONL shows `DispatchOutcome::Fired` with `score: 1.0` (control-plane convention). Key sequence injected via uinput.

### Scenario 4: GUI export/import round-trip

**Prerequisites:** Config window open with HD2 profile loaded; `/tmp` writable.

1. Open the Pack Editor with the HD2 profile active.
2. Note the number of categories and total macros displayed.
3. Click **Export Pack** and save to `/tmp/hd2-uat.hdpack`.
4. **Expected:** `/tmp/hd2-uat.hdpack` exists and is non-empty (`test -s /tmp/hd2-uat.hdpack`).
5. Click **Import Pack** and select `/tmp/hd2-uat.hdpack`.
6. **Expected:** The imported profile appears in the profile list; opening it in the editor shows the same category count and macro count as the source profile. Macro names and key sequences are byte-equivalent to the originals.

### Scenario 5: Malformed import rejection

**Prerequisites:** A valid exported `.hdpack` exists (from Scenario 4 or a fresh export); `/tmp` writable.

1. Truncate the exported pack: `head -c 64 /tmp/hd2-uat.hdpack > /tmp/hd2-uat-bad.hdpack`
2. In the Pack Editor, click **Import Pack** and select `/tmp/hd2-uat-bad.hdpack`.
3. **Expected:** A typed error is displayed inline in the editor in red (the `last_error` field rendered in the UI). The application does not crash.
4. Confirm `~/.config/vibe-attack/profiles/` is uncorrupted: no partial directory was created for the bad import, and the previously active profile is still intact and loadable.

## Edge Cases

### Cancel during Test countdown leaves no trace

1. Click **Test**, immediately click **Cancel**.
2. **Expected:** Daemon JSONL shows no `Fired` event; no key sequence injected to the system.

### Import over existing profile name

1. Export a pack, rename a macro inside the YAML file directly, re-import.
2. **Expected:** Existing profile is replaced cleanly; editor reflects the modified macro name; no orphaned files in profiles directory.

## Automated Evidence

Run `cargo test -- --test-threads=1` from the repo root. All of the following suites must show `0 failed`:

```
pack_hd2_coverage:   test result: ok. 2 passed; 0 failed; ...
pack_lifecycle:      test result: ok. 2 passed; 0 failed; ...
control_integration: test result: ok. 4 passed; 0 failed; ...  (includes TriggerMacro routing)
pack_editor_state_roundtrip: test result: ok. 3 passed; 0 failed; ...
pack_editor_roundtrip:       test result: ok. 3 passed; 0 failed; ...
lib:                 test result: ok. 90 passed; 0 failed; 1 ignored; ...
```

Build verification (clippy substitute):
- `cargo build` → exit 0, 0 warnings
- `cargo build --features gui` → exit 0, 0 warnings

## Failure Signals

- Any `test result:` line showing `>0 failed` — stop; do not sign off
- `vibe-attack-config` crashes on malformed import instead of showing inline error
- No `DispatchOutcome::Fired` in daemon JSONL after Test button countdown completes
- Partial directory left in `~/.config/vibe-attack/profiles/` after a failed import
- Export file is 0 bytes or missing after Export Pack action

## Not Proven By This UAT

- Actual voice recognition accuracy (requires calibrated audio hardware and trained model)
- Multi-user profile isolation
- Performance under high macro count (>500 macros)
- Windows support (deferred per distribution targets decision)
- Per-macro sound feedback (MCRO-04, deferred to future milestone)
- Macro conditional logic (MCRO-03, deferred to future milestone)

## Notes for Tester

- Always run `cargo test -- --test-threads=1` (not parallel) — `test_pack_export_import_with_sounds` has a known shared-tmpdir flake under parallel runs.
- The sign-off line below is intentionally blank — this document is the script deliverable; a human tester must execute the scenarios and countersign.
- If the daemon is not running, Scenarios 1, 2, 3 (fire path), and 4's JSONL checks cannot be completed. Confirm daemon is alive before starting manual scenarios.
- `cargo build` is the accepted clippy substitute in this environment (clippy component not installed). Zero-warning exit from both default and gui feature builds is the verification gate.

---

**Tester sign-off:** _________________________ **Date:** _____________
