# S06: UAT — full pack lifecycle and editor flow

**Goal:** Produce and sign off the manual UAT script for the full pack lifecycle and editor flow, capturing both the manual scenarios that cannot be exercised headlessly and the most recent automated test-suite evidence, so M009's "Final Integrated Acceptance" criteria are demonstrably met.
**Demo:** S06-UAT.md manual steps all pass; cargo test passes including pack_hd2_coverage and pack_lifecycle

## Must-Haves

- .gsd/milestones/M009/slices/S06/S06-UAT.md exists, is non-empty, contains the five manual UAT scenarios from S06-RESEARCH.md (HD2 voice fire, add-macro-and-fire, Test-button countdown, GUI export/import round-trip, malformed import rejection), embeds an automated evidence block from a freshly captured `cargo test -- --test-threads=1` run, and `cargo test -- --test-threads=1` is green (zero failures) at slice completion time.

## Proof Level

- This slice proves: final-assembly — UAT verification only; real runtime required (yes), human/UAT required (yes for the manual scenarios; the automated evidence is captured by running cargo test).

## Integration Closure

Not provided.

## Verification

- Not provided.

## Tasks

- [x] **T01: Author S06-UAT.md and capture cargo test evidence** `est:45m`
  Write the manual UAT script for M009 and capture the automated test-suite evidence in the same document. This is a documentation + verification task — no production code changes. The UAT script must cover the five user-facing scenarios from S06-RESEARCH.md verbatim (HD2 voice stratagem fire, add-macro-and-fire via editor, Test button 1-second countdown, GUI export/import round-trip, malformed import rejection). Each scenario lists prerequisites (e.g. daemon running, profile selected), step-by-step actions, and the explicit observable outcome a human tester checks for. Then run `cargo test -- --test-threads=1` and paste the per-suite `test result:` summary lines into an Automated Evidence section with the date and command. Finally, run `cargo build` and `cargo build --features gui` and record their zero-warning exit (clippy substitute, per S03/S04/S05 precedent — clippy component not installed in this env). The completion handler will move S06 to complete; this task does not call gsd_slice_complete itself.

Do:
1. Create `.gsd/milestones/M009/slices/S06/S06-UAT.md` with sections: Header (title, slice id, date, tester sign-off line), Prerequisites (build, daemon, profile path, hardware desktop session), Manual UAT Scenarios (1–5 from S06-RESEARCH.md), Automated Evidence (cargo test summary block + cargo build summaries), Sign-off line.
2. Run `cargo test -- --test-threads=1` from the repo root. Capture stdout. Extract every `test result:` line and paste into the Automated Evidence block under a fenced code block, prefixed with the absolute date the run was executed and the exact command. Confirm zero failures across the run; if any suite fails, stop and report — do not edit S06-UAT.md to claim a passing run that did not happen.
3. Run `cargo build` and `cargo build --features gui`. Record the exit code and warning count for each.
4. The five scenarios must use these exact stratagem names from `profiles/hd2/pack.yaml`: pick one phrase from each of the 6 ship-module categories (Patriotic Administration Center, Orbital Cannons, Hangar, Bridge, Engineering Bay, Robotics Workshop). Read pack.yaml first to choose unambiguous, distinct phrases.
5. Scenario 2 (add-macro-and-fire) must instruct the tester to enter phrase "extra ammo" with key sequence `KEY_DOWN,KEY_DOWN,KEY_UP,KEY_RIGHT` (verbatim from the research doc) and to verify the daemon JSONL stdout shows a Fired event after the phrase is spoken.
6. Scenario 3 (Test button) must instruct the tester to confirm the countdown label decrements visibly, that Cancel aborts (no MacroCmd::Execute), and that on completion the daemon JSONL shows DispatchOutcome::Fired with score 1.0 (control-plane convention from MEM/S05).
7. Scenario 4 (export/import) must use a tempdir path like `/tmp/hd2-uat.hdpack` and require the tester to verify byte-equivalent macros after import (open the imported profile in the editor and confirm category and macro counts match the source).
8. Scenario 5 (malformed import) must instruct the tester to truncate an exported `.hdpack` (`head -c 64`) and verify a typed error is rendered inline in red in the editor (last_error pattern from S03), no crash, and `~/.config/vibe-attack/profiles/` is uncorrupted (no partial dir was left behind).
9. Do NOT commit any files — `.gsd/` planning docs are managed externally per the milestone instructions.

Must-haves:
- File path is exactly `.gsd/milestones/M009/slices/S06/S06-UAT.md`.
- All 5 scenarios are present, each with Prerequisites/Steps/Expected Outcome subsections.
- Automated Evidence block contains at least 5 `test result:` lines (the suites named in S06-RESEARCH.md: pack_hd2_coverage, pack_lifecycle, control_integration, pack_editor_state_roundtrip, pack_editor_roundtrip) and the lib test summary, all showing 0 failures.
- Date in the doc is the actual run date (today, 2026-04-27).
- Document includes the cargo build evidence (default + gui feature), each with zero warnings.
- Tester sign-off line is included but left unsigned (the document is the script for a human tester; this slice's deliverable is the script itself, not the human signature).
  - Files: `.gsd/milestones/M009/slices/S06/S06-UAT.md`
  - Verify: test -f .gsd/milestones/M009/slices/S06/S06-UAT.md && test -s .gsd/milestones/M009/slices/S06/S06-UAT.md && grep -qc 'Scenario' .gsd/milestones/M009/slices/S06/S06-UAT.md && grep -q 'test result:' .gsd/milestones/M009/slices/S06/S06-UAT.md && grep -q 'cargo test' .gsd/milestones/M009/slices/S06/S06-UAT.md && grep -q 'pack_hd2_coverage' .gsd/milestones/M009/slices/S06/S06-UAT.md && grep -q 'pack_lifecycle' .gsd/milestones/M009/slices/S06/S06-UAT.md && grep -q 'control_integration' .gsd/milestones/M009/slices/S06/S06-UAT.md && cargo test -- --test-threads=1 2>&1 | grep -qE 'test result: ok\. [0-9]+ passed; 0 failed'

## Files Likely Touched

- .gsd/milestones/M009/slices/S06/S06-UAT.md
