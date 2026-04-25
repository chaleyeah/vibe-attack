---
estimated_steps: 19
estimated_files: 5
skills_used: []
---

# T01: Create src/ui/ module with FirstRunState and ConfigApp pure-logic structs plus tests

Create the `src/ui/` module tree with two pure-logic state files: `first_run.rs` (FirstRunState struct with from_checks constructor, is_setup_complete, steps_remaining, first_incomplete_step methods; SetupStep enum with CreateConfig/InstallModel/SetupUinput/ConfigurePtt variants) and `config_app.rs` (ConfigApp struct with profiles vec, active_profile, log_lines, mic_level fields; profile_count, add_log_line, new methods; MAX_LOG_LINES cap). Add `pub mod ui;` to `src/lib.rs`. Then write `tests/ui_distribution.rs` with 11 pure-logic tests:

1. `first_run_complete_when_all_checks_pass` — from_checks(true,true,true,true).is_setup_complete() == true
2. `first_run_incomplete_when_config_missing` — steps_remaining contains CreateConfig
3. `first_run_incomplete_when_model_missing` — steps_remaining contains InstallModel
4. `first_run_incomplete_when_uinput_inaccessible` — steps_remaining contains SetupUinput
5. `first_run_incomplete_when_ptt_missing` — steps_remaining contains ConfigurePtt
6. `first_run_step_ordering_config_before_model` — wizard returns CreateConfig before InstallModel
7. `first_run_all_steps_when_fresh_install` — from_checks(false,false,false,false) returns 4 steps
8. `first_incomplete_step_returns_none_when_done` — first_incomplete_step() is None when complete
9. `config_app_profile_count_reflects_profiles` — profile_count() matches profiles.len()
10. `config_app_add_log_line_grows_log` — add_log_line increments log_lines.len()
11. `config_app_log_capped_at_max` — add_log_line doesn't grow past MAX_LOG_LINES

IMPORTANT CONSTRAINTS:
- `src/ui/first_run.rs` and `src/ui/config_app.rs` must NOT import egui/eframe — they are pure Rust structs with no GUI dependencies
- `src/ui/mod.rs` re-exports both modules publicly
- `src/lib.rs` adds `pub mod ui;` at the end of the existing module list
- Tests use `use hd_linux_voice::ui::first_run::{FirstRunState, SetupStep};` and `use hd_linux_voice::ui::config_app::ConfigApp;`
- ConfigApp::add_log_line must cap at MAX_LOG_LINES (100) by dropping oldest lines
- SetupStep ordering in steps_remaining: CreateConfig, InstallModel, SetupUinput, ConfigurePtt (this order matches the first-run wizard flow)

## Inputs

- `src/lib.rs`
- `src/config.rs`
- `src/control/protocol.rs`
- `src/pack/manager.rs`

## Expected Output

- `src/ui/mod.rs`
- `src/ui/first_run.rs`
- `src/ui/config_app.rs`
- `src/lib.rs`
- `tests/ui_distribution.rs`

## Verification

cargo test --test ui_distribution 2>&1 | grep -E 'test result|running' && test $(grep -c '#\[test\]' tests/ui_distribution.rs) -ge 11
