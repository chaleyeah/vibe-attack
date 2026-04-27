---
phase: M007
phase_name: Codebase Cleanup & Documentation
project: hd-linux-voice
generated: 2026-04-27T12:45:00Z
counts:
  decisions: 5
  lessons: 6
  patterns: 7
  surprises: 3
missing_artifacts: []
---

# M007 Learnings

### Decisions

- **Profile loading canonical format pinned to {name}/pack.yaml subdirectories.** load_profiles in config_app.rs was rewritten to filter by is_dir() && path.join("pack.yaml").exists(), aligning the config UI with Pack::load_from_dir and handle_switch_profile. Flat *.yaml files are no longer recognized; a migration step is prescribed for future user-facing complaints.
  Source: S01-SUMMARY.md/Key decisions

- **DispatcherState visibility narrowed from pub to pub(crate).** Verified zero external references before narrowing; impl Default required no pub annotation. This is the correct scope for state that should never be exposed outside the pipeline module.
  Source: S01-SUMMARY.md/Key decisions

- **Every public item in src/ requires a /// doc comment (project convention, enforced by audit script).** The Python audit script from M007-RESEARCH.md is the mechanical enforcement gate, with cargo doc --no-deps as the authoritative tie-breaker when the script reports false positives.
  Source: D002/Decisions; S03-SUMMARY.md/Key decisions

- **Every unsafe block and #[allow(...)] in src/ requires an adjacent justifying comment.** Each unsafe impl Send/Sync on Dispatcher gets its own // SAFETY: comment (not shared), because the invariants for Send and Sync are subtly different. Each #[allow(...)] gets a one-line justification immediately above it.
  Source: D003/Decisions; S02-SUMMARY.md/Key decisions

- **M007 explicitly excludes behavior changes except the load_profiles bug fix.** Scope guardrail: no new features, no error-handling refactors, no API changes, no CI changes. Visibility narrowing (pub→pub(crate)) is allowed when verifiably internal-only. Any trivial behavior cleanup discovered mid-slice goes into a new slice with explicit success criteria.
  Source: D004/Decisions; M007-ROADMAP.md/Boundary Map

### Lessons

- **cargo clippy is not installed via apt in this system Rust environment.** RUSTFLAGS=-D warnings cargo check --all-targets was used as the substitute throughout all slices. CI runs the authoritative clippy check via rustup-provisioned toolchain. Any future milestone planning on this machine should account for this — don't write slice acceptance criteria that require local clippy without noting the substitute.
  Source: S01-SUMMARY.md/Deviations; S02-SUMMARY.md/Known limitations

- **The Python undocumented-pub-item audit script has two known false-positive patterns:** (1) enums with #[derive(...)] between the doc comment and the pub keyword; (2) functions with a non-doc // comment between the /// block and the pub fn line. cargo doc --no-deps with 0 warnings is the authoritative gate — the script is a screening tool, not the final arbiter.
  Source: S03-SUMMARY.md/Known limitations; S04-SUMMARY.md/Patterns established

- **Intra-doc links work in lib.rs but fail in mod.rs when re-exporting child types.** In lib.rs, top-level modules are in scope so [`module`] links resolve. In mod.rs files, child module types are not in scope, causing unresolved-link warnings under cargo doc — use plain backtick spans instead.
  Source: S03-SUMMARY.md/Key decisions

- **Doc comments must go above #[derive(...)] attributes, not below them.** Both rustdoc and cargo doc handle the above-derive placement correctly; the audit script may not detect the gap when derive comes between the comment and the pub keyword.
  Source: S03-SUMMARY.md/Patterns established

- **DaemonError::Config variant preserved though currently unused.** Config errors propagate as anyhow::Error today; the typed variant is retained for future tightening without removing API surface. A placeholder hostname in UinputPermissionDenied Display (yourusername/vibe-attack) was left unchanged — it is user-facing behavior, not documentation, so it is out of scope for a doc-only milestone.
  Source: S04-SUMMARY.md/Key decisions

- **Doc audit must cross-reference every external prose claim against Cargo.toml [features], src/main.rs Commands enum, control/protocol.rs ControlResponse variants, and src/error.rs Display impls.** Prose summaries drift faster than code. S05 found 10 concrete drift items across 5 doc files through this method — none would have been caught by reading docs in isolation.
  Source: S05-SUMMARY.md/Patterns established; S05-SUMMARY.md/What Happened

### Patterns

- **Profile loading convention: {name}/pack.yaml subdirectory format.** All three surfaces (load_profiles, handle_switch_profile, Pack::load_from_dir) now agree. Any future profile loading code must filter by is_dir() && path.join("pack.yaml").exists() — not by *.yaml glob.
  Source: S01-SUMMARY.md/Patterns established

- **Integration tests for XDG-dependent functions use XDG_CONFIG_HOME env-var override + serial_test::serial.** No API changes needed; the env-var redirect points load_profiles at a tempdir. serial_test::serial prevents env-var races when multiple tests run in parallel.
  Source: S01-SUMMARY.md/Patterns established

- **Full test suite must run with --test-threads=1 locally to avoid test_pack_export_import_with_sounds tmpdir-pollution flake.** This is a pre-existing issue (not introduced in M007) and passes cleanly in isolation. CI handles it via its own thread management.
  Source: S01-SUMMARY.md/Known limitations

- **// SAFETY: comment per unsafe impl block, one per block, placed directly above each impl.** Use two distinct comments for two unsafe impls even if they guard the same struct — invariants may differ between Send and Sync. Cross-reference comments on intentionally duplicated private functions should name the counterpart file path for navigability.
  Source: S02-SUMMARY.md/Patterns established

- **spawn_pipeline thread-topology ASCII diagram pattern.** For multi-thread orchestration functions, an ASCII diagram in the doc comment showing parallel OS thread structure is more scannable than prose. Established in coordinator.rs for spawn_pipeline.
  Source: S03-SUMMARY.md/Key decisions

- **Final-pass verification task (T03/T05 pattern) catches defects missed by per-task checks.** Intra-doc link resolution failures surface only under cargo doc; pub field docs on non-top-level structs require the audit script to scan all nesting depths. Running the full gate (cargo doc, cargo check, audit script) at the slice end is non-optional.
  Source: S04-SUMMARY.md/Patterns established

- **Feature flags must be documented in README at the time they are added to Cargo.toml [features], not retroactively.** S05 added the Feature Flags section to README after discovering it was completely absent — users had no indication the default build ships without STT. Add-feature and add-to-README should be the same commit.
  Source: S05-SUMMARY.md/Patterns established

### Surprises

- **The config UI (load_profiles) and the config switcher (handle_switch_profile) had silently diverged on what constitutes a valid profile directory.** The UI listed flat *.yaml files that the switcher could never load. This went undetected because there were no integration tests asserting the UI and switcher agreed on the profile format. The fix required rewriting load_profiles and adding a hermetic integration test.
  Source: S01-SUMMARY.md/What Happened

- **The Python audit script reported 191 undocumented public items at the start of S03.** The codebase had grown substantially with almost no /// coverage. Documenting all 191 items took the full S03 slice — the estimate was not off, but the volume confirmed that documentation debt compounds silently and is expensive to pay back in bulk.
  Source: S03-SUMMARY.md/One-liner

- **S05 found 10 concrete drift items across 5 doc files despite the codebase having been recently written.** README contained a Feature Flags section that didn't exist at all; troubleshooting.md referenced a `daemon` subcommand that doesn't exist; ControlResponse::Pong casing was wrong; libclang-dev was missing from build prerequisites. Documentation drifts from code even during active development, not just in legacy codebases.
  Source: S05-SUMMARY.md/What Happened
