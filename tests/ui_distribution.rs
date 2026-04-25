use hd_linux_voice::ui::config_app::{ConfigApp, MAX_LOG_LINES};
use hd_linux_voice::ui::first_run::{FirstRunState, SetupStep};

#[test]
fn first_run_complete_when_all_checks_pass() {
    let state = FirstRunState::from_checks(true, true, true, true);
    assert!(state.is_setup_complete());
}

#[test]
fn first_run_incomplete_when_config_missing() {
    let state = FirstRunState::from_checks(false, true, true, true);
    assert!(!state.is_setup_complete());
    assert!(state.steps_remaining().contains(&SetupStep::CreateConfig));
}

#[test]
fn first_run_incomplete_when_model_missing() {
    let state = FirstRunState::from_checks(true, false, true, true);
    assert!(!state.is_setup_complete());
    assert!(state.steps_remaining().contains(&SetupStep::InstallModel));
}

#[test]
fn first_run_incomplete_when_uinput_inaccessible() {
    let state = FirstRunState::from_checks(true, true, false, true);
    assert!(!state.is_setup_complete());
    assert!(state.steps_remaining().contains(&SetupStep::SetupUinput));
}

#[test]
fn first_run_incomplete_when_ptt_missing() {
    let state = FirstRunState::from_checks(true, true, true, false);
    assert!(!state.is_setup_complete());
    assert!(state.steps_remaining().contains(&SetupStep::ConfigurePtt));
}

#[test]
fn first_run_step_ordering_config_before_model() {
    let state = FirstRunState::from_checks(false, false, true, true);
    let steps = state.steps_remaining();
    let config_pos = steps.iter().position(|s| *s == SetupStep::CreateConfig);
    let model_pos = steps.iter().position(|s| *s == SetupStep::InstallModel);
    assert!(config_pos.is_some() && model_pos.is_some());
    assert!(config_pos.unwrap() < model_pos.unwrap());
}

#[test]
fn first_run_all_steps_when_fresh_install() {
    let state = FirstRunState::from_checks(false, false, false, false);
    assert_eq!(state.steps_remaining().len(), 4);
}

#[test]
fn first_incomplete_step_returns_none_when_done() {
    let state = FirstRunState::from_checks(true, true, true, true);
    assert!(state.first_incomplete_step().is_none());
}

#[test]
fn config_app_profile_count_reflects_profiles() {
    let mut app = ConfigApp::new();
    assert_eq!(app.profile_count(), 0);
    app.profiles.push("default".to_string());
    app.profiles.push("gaming".to_string());
    assert_eq!(app.profile_count(), app.profiles.len());
    assert_eq!(app.profile_count(), 2);
}

#[test]
fn config_app_add_log_line_grows_log() {
    let mut app = ConfigApp::new();
    assert_eq!(app.log_lines.len(), 0);
    app.add_log_line("first entry".to_string());
    assert_eq!(app.log_lines.len(), 1);
    app.add_log_line("second entry".to_string());
    assert_eq!(app.log_lines.len(), 2);
}

#[test]
fn config_app_log_capped_at_max() {
    let mut app = ConfigApp::new();
    for i in 0..MAX_LOG_LINES + 10 {
        app.add_log_line(format!("line {i}"));
    }
    assert_eq!(app.log_lines.len(), MAX_LOG_LINES);
    // The oldest lines should have been dropped; the last line is still present.
    assert!(app.log_lines.last().unwrap().contains(&format!("{}", MAX_LOG_LINES + 9)));
}
