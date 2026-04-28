---
phase: M009
phase_name: Pack UX — Editor, Import/Export, Full HD2 Coverage
project: hd-linux-voice
generated: "2026-04-28T03:45:00Z"
counts:
  decisions: 9
  lessons: 6
  patterns: 8
  surprises: 3
missing_artifacts: []
---

### Decisions

- **HD2 key sequences use KEY_UP/KEY_DOWN/KEY_LEFT/KEY_RIGHT evdev names exclusively — no WASD mixing.** Chosen for semantic clarity and consistency with evdev input layer; WASD names are UI labels, not kernel key codes.
  Source: S01-SUMMARY.md/Key decisions

- **PackEditor wraps Pack as a mutable state machine with typed CRUD methods returning Result<()>.** Chose typed API over direct Pack field mutation to surface precondition errors at the API boundary rather than silently producing invalid state.
  Source: S02-SUMMARY.md/Key decisions

- **MacroUpdates uses Option<Option<T>> for optional fields to distinguish leave-unchanged (None) from clear (Some(None)).** Standard Option<T> cannot encode three states (set, clear, no-op) without an enum; the double-Option pattern is idiomatic Rust for partial-update builders.
  Source: S02-SUMMARY.md/Key decisions

- **pub mod pack_editor declared without #[cfg(feature="gui")] in mod.rs (wizard pattern, not tray pattern).** Pure-logic helpers parse_key_sequence and build_macro_config_from_form must compile under the default build for unit testability; placing them behind the gui gate would require cargo test --features gui for every pack-logic test.
  Source: S03-SUMMARY.md/Key decisions

- **rfd 0.17 added as optional dep under gui feature only.** Keeps the default headless daemon build free of file-dialog backends (GTK, xdg-portal) that require display-server libraries at link time.
  Source: S04-SUMMARY.md/Key decisions

- **Pack::import_to accepts the parent profiles dir (not the pack subdir); function appends pack.name internally.** Consistent with the established Pack::import contract; callers do not need to know the pack's internal directory name before the archive is inspected.
  Source: S04-SUMMARY.md/Key decisions

- **score=1.0 in Dispatcher::fire_named marks direct control-plane triggers vs fuzzy phrase-matched scores.** JSONL consumers (dashboards, tests) can distinguish "fired by voice" from "fired by control plane" without a separate event type field.
  Source: S05-SUMMARY.md/Key decisions

- **block_in_place requires multi_thread Tokio flavor — TestMacro integration tests use #[tokio::test(flavor="multi_thread", worker_threads=2)].** block_in_place panics on a single-thread executor; the handler calls synchronous disk I/O inside the async socket handler, making this unavoidable without an async rewrite of the dispatcher.
  Source: S05-SUMMARY.md/Key decisions

- **Catch-all match arm removed (not kept) when TestMacro became the last unhandled variant.** unreachable_patterns is a compile error under -D warnings; keeping an unreachable arm breaks the build gate.
  Source: S05-SUMMARY.md/Deviations

---

### Lessons

- **cargo clippy is not installed in this build environment; zero-warning compliance is verified via rustc output from cargo build.** Both `cargo build` (default) and `cargo build --features gui` must be run and must complete with `Finished` and no `warning:` lines as the clippy substitute. Record as MEM038 convention.
  Source: S03-SUMMARY.md/Known limitations

- **rfd dialog trigger paths cannot be driven headlessly — Import/Export dialog integration requires manual smoke testing.** The rfd library opens a native OS file picker; there is no headless/mock mode. Automated tests can cover the backend logic (Pack::import_to, Pack::export) but cannot exercise the button → picker → path flow.
  Source: S04-SUMMARY.md/Known limitations

- **TestMacro integration tests must declare flavor="multi_thread" due to block_in_place inside the handler.** This was not documented in the original task plan and caused test failures during S05 implementation. Any future control-plane handler that calls synchronous blocking ops inside an async context must declare the multi-thread flavor.
  Source: S05-SUMMARY.md/Deviations

- **Catch-all match arms on enums with -D warnings must be removed when all variants are handled — not kept as documentation.** The compiler treats an unreachable arm as unreachable_patterns, which is an error under -D warnings. Future protocol additions will surface as compile errors, which is the correct behavior.
  Source: S05-SUMMARY.md/Deviations

- **UAT scripts for GUI-heavy milestones must combine artifact-driven automated evidence with human-experience manual scenarios.** Automated cargo test output proves logic correctness but cannot validate egui rendering, file picker UX, countdown animation, or audio input responsiveness. Neither coverage type alone is sufficient.
  Source: S06-SUMMARY.md/Patterns established

- **HD2 stratagem key sequence accuracy has not been verified against live gameplay.** Sequences were sourced from the canonical community stratagem reference. The Spear in particular (↓↓↑↓↓) has some community disagreement. Live validation is deferred to a future milestone or community contribution.
  Source: S01-SUMMARY.md/Known limitations

---

### Patterns

- **Hermetic pack coverage tests: load from repo fixture path, assert by HashSet difference for clear failure messages, no env mutation, no #[serial] needed.** Using Path::new("profiles/hd2") with no XDG_CONFIG_HOME mutation makes the test portable and parallelizable; HashSet difference names the absent categories explicitly on failure.
  Source: S01-SUMMARY.md/Patterns established

- **Integration test style: vibe_attack::pack::Pack import, .expect() for fixture loads (not anyhow Result), module-level //! doc comment per D002.** Consistent style across all pack integration tests makes the corpus greppable and reviewable without per-file style decisions.
  Source: S01-SUMMARY.md/Patterns established

- **PackEditor validate-then-mutate order for all multi-step Vec operations ensures atomicity.** All preconditions (bounds, name uniqueness, not-same-name) are checked before any mutation begins, so a failed precondition leaves the Pack unchanged.
  Source: S02-SUMMARY.md/Patterns established

- **clone Vec<String> of names before egui closures to avoid simultaneous mutable borrow conflicts.** Mirrors the app.device_names.clone() precedent from the config window. Required whenever an egui closure iterates a name list while the UI also holds a mutable reference to the state struct.
  Source: S03-SUMMARY.md/Patterns established

- **last_error: Option<String> on UI state structs for synchronous inline failure visibility without modals.** Set on any operation that may fail (Save, Test); cleared on the next successful operation. Displayed as a red label inline in the panel rather than a modal dialog.
  Source: S03-SUMMARY.md/Patterns established

- **Two-click inline confirmation for destructive egui actions via pending_* bool flag pattern.** First click sets pending_remove_macro = true; second click (now labeled Confirm Remove) executes the destructive action. Avoids modal state machine complexity while preventing accidental data loss.
  Source: S03-SUMMARY.md/Patterns established

- **Hermetic pack round-trip tests use Pack::import_to + tempfile::tempdir() — never XDG_CONFIG_HOME mutation, no #[serial] needed.** The hermetic import_to API accepts an explicit dest_dir, eliminating any shared-directory race condition in parallel test runs.
  Source: S04-SUMMARY.md/Patterns established

- **UI safety countdown: pending_test: Option<(String, Instant)> with Instant polling, no sleep, request_repaint_after(50ms).** The eframe event loop drives the countdown via elapsed() checks each repaint; request_repaint_after(50ms) ensures smooth visual updates without blocking the UI thread.
  Source: S05-SUMMARY.md/Patterns established

---

### Surprises

- **pack_test_export_import_with_sounds flakes under parallel test execution due to a shared /tmp path.** Fixed by running cargo test --test-threads=1 for full suite runs (MEM005). The flake only manifests when two test threads write to the same tempdir path simultaneously; hermetic tests using tempfile::tempdir() are immune.
  Source: S06-SUMMARY.md/Key decisions

- **The egui pack editor required a daemon_running: bool parameter to be threaded into show_pack_editor to gate the Test button.** The original plan did not anticipate this parameter; it was added during S05 implementation to prevent the Test button from being clickable when no daemon is running. The bool is polled from the existing daemon-status infrastructure.
  Source: S05-SUMMARY.md/Key decisions

- **The rfd 0.17 import confirmed that async rfd is not needed — synchronous dialog calls on the egui frame thread are the ecosystem convention.** The rfd docs note that async is available but the sync API is recommended for egui integrations. This avoided adding a tokio spawn for dialog calls, keeping the import/export code straightforward.
  Source: S04-SUMMARY.md/Key decisions
