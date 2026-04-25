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

#[test]
fn pkgbuild_file_exists_and_has_required_fields() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let pkgbuild = root.join("packaging/PKGBUILD");
    assert!(pkgbuild.exists(), "packaging/PKGBUILD must exist");
    let contents = std::fs::read_to_string(&pkgbuild).expect("failed to read PKGBUILD");
    assert!(contents.contains("pkgname="), "PKGBUILD missing pkgname=");
    assert!(contents.contains("pkgver="), "PKGBUILD missing pkgver=");
    assert!(contents.contains("url="), "PKGBUILD missing url=");
    assert!(contents.contains("license="), "PKGBUILD missing license=");
}

#[test]
fn desktop_file_exists_and_has_required_keys() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let desktop = root.join("packaging/appimage/hd-linux-voice.desktop");
    assert!(desktop.exists(), "packaging/appimage/hd-linux-voice.desktop must exist");
    let contents = std::fs::read_to_string(&desktop).expect("failed to read .desktop file");
    assert!(contents.contains("Name="), ".desktop missing Name=");
    assert!(contents.contains("Exec="), ".desktop missing Exec=");
    assert!(contents.contains("Type="), ".desktop missing Type=");
}

#[test]
fn appimage_build_script_exists() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let build_sh = root.join("packaging/appimage/build.sh");
    assert!(build_sh.exists(), "packaging/appimage/build.sh must exist");
    let contents = std::fs::read_to_string(&build_sh).expect("failed to read build.sh");
    assert!(!contents.is_empty(), "build.sh must not be empty");
}

#[test]
fn appimage_build_script_has_shebang() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let build_sh = root.join("packaging/appimage/build.sh");
    let contents = std::fs::read_to_string(&build_sh).expect("failed to read build.sh");
    let first_line = contents.lines().next().expect("build.sh must not be empty");
    assert!(
        first_line.starts_with("#!"),
        "build.sh first line must be a shebang, got: {:?}",
        first_line
    );
}

#[test]
fn daemon_default_features_exclude_gui() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = root.join("Cargo.toml");
    let contents = std::fs::read_to_string(&cargo_toml).expect("failed to read Cargo.toml");
    // Parse [features] section to extract the default = [...] line.
    let in_features = &mut false;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed == "[features]" {
            *in_features = true;
            continue;
        }
        if *in_features {
            // Stop at next section header.
            if trimmed.starts_with('[') {
                break;
            }
            if trimmed.starts_with("default") {
                assert!(
                    !trimmed.contains("gui"),
                    "default features must not include 'gui', found: {trimmed}"
                );
                return;
            }
        }
    }
    // If we reach here, the default line was found and does not include "gui",
    // or the [features] section had no explicit default (which is also fine).
}
