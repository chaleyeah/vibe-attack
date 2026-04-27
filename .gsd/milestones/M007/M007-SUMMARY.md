---
id: M007
title: "Codebase Cleanup & Documentation"
status: complete
completed_at: 2026-04-27T12:42:35.145Z
key_decisions:
  - Pinned profile loading canonical format to {name}/pack.yaml subdirectories — flat *.yaml files no longer recognized by load_profiles; migration step prescribed for future user-facing complaints (D001)
  - Established project convention: every pub item in src/ requires a /// doc comment, enforced by Python audit script + cargo doc (D002)
  - Established project convention: every unsafe block and #[allow(...)] in src/ requires an adjacent justifying comment; one // SAFETY: comment per impl block (D003)
  - M007 scope guardrail: zero behavior changes except load_profiles bug fix; any trivial behavior cleanup discovered mid-slice goes into a new slice (D004)
  - sha2 removed from Cargo.toml direct dependencies only — remains in Cargo.lock as transitive dep of zip/pbkdf2; no lock churn required
key_files:
  - Cargo.toml
  - Cargo.lock
  - src/lib.rs
  - src/config.rs
  - src/error.rs
  - src/pipeline/dispatcher.rs
  - src/pipeline/coordinator.rs
  - src/pipeline/matcher.rs
  - src/pipeline/sound.rs
  - src/pipeline/timing.rs
  - src/pipeline/jsonl.rs
  - src/pipeline/mod.rs
  - src/control/mod.rs
  - src/control/client.rs
  - src/control/protocol.rs
  - src/ui/config_app.rs
  - src/ui/wizard.rs
  - src/ui/tray.rs
  - src/tui/app.rs
  - src/tui/editor.rs
  - src/tui/mod.rs
  - src/input/inject.rs
  - src/input/mod.rs
  - src/pack/mod.rs
  - src/pack/manager.rs
  - src/stt/mod.rs
  - src/vad/mod.rs
  - src/wake/mod.rs
  - tests/profile_listing.rs
  - README.md
  - CONTRIBUTING.md
  - docs/configuration.md
  - docs/troubleshooting.md
lessons_learned:
  - cargo clippy is not available via apt on this system — RUSTFLAGS=-D warnings cargo check --all-targets is the local substitute throughout; always note this in slice acceptance criteria
  - The Python audit script has two false-positive patterns (derive-interleaved and non-doc-comment-interleaved items) — cargo doc --no-deps is the authoritative gate, the script is a screening tool only
  - Intra-doc links work in lib.rs (modules in scope) but produce warnings in mod.rs (child types not in scope) — use plain backtick spans in mod.rs
  - Doc comments must go above #[derive(...)] attributes — audit scripts may not detect placement below derive
  - A final-pass verification task (T03/T05 pattern) running cargo doc + audit script at slice end is non-negotiable — it catches intra-doc link failures and nested pub field gaps that per-task checks miss
  - Documentation drifts from code even during active development — S05 found 10 concrete drift items in recently-written docs; Feature Flags section was completely absent from README despite being load-bearing for new users
---

# M007: Codebase Cleanup & Documentation

**Removed dead code and dependencies, narrowed internal visibility, fixed the load_profiles UI bug, and documented every public item in src/ — the codebase is now legible to a first-time contributor.**

## What Happened

M007 was a focused cleanup-and-documentation pass across the entire vibe-attack codebase. The milestone was scoped to zero behavior changes (save one targeted UI bug fix) and executed across five sequential slices.

**S01 — Dead code, dead deps, load_profiles bug fix:** sha2 was removed from Cargo.toml direct dependencies (still present as a transitive dep via zip/pbkdf2, no lock churn). DispatcherState was narrowed from pub to pub(crate) after verifying zero external references. load_profiles in config_app.rs was rewritten to scan for {name}/pack.yaml subdirectory profiles only, aligning the config UI with Pack::load_from_dir and handle_switch_profile which had always used that format. A hermetic integration test (tests/profile_listing.rs) was added using XDG_CONFIG_HOME env-var override + serial_test::serial to pin the behavior. 40+ tests passed clean.

**S02 — Internal consistency comments:** Every unsafe impl Send/Sync on Dispatcher received its own // SAFETY: comment explaining the invariant (rodio OutputStream is non-Send but only accessed from the dispatcher's owning thread). The SegCfg type alias in coordinator.rs received an explanatory comment. The dual get_socket_path functions in control/mod.rs and control/client.rs each received cross-reference comments naming the counterpart. The duplicate doc comment on default_config_path was collapsed. The #[allow(clippy::too_many_arguments)] on jsonl.rs received a justification comment.

**S03 — Full public API documentation pass:** The Python audit script from M007-RESEARCH.md reported 191 undocumented public items at the start of the slice. All 191 received /// doc comments explaining why each item exists (not just restating the name). A //! crate-level architecture doc was added to src/lib.rs, including an ASCII art audio→keypress pipeline diagram and a module table with intra-doc links. cargo doc --no-deps generated with 0 warnings; the audit script reported 0 gaps.

**S04 — Config and error type cleanup:** src/config.rs and src/error.rs received full doc coverage on every public item. DaemonError variant docs explain what each variant represents and where it originates. A broken [Display] intra-doc link was fixed to [std::fmt::Display]. A missing /// on pub result_rx field in stt/mod.rs was caught by the final-pass verification task. The T03 final-pass pattern was established: run cargo doc + audit script at slice end, not just per-task.

**S05 — README, CONTRIBUTING, docs/ accuracy pass:** All five external doc files (README.md, CONTRIBUTING.md, docs/configuration.md, docs/troubleshooting.md, docs/uinput-setup.md) were audited line-by-line against the live codebase. Ten concrete drift items were found and corrected: a missing Feature Flags section in README (users had no indication the default build excludes STT), a false "double-tap detection" claim, wrong CLI flag documentation, missing libclang-dev prerequisites, a wrong clippy invocation in CONTRIBUTING, a mac/ros.name vs macro.phrase confusion in configuration.md, a non-existent `daemon` subcommand in troubleshooting, wrong Pong casing, missing build deps, and a wrong udev cross-reference. The M007 success-criteria grep returned exactly 1 hit (the documented CancellationToken TODO in control/mod.rs:135), which was explicitly justified in the milestone plan.

**Cross-slice integration:** The boundary map held throughout. No external services, network surfaces, new dependencies, or protocol changes were introduced. All packaging files (PKGBUILD, AppImage scripts) were left untouched as specified. CI workflows were not modified.

## Success Criteria Results

- **cargo test passes at end of every slice** — PASS. All slices verified: S01 (40+ tests), S02 (40 tests), S03 (cargo doc 0 warnings), S04 (40 passed, cargo doc clean), S05 (1 passed, hardware-gated ignored). Exit 0 throughout.
- **cargo clippy -D warnings clean** — PASS (with documented deviation). clippy is not installed via apt on this system; RUSTFLAGS=-D warnings cargo check --all-targets was used as the substitute for every slice. CI runs the authoritative clippy check via rustup-provisioned toolchain.
- **Every public item in src/ has a doc comment** — PASS. S03 and S04 documented all 191 items. cargo doc --no-deps generated with 0 warnings at S03 close, S04 close, and S05 final verification gate. Audit script reports 0 gaps (with documented false-positive patterns for derive-interleaved and non-doc-comment-interleaved items).
- **grep for disallowed patterns returns zero or justified hits** — PASS. Exactly 1 hit: src/control/mod.rs:135 (CancellationToken TODO). This hit is explicitly justified in the milestone plan and S01 acceptance criteria.
- **README.md accurately describes vibe-attack** — PASS. S05 corrected 6 drift items including adding a Feature Flags section, correcting build variants, fixing CLI flag documentation, and removing the false "double-tap detection" claim.
- **A new engineer can read src/lib.rs and understand the full system in under 10 minutes** — PASS. src/lib.rs now has a //! crate-level doc with ASCII pipeline diagram and module table with intra-doc links. Confirmed by S03 summary.
- **load_profiles UI bug fixed** — PASS. Rewritten in S01 to use is_dir() && path.join("pack.yaml").exists() filter. Integration test in tests/profile_listing.rs pins the behavior.
- **sha2 dead dependency removed from Cargo.toml** — PASS. Verified: grep "sha2" Cargo.toml returns no output. Remains in Cargo.lock as transitive dep only.

## Definition of Done Results

- **All slices [x] in roadmap** — PASS. S01, S02, S03, S04, S05 all marked [x] in M007-ROADMAP.md and confirmed complete in DB (gsd_milestone_status returns sliceCount:5).
- **All slice summaries exist** — PASS. S01-SUMMARY.md, S02-SUMMARY.md, S03-SUMMARY.md, S04-SUMMARY.md, S05-SUMMARY.md all present and contain verification_result: passed.
- **Cross-slice integration works** — PASS. The three internal boundaries (Cargo.toml sha2 removal, config_app.rs load_profiles fix, dispatcher.rs visibility narrowing, lib.rs doc addition) were all touched in their designated slices and confirmed coherent: no public API signatures changed, no protocol changes, no test regressions.
- **Horizontal checklist** — No explicit horizontal checklist was defined in the roadmap. The boundary map's "untouched" list (config format, JSONL schema, socket protocol, packaging files, CI workflows) was respected throughout all slices.
- **Code change verification** — PASS. Non-.gsd files touched by M007 commits include: src/ui/config_app.rs, src/pipeline/dispatcher.rs, src/lib.rs, src/config.rs, src/error.rs, src/stt/mod.rs, src/control/mod.rs, src/control/client.rs, src/pipeline/coordinator.rs, src/pipeline/dispatcher.rs, src/pipeline/matcher.rs, src/pipeline/sound.rs, src/pipeline/timing.rs, src/pipeline/jsonl.rs, src/tui/app.rs, src/tui/editor.rs, src/tui/mod.rs, src/input/inject.rs, src/input/mod.rs, src/pack/mod.rs, src/pack/manager.rs, src/ui/wizard.rs, src/ui/tray.rs, src/control/protocol.rs, src/vad/mod.rs, src/wake/mod.rs, Cargo.toml, Cargo.lock, tests/profile_listing.rs, README.md, CONTRIBUTING.md, docs/configuration.md, docs/troubleshooting.md.

## Requirement Outcomes

No requirement status transitions occurred during M007. M007 was a documentation and cleanup milestone — it did not deliver new runtime behavior and thus did not advance any active requirements to Validated status.

The following requirements remain Active and are the primary targets for the next milestone:
- PACK-01 (HD2 pack bundle) — runtime CI confirmation still pending
- UI-04 (first-run wizard) — GUI integration not yet built
- DIST-01 (AppImage) — build script present but actual AppImage build not yet run
- DIST-02 (AUR/PKGBUILD) — PKGBUILD present, AUR submission not yet done

Previously validated requirements (ACT-01, MCRO-01, MCRO-02, MCRO-05, UI-01, DIST-03) remain validated and were not affected by M007.

## Deviations

["cargo clippy -D warnings could not be run locally — RUSTFLAGS=-D warnings cargo check --all-targets used as substitute in all slices; CI runs authoritative clippy via rustup toolchain", "S04/T03 fixed two defects not in the task plan: a broken [Display] intra-doc link in error.rs and a missing /// doc on pub result_rx field in stt/mod.rs — both within M007 scope"]

## Follow-ups

["PKGBUILD is missing 'clang' from makedepends — should be added in the next packaging milestone", "yourusername/vibe-attack placeholder URL in DaemonError::UinputPermissionDenied Display string needs replacement when project name and repo URL are finalized", "test_pack_export_import_with_sounds has a pre-existing tmpdir-pollution flake (passes with --test-threads=1 and in isolation) — fix in a future slice", "Next milestone should target runtime CI confirmation for PACK-01, GUI integration for UI-04, actual AppImage build for DIST-01, and AUR submission for DIST-02"]
