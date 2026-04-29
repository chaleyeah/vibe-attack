# S03: UI polish from proof-run findings — UAT

**Milestone:** M011
**Written:** 2026-04-29T01:48:26.122Z

# S03 UAT — UI Polish from Proof-Run Findings

## Preconditions
- Dev host with GUI capability (X11 or Wayland).
- `cargo build --release --features gui --bin vibe-attack-config` succeeds.
- A valid `~/.config/vibe-attack/config.yaml` exists with a `ptt_binding` set (e.g. `KEY_F13`).

---

## Test Cases

### TC-01: PTT Manual Key Persists Across Frames
**What:** Verify the manual-key TextEdit in the wizard PTT configuration step retains typed text across repaints.

**Steps:**
1. Launch `./target/release/vibe-attack-config` (without `--skip-wizard` if setup is incomplete, or reset wizard state).
2. Navigate to the PTT key configuration step.
3. Select "Enter key name manually" (or equivalent option).
4. Type `KEY_F13` character by character into the text field.
5. Observe the field after typing — do not press Save yet.

**Expected:** The field retains `KEY_F13`; it does not reset to empty between characters or on the next repaint.

**Edge case:** Click elsewhere in the window to force a repaint, then return focus to the field. Text must still be `KEY_F13`.

---

### TC-02: Install-Model Auto-Advances When Model Already Downloaded
**What:** Verify that re-entering the wizard after a completed download advances past the install-model step automatically.

**Steps:**
1. Ensure the model file is present (probe::run() would return a state beyond InstallModel).
2. Launch the wizard (force-reset state if needed so it starts at InstallModel step).
3. Observe the install-model panel on entry.

**Expected:** The wizard advances to the next step immediately without requiring the user to click Re-check. If the Re-check button appears, clicking it must also advance correctly.

---

### TC-03: Uinput Note Has Dark-Amber Background (Legibility)
**What:** Verify the uinput warning note is visually bounded with a dark-amber frame.

**Steps:**
1. Navigate to the uinput setup step in the wizard.
2. Observe the warning/note text about uinput group membership.

**Expected:** The note appears inside a visually distinct dark-amber framed box (not bare yellow text on the default background). Text color is warm yellow (~rgb(255,200,60)) on a dark amber background (~rgb(64,50,0)).

---

### TC-04: Download Failure Shows HuggingFace CDN Hint
**What:** Verify that a model download failure prepends the CDN redirect hint.

**Steps:**
1. Block outbound connections to HuggingFace/CDN (e.g. via firewall rule or disconnect network).
2. Trigger model download in the wizard.
3. Wait for the download to fail.

**Expected:** The failure message begins with "HuggingFace serves a 302 redirect to a CDN — if your network blocks the CDN this will fail." followed by the raw error string on the next line.

---

### TC-05: Tray Quit Closes Window Cleanly (No process::exit)
**What:** Verify that right-clicking the tray icon and selecting Quit closes the config window gracefully.

**Steps:**
1. Launch `./target/release/vibe-attack-config --skip-wizard` with a D-Bus session available.
2. Observe the tray icon appears in the system tray.
3. Right-click the tray icon → select "Quit".
4. Observe the application window and stderr output.

**Expected:** The window closes without abrupt termination. Stderr contains `"Tray quit requested"` (from `tracing::info!`). The process exits cleanly (exit code 0, no panic backtrace).

**Edge case (no D-Bus):** If D-Bus is unavailable, tray is None; the Quit option cannot be exercised but the application must still launch and close via the window's native close button.

---

### TC-06: Tray Tooltip Reflects Activation Mode
**What:** Verify the tray tooltip text changes depending on whether PTT or Wake mode is active.

**Steps:**
1. Launch with PTT mode configured. Hover over the tray icon while daemon is Idle.
2. Switch to Wake mode (via config window or daemon control). Hover again.

**Expected:**
- PTT mode: tooltip contains "waiting for PTT key".
- Wake mode: tooltip contains "listening for wake word".
- Non-Idle states (Recording, Muted, Listening): tooltip is the same regardless of mode.

---

### TC-07: Config Screen Shows "(configured in wizard)" Affordance
**What:** Verify the main config screen's PTT key row shows the disambiguation weak text.

**Steps:**
1. Launch `./target/release/vibe-attack-config --skip-wizard`.
2. Observe the PTT key row in the main config screen.

**Expected:** The row shows both `PTT key: KEY_F13` (or configured value) and a muted/weak "(configured in wizard)" label next to it. No "Reconfigure…" button is present.

---

### TC-08: Unit Test Suite Passes (Regression Check)
**What:** Confirm all unit tests pass after S03 changes.

**Steps:**
1. Run `cargo test --features gui --lib`.

**Expected:** `test result: ok. N passed; 0 failed` where N ≥ 105. Specifically, the following new tests must pass: `ui::wizard::tests::manual_key_persists_in_state`, `ui::wizard::tests::manual_key_default_empty`, `ui::tray::tests::tooltip_description_idle_ptt`, `ui::tray::tests::tooltip_description_idle_wake`, `ui::tray::tests::tray_handle_take_quit_request_clears_flag`.

---

### TC-09: Distribution and Wizard Proofs Remain Green
**What:** Sanity check that S01/S02 scaffolding is unbroken.

**Steps:**
1. Run `cargo test --test distribution_proofs -- --test-threads=1`.
2. Run `cargo test --test wizard_proofs -- --test-threads=1`.

**Expected:** Both suites pass (11/11 and 5/5 respectively).

---

## Follow-Up (Out of Scope for This UAT)
Wizard-finding-driven items (from actual VM runs) are deferred. Gated on at least one `wizard/{distro}/transcript.md` reaching `SCENARIO_A: ok|failed:*`. When unblocked: read all four transcripts' `## Findings` sections, group by file/severity, and file as M012 candidate work.
