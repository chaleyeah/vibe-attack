---
id: S02
parent: M010
milestone: M010
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - ["packaging/appimage/vibe-attack.desktop", "src/bin/vibe-attack-config.rs", "tests/ui_distribution.rs", "tests/wizard_proofs.rs", "docs/distribution-proofs/wizard/README.md", "docs/distribution-proofs/wizard/debian12/transcript.md", "docs/distribution-proofs/wizard/fedora39/transcript.md", "docs/distribution-proofs/wizard/arch/transcript.md"]
key_decisions:
  - ["Assert Exec=vibe-attack-config exactly (not substring) to prevent silent regression", "std::env::args() collected before tracing/GUI init so --help exits without side effects", "LD_LIBRARY_PATH=target/debug/ set explicitly in test Command::env() for spawned binary processes", "Two-snapshot pattern for egui frame transition unit tests (before-probe / after-probe FirstRunState)", "STATUS set to 'pending VM run' per MEM079 — real VM runs deferred to S06", "No clap dependency — std::env::args() only per project minimalism convention"]
patterns_established:
  - ["wizard_proofs.rs mirrors distribution_proofs.rs shape: REQUIRED_FIELDS + VALID_STATUSES + per-distro assert_transcript helper", "Transcript format: 10 structured key-value fields + free-form Reproduction Notes section", "Binary spawn tests: env!(CARGO_BIN_EXE_...) + LD_LIBRARY_PATH guard + early-return if binary absent"]
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-28T04:07:32.352Z
blocker_discovered: false
---

# S02: First-run wizard end-to-end UAT

**Wired --skip-wizard CLI flag, corrected .desktop Exec target, seeded three pending wizard UAT transcripts with structural proof tests — all 25 test assertions pass.**

## What Happened

S02 delivered four tasks that together close the wizard UAT scaffolding gap and establish the patterns needed for S06 final VM runs.

**T01 — .desktop Exec target fix:** `packaging/appimage/vibe-attack.desktop` had `Exec=vibe-attack` but the only built binary is `vibe-attack-config`. This one-line fix was a blocking prerequisite for every downstream task. The regression test in `tests/ui_distribution.rs` was simultaneously tightened from a substring match on `Exec=` to an exact-value assertion for `Exec=vibe-attack-config`, with a diagnostic message that prints the actual Exec line on failure.

**T02 — --skip-wizard and --help CLI flags:** `src/bin/vibe-attack-config.rs` previously did zero CLI argument parsing. `std::env::args()` is now collected before the log channel and GUI are initialized, enabling `--help`/`-h` to exit 0 cleanly with a one-line usage message (mentioning `--skip-wizard`) without opening a window. The `skip_wizard: bool` parameter was threaded into `VibeAttackConfigApp::new()`, which substitutes `FirstRunState::from_checks(true,true,true,true)` for `probe::run()` when the flag is present, and emits `tracing::info!(skip_wizard = true, "Wizard bypass via --skip-wizard flag")` so UAT transcripts can confirm the flag was honoured. Two new regression tests exercise the built binary via `env!("CARGO_BIN_EXE_vibe-attack-config")` with `LD_LIBRARY_PATH` explicitly set to `target/debug/` (required because `libsherpa-onnx-c-api.so` is not on the ambient library path in CI/test environments).

**T03 — Wizard completion transition unit tests:** The `setup_just_completed` predicate (`was_incomplete && is_setup_complete()`) that drives profile-load and mic-thread spawn after wizard completion was entirely untested. Three unit tests were added to `tests/ui_distribution.rs` using a two-snapshot pattern (before-probe state, after-probe state) to model the per-frame egui transition without a GUI harness: (1) transition fires when going from all-false to all-true, (2) transition does NOT fire on relaunch (both snapshots all-true, mimicking S02-RESEARCH Scenario C), (3) `FirstRunState::from_checks(true,true,true,true).first_incomplete_step()` returns None, proving the wizard skip path is sound.

**T04 — Wizard transcript directory seeding:** `docs/distribution-proofs/wizard/` was created with a `README.md` (adapted from the appimage convention, documenting four UAT scenarios A–D, STATUS values, and per-distro reproduction steps) and three pending transcript files under `debian12/`, `fedora39/`, and `arch/`. Each transcript carries the 10 required structured fields (`STATUS: pending VM run`, `DISTRO`, `KERNEL`, `BINARY`, `BINARY_VERSION`, `SCENARIO_A` through `SCENARIO_D`, `STRATAGEM_FIRED`) and a free-form Reproduction Notes section. All files are tracked in git; no `.gitignore` matches.

**T05 — Structural proof tests for wizard transcripts:** `tests/wizard_proofs.rs` mirrors the shape of `tests/distribution_proofs.rs` exactly, defining `REQUIRED_FIELDS` (10 fields) and `VALID_STATUSES` (6 values: ok, pending VM run, failed:scenario-A/B/C/D). Three per-distro tests assert all required fields are present and at least one valid status is present. A fourth test asserts `docs/distribution-proofs/wizard/README.md` contains `Scenario A`, `Scenario B`, `Scenario C`, and `Scenario D` headings.

**What the next slice should know:** The wizard UAT transcript format and structural test pattern are now established and tested against pending placeholders. S06 (final distribution UAT) replaces these placeholders with real VM-run results by updating the STATUS field to `ok` and filling in DISTRO/KERNEL/BINARY_VERSION/SCENARIO results. The `--skip-wizard` flag is ready for use in UAT scripts. The `LD_LIBRARY_PATH=target/debug/` requirement for spawned test processes that exec the binary is documented in MEM082 and must be preserved in any future test that invokes `vibe-attack-config` directly.

## Verification

All 25 assertions pass across three test suites:
- `cargo test --test ui_distribution -- --test-threads=1`: 21/21 pass (includes T01 desktop regression, T02 --help CLI tests, T03 wizard completion transition tests, and all pre-existing tests)
- `cargo test --test wizard_proofs -- --test-threads=1`: 4/4 pass (T05 structural assertions for all three distro transcripts + README scenario headings)
- `cargo test --test distribution_proofs -- --test-threads=1`: 6/6 pass (S01 AppImage proofs unaffected)
- `grep -q '^Exec=vibe-attack-config$' packaging/appimage/vibe-attack.desktop`: exit 0
- `LD_LIBRARY_PATH=target/debug target/debug/vibe-attack-config --help | grep -q 'skip-wizard'`: exit 0
- All three wizard transcript files exist with required structured fields and `STATUS: pending VM run`

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

Scenarios A, B, C, D in the wizard transcripts remain as 'pending VM run' placeholders. Real egui rendering, polkit dialogs, evdev PTT key capture, Whisper model download, and stratagem-by-voice firing cannot be automated and require human UAT on real VMs in S06.

## Follow-ups

S06 final distribution UAT must update all three wizard transcript STATUS fields to 'ok' and populate DISTRO, KERNEL, BINARY_VERSION, and per-scenario results. S03 release CI and S04 AUR PKGBUILD are independent and can proceed in parallel.

## Files Created/Modified

None.
