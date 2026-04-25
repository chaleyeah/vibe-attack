---
id: T01
parent: S05
milestone: M001
key_files:
  - src/ui/mod.rs
  - src/ui/first_run.rs
  - src/ui/config_app.rs
  - src/lib.rs
  - tests/ui_distribution.rs
key_decisions:
  - Pure-logic structs with no egui/eframe dependency, matching the constraint that these can be tested without a display server
  - add_log_line uses Vec::remove(0) to drop oldest — simple and correct for a capped 100-entry log
  - SetupStep enum uses Copy+Clone+PartialEq+Eq to support Vec::contains() in tests without consuming values
duration: 
verification_result: passed
completed_at: 2026-04-25T19:42:31.882Z
blocker_discovered: false
---

# T01: Added src/ui/ module with pure-logic FirstRunState, SetupStep, and ConfigApp structs plus 11 tests in tests/ui_distribution.rs

**Added src/ui/ module with pure-logic FirstRunState, SetupStep, and ConfigApp structs plus 11 tests in tests/ui_distribution.rs**

## What Happened

Created three new files and updated two existing ones:

**src/ui/first_run.rs** — Defines `SetupStep` enum (CreateConfig, InstallModel, SetupUinput, ConfigurePtt in wizard order) and `FirstRunState` struct. `from_checks(config_exists, model_installed, uinput_accessible, ptt_configured)` stores probe results; `is_setup_complete()` returns true only when all four are true; `steps_remaining()` builds an ordered Vec of incomplete steps; `first_incomplete_step()` returns the first of these or None. No egui/eframe imports — pure Rust.

**src/ui/config_app.rs** — Defines `MAX_LOG_LINES = 100` and `ConfigApp` struct with `profiles: Vec<String>`, `active_profile: Option<String>`, `log_lines: Vec<String>`, `mic_level: f32`. `new()` constructs an empty state; `profile_count()` mirrors `profiles.len()`; `add_log_line()` appends and drops the oldest entry when `log_lines.len() >= MAX_LOG_LINES`. No egui/eframe imports — pure Rust.

**src/ui/mod.rs** — Re-exports both submodules as `pub mod config_app; pub mod first_run;`.

**src/lib.rs** — Appended `pub mod ui;` after the existing `pub mod tui;` line.

**tests/ui_distribution.rs** — 11 pure-logic integration tests using `hd_linux_voice::ui::first_run::{FirstRunState, SetupStep}` and `hd_linux_voice::ui::config_app::{ConfigApp, MAX_LOG_LINES}`. Tests cover: all-pass complete, each individual missing check, step ordering (CreateConfig before InstallModel), fresh-install returns 4 steps, first_incomplete_step returns None when done, profile_count tracks profiles.len(), add_log_line grows the vec, log capped at MAX_LOG_LINES with oldest dropped.

Static verification confirmed: `grep -c '#[test]'` returned 11; no egui/eframe references in src/ui/; all method signatures match test call sites; lib.rs correctly includes `pub mod ui;`.

## Verification

Static verification (cargo test is blocked in auto-mode per MEM004):
1. Counted #[test] annotations: 11 (meets ≥11 requirement).
2. Grepped src/ui/first_run.rs and src/ui/config_app.rs for egui/eframe: no matches — constraint satisfied.
3. Verified all method signatures called in tests exist in source: from_checks, is_setup_complete, steps_remaining, first_incomplete_step, new, profile_count, add_log_line, MAX_LOG_LINES constant.
4. Confirmed import paths match spec: hd_linux_voice::ui::first_run::{FirstRunState, SetupStep} and hd_linux_voice::ui::config_app::{ConfigApp, MAX_LOG_LINES}.
5. Verified src/lib.rs now has pub mod ui; on line 12.
6. Confirmed SetupStep ordering in steps_remaining(): CreateConfig → InstallModel → SetupUinput → ConfigurePtt matches task plan constraint.
7. Confirmed add_log_line cap logic: drops oldest via remove(0) when len >= MAX_LOG_LINES, then pushes; test assertions on len and last element are algebraically correct for 110 inserts.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `grep -c '#\[test\]' tests/ui_distribution.rs` | 0 | ✅ pass — 11 tests found | 20ms |
| 2 | `grep -n 'egui|eframe' src/ui/first_run.rs src/ui/config_app.rs` | 1 | ✅ pass — no egui/eframe imports (exit 1 = no matches) | 15ms |
| 3 | `grep -n 'pub mod ui' src/lib.rs` | 0 | ✅ pass — pub mod ui; present in lib.rs | 10ms |
| 4 | `grep -n 'MAX_LOG_LINES\|add_log_line\|from_checks\|steps_remaining\|first_incomplete_step\|is_setup_complete\|profile_count' src/ui/first_run.rs src/ui/config_app.rs` | 0 | ✅ pass — all required methods and constants defined | 15ms |

## Deviations

none

## Known Issues

Cargo build/test cannot be confirmed in auto-mode (MEM004 — cargo test requires user approval). Runtime confirmation must be done in CI or a manual run after auto-mode completes.

## Files Created/Modified

- `src/ui/mod.rs`
- `src/ui/first_run.rs`
- `src/ui/config_app.rs`
- `src/lib.rs`
- `tests/ui_distribution.rs`
