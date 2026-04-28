# S05: TriggerMacro control request + editor Test button — UAT

**Milestone:** M009
**Written:** 2026-04-28T03:22:19.211Z

# S05 UAT — TriggerMacro Control Request + Editor Test Button

## Preconditions

- Daemon (`vibe-attack-daemon`) is running with a profile that contains at least one macro (e.g., `profiles/hd2/pack.yaml`).
- Config app (`vibe-attack-config`) is built with `--features gui`.
- User has `/dev/uinput` access (or is in the `input` group / has `udev` rule applied).
- Daemon stdout is visible (e.g., piped through `journalctl -f` or a terminal).

---

## Test Case 1 — Happy Path: Test button fires macro via daemon

**Steps:**
1. Launch the config app: `./target/debug/vibe-attack-config`
2. Open the Pack Editor panel.
3. Select any macro from the macro list (e.g., "eagle_airstrike").
4. Observe: a **Test** button appears in the button row alongside Update Macro / Remove Macro.
5. Click **Test**.
6. **Expected:** The Test button disappears and a countdown label appears: `Firing in 1.0s... (Cancel)`.
7. Watch the countdown animate smoothly from 1.0s to 0.0s over approximately 1 second (repaint every ~50ms).
8. After 1 second, the countdown disappears and a green status label appears: `Fired: eagle_airstrike`.
9. **Expected:** The daemon's stdout/journalctl shows `INFO Firing macro (direct) macro_name="eagle_airstrike"` and `INFO TestMacro request received macro_name="eagle_airstrike"`.
10. **Expected:** The uinput subsystem received the key sequence for eagle_airstrike (verifiable via `evtest` or game observation).

**Pass criteria:** Status label shows "Fired: ..." in green; daemon log shows both tracing lines; no crash.

---

## Test Case 2 — Cancel aborts the countdown

**Steps:**
1. Select a macro in the Pack Editor.
2. Click **Test**.
3. While the countdown is running (before 1 second elapses), click **Cancel**.
4. **Expected:** Countdown disappears; Test button reappears immediately.
5. **Expected:** No `MacroCmd::Execute` is delivered to the daemon (no "Firing macro (direct)" log line appears).
6. **Expected:** `last_test_status` remains unchanged (no new status label if none existed before).

**Pass criteria:** Cancel halts the countdown; no macro fires; UI returns to idle state.

---

## Test Case 3 — Test button greyed out when daemon is not running

**Steps:**
1. Stop the daemon: `pkill vibe-attack-daemon` (or equivalent).
2. Open the Pack Editor and select a macro.
3. **Expected:** The Test button is rendered but greyed out (non-interactive via `add_enabled(false, ...)`).
4. Attempt to click the greyed button.
5. **Expected:** Nothing happens; no countdown starts; no error log.

**Pass criteria:** Button visually disabled when daemon is absent; clicking has no effect.

---

## Test Case 4 — Unknown macro name returns error in UI

**Steps (automated path — verified by integration test; manual path for UAT):**
1. If the profile is editable, manually change a macro name in pack.yaml to something unique.
2. Reload the profile (SwitchProfile or daemon restart).
3. In the editor, select the renamed macro.
4. Click **Test** using the *old* name (simulate by temporarily calling send_command with a nonexistent name via a debug build if necessary — otherwise this is covered by `test_macro_unknown_name_returns_error`).
5. **Expected:** After 1 second, status label shows `Test failed: macro not found: <name>` in red.

**Pass criteria:** Error path surfaces cleanly in the UI; no panic; daemon returns ControlResponse::Error.

---

## Test Case 5 — Daemon connection failure shows error

**Steps:**
1. Stop the daemon mid-countdown (kill daemon after Test is clicked but before 1 second elapses, then let the countdown complete).
2. **Expected:** After elapsed >= 1s, `send_command` returns an IO error.
3. **Expected:** Status label shows `Daemon error: <io error message>` in red.

**Pass criteria:** Network failure is surfaced as a red label; no panic; UI remains usable.

---

## Automated Regression Checks (run before sign-off)

```bash
cargo test -- --test-threads=1
RUSTFLAGS="-D warnings" cargo check --all-targets
RUSTFLAGS="-D warnings" cargo build --features gui --bin vibe-attack-config
```

All must exit 0 with 0 failures and 0 warnings.
