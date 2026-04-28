# S06 UAT — Full Pack Lifecycle and Editor Flow

**Slice:** S06  
**Milestone:** M009  
**Date:** 2026-04-27  
**Tester sign-off:** _______________________ (name / date)

---

## Prerequisites

| Requirement | Detail |
|---|---|
| Build | `cargo build --features gui` exits 0 with zero warnings |
| Daemon | `vibe-attack` daemon running (`vibe-attack --config ~/.config/vibe-attack/config.yaml`) |
| Profile | `hd2` profile selected as active (`~/.config/vibe-attack/profiles/hd2/pack.yaml` present) |
| Desktop session | Physical Wayland or X11 session required (rfd file dialog and uinput injection need a real desktop) |
| uinput access | Current user is in the `input` group or `/dev/uinput` is readable |
| Config app | `vibe-attack-config` binary built and launchable |

---

## Manual UAT Scenarios

### Scenario 1 — HD2 Voice Stratagem Fire

**Goal:** Verify that speaking a stratagem phrase from each of the 6 ship-module categories causes the corresponding key sequence to be injected via uinput.

**Prerequisites:**
- Daemon running with `hd2` as the active profile.
- A Helldivers 2 game session or any app that captures directional key input is open (or use `evtest` / `xev` to observe injected keys).

**Steps:**
1. Ensure the push-to-talk key is bound and the daemon status shows `listening`.
2. Hold the PTT key and speak: **"railgun"** (Patriotic Administration Center).  
   Release PTT after speaking.  
   Expected outcome: keys `DOWN RIGHT DOWN LEFT DOWN RIGHT` are injected.
3. Hold PTT and speak: **"orbital airburst strike"** (Orbital Cannons).  
   Expected outcome: keys `RIGHT RIGHT RIGHT` are injected.
4. Hold PTT and speak: **"eagle strafing run"** (Hangar).  
   Expected outcome: keys `UP RIGHT RIGHT` are injected.
5. Hold PTT and speak: **"resupply"** (Bridge).  
   Expected outcome: keys `DOWN DOWN UP RIGHT` are injected.
6. Hold PTT and speak: **"guard dog"** (Engineering Bay).  
   Expected outcome: keys `DOWN UP LEFT UP RIGHT DOWN` are injected.
7. Hold PTT and speak: **"gatling sentry"** (Robotics Workshop).  
   Expected outcome: keys `DOWN UP RIGHT LEFT` are injected.

**Expected Outcome:**  
For each of the 6 phrases above, the daemon JSONL stdout emits a `dispatch` event with `outcome: Fired` and the injected key sequence matches the `keys` field in `profiles/hd2/pack.yaml`. No `NoMatch` events appear for these phrases. The game or key-capture tool registers the correct directional inputs.

---

### Scenario 2 — Add Macro via Editor and Fire It

**Goal:** Verify that a new macro added through the pack editor is saved, recognized by the daemon, and fired by voice.

**Prerequisites:**
- Daemon running with `hd2` as the active profile.
- `vibe-attack-config` config app open, hd2 pack editor visible.

**Steps:**
1. Open `vibe-attack-config`. Navigate to the Pack Editor tab for the `hd2` profile.
2. Click **Add Macro** (or the equivalent "+" button in a category).
3. Set the macro fields:
   - **Phrase:** `extra ammo`
   - **Keys:** `KEY_DOWN,KEY_DOWN,KEY_UP,KEY_RIGHT`
   - **Category:** pick any existing category (e.g. "Bridge")
4. Click **Save**. Confirm the macro appears in the category list with phrase "extra ammo".
5. Reload the daemon profile: restart `vibe-attack`, or use the Reload Config command if available, so the new macro is in the active dispatcher.
6. Hold the PTT key, speak **"extra ammo"**, release PTT.
7. Inspect the daemon JSONL stdout for a `dispatch` event.

**Expected Outcome:**  
The daemon JSONL stdout shows a `dispatch` event with `outcome: Fired` for macro name containing "extra ammo". The key sequence `DOWN DOWN UP RIGHT` is injected. No crash occurs. The macro persists in `profiles/hd2/pack.yaml` after saving.

---

### Scenario 3 — Test Button (1-Second Countdown)

**Goal:** Verify the Test button in the pack editor fires a macro via the control plane and shows the 1-second safety countdown, including Cancel behavior.

**Prerequisites:**
- Daemon running (the Test button requires `daemon_running: true` in the config app state).
- `vibe-attack-config` config app open, hd2 pack editor visible with at least one macro selectable.

**Steps — happy path (fire):**
1. In the Pack Editor, select any macro (e.g. click on "Railgun").
2. Click the **Test** button.
3. Observe the button label: it must visibly decrement from approximately `Test (1.0s)` → `Test (0.5s)` → fire. The countdown uses 50 ms repaint intervals driven by `Instant` polling (no sleep).
4. Let the countdown complete without clicking Cancel.
5. Inspect the daemon JSONL stdout.

**Expected Outcome (fire):**  
The daemon JSONL stdout shows a `dispatch` event with `outcome: Fired` and `score: 1.0`. The key sequence for the selected macro is injected. The button returns to its normal "Test" label.

**Steps — Cancel path:**
1. Select any macro and click **Test**.
2. While the countdown label is visible, immediately click **Cancel** (or the Test button again, per the abort wiring).
3. Confirm the countdown stops and no key injection occurs.

**Expected Outcome (cancel):**  
No `MacroCmd::Execute` event appears in the daemon JSONL after cancel. The button returns to the normal "Test" label. No crash occurs.

---

### Scenario 4 — Export → Import Round-Trip (GUI)

**Goal:** Verify that exporting a pack to `.hdpack` and importing it into a fresh profile slot produces an identical macro set.

**Prerequisites:**
- `vibe-attack-config` config app open, hd2 pack editor visible.
- `/tmp/` is writable.

**Steps:**
1. In the Pack Editor for `hd2`, click **Export Pack**.
2. In the system file-picker dialog, navigate to `/tmp/` and save as `hd2-uat.hdpack`. Confirm the dialog closes and no error appears.
3. Click **Import Pack**.
4. In the file-picker, select `/tmp/hd2-uat.hdpack`.
5. The config app imports the pack; the profiles list should now show a new profile slot (the imported pack name, or a name derived from the `.hdpack` contents).
6. Open the newly imported profile in the Pack Editor.
7. Compare category names and macro counts to the original `hd2` profile.

**Expected Outcome:**  
The imported profile contains the same 6 categories (Patriotic Administration Center, Orbital Cannons, Hangar, Bridge, Engineering Bay, Robotics Workshop) with the same macro counts as the source pack. At least one macro name and phrase can be spot-checked (e.g. "Railgun" → phrase "railgun"). No error banner appears. `~/.config/vibe-attack/profiles/` contains the newly imported profile directory with a valid `pack.yaml`.

---

### Scenario 5 — Malformed Import Rejection

**Goal:** Verify that a corrupted `.hdpack` file produces a typed inline error in the editor without crashing the app or corrupting the profiles directory.

**Prerequisites:**
- `vibe-attack-config` config app open.
- A valid `.hdpack` exists at `/tmp/hd2-uat.hdpack` (from Scenario 4).

**Steps:**
1. In a terminal, create a truncated copy of the pack:
   ```
   head -c 64 /tmp/hd2-uat.hdpack > /tmp/hd2-corrupt.hdpack
   ```
2. In the Pack Editor, click **Import Pack**.
3. In the file-picker, select `/tmp/hd2-corrupt.hdpack`.
4. Observe the editor UI after the import attempt.

**Expected Outcome:**  
- An inline error message is displayed in red in the pack editor (the `last_error` label rendered in `Color32::RED`). The error text describes the failure (e.g. "invalid zip", "failed to parse pack.yaml", or similar).
- The config app does **not** crash or become unresponsive.
- No partial profile directory is left behind in `~/.config/vibe-attack/profiles/`. Verify with:
  ```
  ls ~/.config/vibe-attack/profiles/
  ```
  The directory listing shows only pre-existing profiles; no new partial directory from the failed import appears.
- Existing profiles (e.g. `hd2`) remain intact and loadable.

---

## Automated Evidence

Run date: **2026-04-27**  
Command: `cargo test -- --test-threads=1`

```
Finished `test` profile [unoptimized + debuginfo] target(s) in 0.09s
 Running unittests src/lib.rs

running 91 tests
test result: ok. 90 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.17s

 Running unittests src/main.rs

running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

 Running tests/concurrency_stress.rs

running 1 test
test concurrency_stress_pipeline_topology ... ignored, stress test; set RUN_STRESS_TESTS=1 to enable
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s

 Running tests/config_parse.rs

running 6 tests
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

 Running tests/control_integration.rs

running 4 tests
test set_mode_round_trip_via_socket ... ok
test set_threshold_via_socket_updates_dispatcher ... ok
test test_macro_unknown_name_returns_error ... ok
test test_macro_via_socket_fires_dispatcher ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

 Running tests/control_protocol.rs

running 21 tests
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

 Running tests/daemon_headless.rs

running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

 Running tests/dispatcher_logic.rs

running 4 tests
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

 Running tests/documentation.rs

running 11 tests
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

 Running tests/drop_oldest_queue.rs

running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

 Running tests/jsonl_schema.rs

running 4 tests
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

 Running tests/macro_inject.rs

running 3 tests
test result: ok. 0 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 0.00s

 Running tests/pack_editor_roundtrip.rs

running 3 tests
test roundtrip_after_full_crud_sequence ... ok
test roundtrip_preserves_optional_fields ... ok
test roundtrip_yaml_text_stable_within_run ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

 Running tests/pack_editor_state_roundtrip.rs

running 3 tests
test state_parse_key_sequence_drives_form_to_save ... ok
test state_roundtrip_after_full_crud_via_state_layer ... ok
test state_save_to_dir_writes_pack_yaml ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

 Running tests/pack_hd2_bundle.rs

running 22 tests
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

 Running tests/pack_hd2_coverage.rs

running 2 tests
test hd2_pack_covers_all_ship_modules ... ok
test hd2_pack_phrases_are_unique ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

 Running tests/pack_lifecycle.rs

running 2 tests
test pack_export_imports_sounds_subdirectory ... ok
test pack_export_then_import_to_round_trips_macros ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

 Running tests/packaging.rs

running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

 Running tests/profile_listing.rs

running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

 Running tests/runtime_commands.rs

running 6 tests
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

 Running tests/stt_smoke.rs

running 1 test
test whisper_loads_model_and_runs_one_pass ... ignored
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s

 Running tests/ui_distribution.rs

running 16 tests
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

 Running tests/uinput_smoke.rs

running 2 tests
test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s

 Running tests/wake_word.rs

running 2 tests
test result: ok. 0 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.00s

 Doc-tests vibe_attack

running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Key suite summaries (required by M009 acceptance criteria):**

| Suite | Result |
|---|---|
| `lib` (unit tests) | ok. 90 passed; 0 failed; 1 ignored |
| `pack_hd2_coverage` | ok. 2 passed; 0 failed; 0 ignored |
| `pack_lifecycle` | ok. 2 passed; 0 failed; 0 ignored |
| `control_integration` | ok. 4 passed; 0 failed; 0 ignored |
| `pack_editor_state_roundtrip` | ok. 3 passed; 0 failed; 0 ignored |
| `pack_editor_roundtrip` | ok. 3 passed; 0 failed; 0 ignored |

**cargo build evidence:**

```
# Default (no gui feature)
$ cargo build
   Compiling vibe-attack v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.50s
Exit code: 0   Warnings: 0

# GUI feature
$ cargo build --features gui
   Compiling vibe-attack v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.07s
Exit code: 0   Warnings: 0
```

Note: `cargo clippy` is not installed in this environment (no rustup/rust-clippy). `cargo build` is the accepted substitute per prior slice conventions (S03/S04/S05 precedent, MEM038).

---

## Sign-Off

| Role | Name | Date | Signature |
|---|---|---|---|
| Tester | | | |
| Reviewer | | | |
