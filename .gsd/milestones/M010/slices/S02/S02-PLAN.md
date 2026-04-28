# S02: First-run wizard end-to-end UAT

**Goal:** Wizard runs end-to-end on Debian, Fedora, and Arch (fresh + relaunch + skip-wizard scenarios), with the .desktop Exec target corrected, a --skip-wizard CLI flag implemented, three pending UAT transcripts seeded under docs/distribution-proofs/wizard/, and structural proof tests asserting transcript completeness.
**Demo:** Wizard completes on all three distros; relaunch skips wizard; stratagem fires by voice; three UAT transcripts under docs/distribution-proofs/wizard/

## Must-Haves

- After this: --skip-wizard flag bypasses wizard regardless of disk state; .desktop Exec=vibe-attack-config; docs/distribution-proofs/wizard/{debian12,fedora39,arch}/transcript.md exist with the four-scenario structure (A/B/C/D) and STATUS placeholders; tests/wizard_proofs.rs asserts structural completeness of all three transcripts; existing 16 tests in ui_distribution.rs still pass; new wizard-completion-transition test covers the setup_just_completed edge.

## Proof Level

- This slice proves: - This slice proves: integration (CLI flag + .desktop wiring + structural proof tests, with manual UAT transcripts seeded as pending placeholders matching MEM079 policy)
- Real runtime required: yes (vibe-attack-config binary must launch on each distro for full UAT)
- Human/UAT required: yes (egui rendering, polkit dialogs, evdev key capture cannot be auto-tested)

## Integration Closure

- Upstream surfaces consumed: src/ui/wizard.rs (show_wizard entry), src/ui/first_run.rs (FirstRunState::from_checks), src/ui/probe.rs (probe::run), src/bin/vibe-attack-config.rs (main + VibeAttackConfigApp), packaging/appimage/vibe-attack.desktop (AppImage launch target)
- New wiring introduced in this slice: --skip-wizard arg parse in main() substituting from_checks(true,true,true,true); .desktop Exec target corrected to vibe-attack-config; tests/wizard_proofs.rs new test file
- What remains before the milestone is truly usable end-to-end: real VM-run UAT transcripts (replacing pending placeholders) — completed in S06 final UAT slice per MEM079 policy

## Verification

- Runtime signals: tracing::info! on `--skip-wizard` flag detected at main entry; existing wizard step transitions already emit tracing events
- Inspection surfaces: stderr tracing output when launched from terminal; FirstRunState::is_setup_complete() return value reflected in UI
- Failure visibility: wizard already shows step-level error strings (model download error, modprobe error, ptt capture error); no new failure surfaces required for this slice
- Redaction constraints: none (no secrets in wizard flow; XDG paths are user-local but not sensitive)

## Tasks

- [x] **T01: Fix .desktop Exec target and add ui_distribution.rs assertion** `est:20m`
  The packaging/appimage/vibe-attack.desktop file currently has `Exec=vibe-attack`, but the only built binary is `vibe-attack-config`. AppImage launch will fail on every distro until this is corrected. Also tighten the existing `desktop_file_exists_and_has_required_keys` test in tests/ui_distribution.rs to specifically assert `Exec=vibe-attack-config` so this regression cannot recur silently.

This is a quick, blocking fix — every later task in this slice (and S03/S05/S06) depends on the AppImage actually launching the right binary. No --skip-wizard logic yet; that comes in T02.
  - Files: `packaging/appimage/vibe-attack.desktop`, `tests/ui_distribution.rs`
  - Verify: cargo test --test ui_distribution -- --test-threads=1 desktop_file && grep -q '^Exec=vibe-attack-config$' packaging/appimage/vibe-attack.desktop

- [x] **T02: Implement --skip-wizard CLI flag in vibe-attack-config main()** `est:1h`
  The M010 roadmap specifies a `--skip-wizard` CLI flag so users (and UAT testers) can bypass the wizard regardless of probe state. The flag is currently NOT implemented — `src/bin/vibe-attack-config.rs` does no CLI argument parsing at all.

Minimal implementation per S02-RESEARCH.md recommendation:
- In `main()`, before constructing `VibeAttackConfigApp`, check `std::env::args().any(|a| a == "--skip-wizard")`.
- Pass a bool through `VibeAttackConfigApp::new(log_rx, skip_wizard)` (or via a constructor variant).
- When skip_wizard is true, substitute `FirstRunState::from_checks(true, true, true, true)` instead of calling `probe::run()`.
- Log the skip via `tracing::info!(skip_wizard = true, "Wizard bypass via --skip-wizard flag")` so transcripts show the flag was honored.
- Also handle `--help` / `-h` to print a one-line usage and exit 0 (so `vibe-attack-config --help` works without launching a window).

Add two unit tests in `tests/ui_distribution.rs` (or `tests/cli.rs`) that EXEC the built binary with `--help` and assert exit 0 + usage text presence. Use `env!("CARGO_BIN_EXE_vibe-attack-config")` to locate the binary. Skip the test gracefully if the binary is not built (return early with a log line) rather than panicking — this keeps the suite green on `cargo test --no-default-features` runs.

Do NOT add a full clap dependency for this — keep it std::env only, per project minimalism convention.
  - Files: `src/bin/vibe-attack-config.rs`, `tests/ui_distribution.rs`
  - Verify: cargo test --test ui_distribution -- --test-threads=1 && cargo build --bin vibe-attack-config --features gui && target/debug/vibe-attack-config --help | grep -q 'skip-wizard'

- [x] **T03: Add wizard-completion transition unit test for setup_just_completed** `est:30m`
  The `setup_just_completed` boolean in `src/bin/vibe-attack-config.rs` (line 165, 225, 279, 307) is the edge that triggers profile-load and mic-thread spawn after the wizard finishes. It is currently UNTESTED — there is no automated test covering the `was_incomplete && is_setup_complete()` transition.

Add unit tests that exercise the FirstRunState transition logic that drives `setup_just_completed`. We cannot exec egui frames in a unit test, but we CAN test the pure transition predicate. Specifically:

1. Construct a FirstRunState with all-false; assert `is_setup_complete() == false` (was_incomplete = true).
2. Construct a fresh FirstRunState with all-true (simulating the wizard updating the state via probe::run() at the end); assert `is_setup_complete() == true`.
3. Add a helper test asserting that the boolean transition `was_incomplete && now_complete` holds when going from (false,*,*,*) to (true,true,true,true) but does NOT hold when starting already complete (relaunch scenario per S02-RESEARCH Scenario C).

Write the test in `tests/ui_distribution.rs` (or a new `tests/wizard_transition.rs` if size warrants). Use only the FirstRunState public API — do not poke private fields. The tests must compile WITHOUT `--features gui` (FirstRunState has no egui dependency).

Also add a regression test asserting that `FirstRunState::from_checks(true,true,true,true).first_incomplete_step()` is None — proving the relaunch path will not spuriously enter the wizard.
  - Files: `tests/ui_distribution.rs`
  - Verify: cargo test --test ui_distribution -- --test-threads=1 wizard_completion_transition && cargo test --test ui_distribution -- --test-threads=1

- [ ] **T04: Seed docs/distribution-proofs/wizard/ structure and three pending transcripts** `est:45m`
  Create the wizard UAT proof directory structure following the same MEM079/MEM081 convention used for `docs/distribution-proofs/appimage/`. The directory does not exist yet.

Create the following:

1. `docs/distribution-proofs/wizard/README.md` — copy the format/policy framing from `docs/distribution-proofs/appimage/README.md`, adapted for wizard scenarios. Document the four required UAT scenarios (A: fresh install, B: partial state with model pre-placed, C: relaunch / wizard skipped, D: --skip-wizard flag) per S02-RESEARCH.md. Document the per-distro reproduction steps. Document the STATUS values: `ok`, `pending VM run`, `failed:scenario-A`, `failed:scenario-B`, `failed:scenario-C`, `failed:scenario-D`.

2. `docs/distribution-proofs/wizard/debian12/transcript.md` — pending placeholder with these required structured fields (one per line):
   - `STATUS: pending VM run`
   - `DISTRO: pending`
   - `KERNEL: pending`
   - `BINARY: vibe-attack-config`
   - `BINARY_VERSION: pending`
   - `SCENARIO_A: pending` (fresh install)
   - `SCENARIO_B: pending` (model pre-placed)
   - `SCENARIO_C: pending` (relaunch skips wizard)
   - `SCENARIO_D: pending` (--skip-wizard flag)
   - `STRATAGEM_FIRED: pending` (whether at least one stratagem was fired by voice in scenario A)
   Followed by a free-form `## Reproduction Notes` section with bullet-point steps a tester would follow.

3. `docs/distribution-proofs/wizard/fedora39/transcript.md` — same structure, pending placeholders.

4. `docs/distribution-proofs/wizard/arch/transcript.md` — same structure, pending placeholders.

IMPORTANT: do NOT mark these as `STATUS: ok` — they are pending real VM runs. Use `pending VM run` per MEM079. The tests in T05 will accept that status. Real `ok` runs are completed in S06 (final UAT) per the milestone roadmap.

All four files must be tracked in git (no .gitignore matches under docs/).
  - Files: `docs/distribution-proofs/wizard/README.md`, `docs/distribution-proofs/wizard/debian12/transcript.md`, `docs/distribution-proofs/wizard/fedora39/transcript.md`, `docs/distribution-proofs/wizard/arch/transcript.md`
  - Verify: test -f docs/distribution-proofs/wizard/README.md && for d in debian12 fedora39 arch; do test -f docs/distribution-proofs/wizard/$d/transcript.md && grep -q '^STATUS:' docs/distribution-proofs/wizard/$d/transcript.md && grep -q '^SCENARIO_A:' docs/distribution-proofs/wizard/$d/transcript.md && grep -q '^SCENARIO_D:' docs/distribution-proofs/wizard/$d/transcript.md; done

- [ ] **T05: Add tests/wizard_proofs.rs structural assertions for wizard transcripts** `est:45m`
  Mirror the structure of `tests/distribution_proofs.rs` (which validates AppImage transcripts) for the wizard transcripts created in T04. The test file is `tests/wizard_proofs.rs` (new).

Requirements:

1. Define `REQUIRED_FIELDS` slice including: `STATUS:`, `DISTRO:`, `KERNEL:`, `BINARY:`, `BINARY_VERSION:`, `SCENARIO_A:`, `SCENARIO_B:`, `SCENARIO_C:`, `SCENARIO_D:`, `STRATAGEM_FIRED:`.

2. Define `VALID_STATUSES` slice: `STATUS: ok`, `STATUS: pending VM run`, plus the four `STATUS: failed:scenario-{A,B,C,D}` values.

3. One test per distro (`debian12_wizard_transcript_has_required_fields`, `fedora39_*`, `arch_*`) — each reads the transcript and asserts every REQUIRED_FIELD substring is present and at least one VALID_STATUS line is present.

4. One test asserting `docs/distribution-proofs/wizard/README.md` exists and contains the substrings `Scenario A`, `Scenario B`, `Scenario C`, and `Scenario D` so future testers cannot accidentally drop the four-scenario structure.

5. Use `env!("CARGO_MANIFEST_DIR")` to locate the project root; do NOT depend on the test's CWD. Match the helper-function shape of `tests/distribution_proofs.rs` exactly (same `project_root`, `read_file`, `assert_transcript` pattern).

6. Per MEM080: this test file will run with `--test-threads=1` along with the other distribution_proofs/packaging/ui_distribution suites. It is purely file-IO so it will not flake, but follow the convention.

After writing, run `cargo test --test wizard_proofs -- --test-threads=1` and confirm all tests pass against the pending placeholders from T04.
  - Files: `tests/wizard_proofs.rs`
  - Verify: cargo test --test wizard_proofs -- --test-threads=1 && cargo test --test ui_distribution -- --test-threads=1 && cargo test --test distribution_proofs -- --test-threads=1

## Files Likely Touched

- packaging/appimage/vibe-attack.desktop
- tests/ui_distribution.rs
- src/bin/vibe-attack-config.rs
- docs/distribution-proofs/wizard/README.md
- docs/distribution-proofs/wizard/debian12/transcript.md
- docs/distribution-proofs/wizard/fedora39/transcript.md
- docs/distribution-proofs/wizard/arch/transcript.md
- tests/wizard_proofs.rs
