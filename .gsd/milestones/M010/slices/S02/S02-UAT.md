# S02: First-run wizard end-to-end UAT — UAT

**Milestone:** M010
**Written:** 2026-04-28T04:07:32.352Z

# S02 UAT: First-Run Wizard End-to-End

## Preconditions

- Built binary available: `target/debug/vibe-attack-config` (or AppImage equivalent)
- `libsherpa-onnx-c-api.so` present in `target/debug/` for dynamic linking
- Test machine has no pre-existing `~/.config/vibe-attack/config.yaml` (fresh install scenario)

---

## TC-01: .desktop Exec Target Correctness

**Precondition:** `packaging/appimage/vibe-attack.desktop` exists.

**Steps:**
1. Run: `grep '^Exec=' packaging/appimage/vibe-attack.desktop`

**Expected:** Output is exactly `Exec=vibe-attack-config` (not `Exec=vibe-attack` or any other value).

**Pass criterion:** grep exits 0 and output matches exactly.

---

## TC-02: --help Flag Exits Zero Without Opening a Window

**Precondition:** Binary is built (`cargo build --bin vibe-attack-config --features gui`).

**Steps:**
1. Run: `LD_LIBRARY_PATH=target/debug target/debug/vibe-attack-config --help`
2. Observe exit code.
3. Observe stdout content.

**Expected:**
- Exit code: 0
- stdout contains the substring `skip-wizard`
- stdout contains the substring `Usage:`
- No GUI window opens; process returns immediately.

**Edge case:** Also test `-h` alias — must behave identically.

---

## TC-03: --skip-wizard Flag Bypasses Wizard Regardless of Disk State

**Precondition:** Binary is built. Fresh install (no `config.yaml`).

**Steps:**
1. Run: `LD_LIBRARY_PATH=target/debug target/debug/vibe-attack-config --skip-wizard 2>&1 | head -5`
2. Check stderr/tracing output for the bypass log line.

**Expected:**
- tracing output contains: `skip_wizard=true` and `Wizard bypass via --skip-wizard flag`
- App opens directly to the main config view (no wizard steps shown).

**Edge case:** Run with `--skip-wizard` when `config.yaml` already exists — wizard must also not appear (both paths should skip it).

---

## TC-04: Wizard Appears on Fresh Install (No --skip-wizard)

**Precondition:** No `~/.config/vibe-attack/config.yaml`. No `--skip-wizard` flag.

**Steps:**
1. Run: `LD_LIBRARY_PATH=target/debug target/debug/vibe-attack-config`
2. Observe the GUI window.

**Expected:**
- Wizard steps are shown starting from the first incomplete step (mic check or uinput permission or model download).
- The wizard does NOT skip to the main config view.

---

## TC-05: Relaunch Skips Wizard (Scenario C)

**Precondition:** Wizard was previously completed; `config.yaml` exists; all probes pass.

**Steps:**
1. Ensure `~/.config/vibe-attack/config.yaml` exists and is valid.
2. Run: `LD_LIBRARY_PATH=target/debug target/debug/vibe-attack-config`
3. Observe the GUI window.

**Expected:**
- Wizard does NOT appear.
- App opens directly to the main config view.
- `FirstRunState::from_checks(true,true,true,true).first_incomplete_step()` returns None (verified by unit test `relaunch_state_has_no_first_incomplete_step`).

---

## TC-06: Wizard Transcript Files Exist with Required Structure

**Steps:**
1. Run: `cargo test --test wizard_proofs -- --test-threads=1`

**Expected:** 4/4 tests pass:
- `debian12_wizard_transcript_has_required_fields`
- `fedora39_wizard_transcript_has_required_fields`
- `arch_wizard_transcript_has_required_fields`
- `wizard_readme_contains_four_scenario_headings`

---

## TC-07: Wizard Transcript STATUS Values Are Pending (Not Prematurely Marked OK)

**Steps:**
1. For each distro: `grep '^STATUS:' docs/distribution-proofs/wizard/{debian12,fedora39,arch}/transcript.md`

**Expected:** All three output `STATUS: pending VM run`. None should show `STATUS: ok` until S06 VM runs are complete.

---

## TC-08: Full Test Suite Regression Check

**Steps:**
1. `cargo test --test ui_distribution -- --test-threads=1`
2. `cargo test --test wizard_proofs -- --test-threads=1`
3. `cargo test --test distribution_proofs -- --test-threads=1`

**Expected:** 21 + 4 + 6 = 31 total assertions, 0 failures.

---

## Deferred to S06 (Requires Real VM Runs)

- Scenario A: Fresh install wizard completes end-to-end on Debian 12, Fedora 39, Arch
- Scenario B: Model pre-placed, wizard skips model download step
- Scenario D: `--skip-wizard` flag confirmed in UAT transcript on each distro
- `STRATAGEM_FIRED: ok` confirmed for each distro
- `STATUS: ok` written to all three transcript files
