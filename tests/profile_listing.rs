// Integration test for load_profiles canonical subdirectory format.
//
// M007/S01 fix: load_profiles was changed to scan for {name}/pack.yaml
// subdirectories instead of flat *.yaml files, aligning it with
// Pack::load_from_dir and handle_switch_profile. This test pins that contract
// so any regression to flat-file scanning fails loudly.

use serial_test::serial;

/// Asserts load_profiles returns only subdirectory-format profiles and ignores
/// both flat .yaml files and subdirectories that lack a pack.yaml.
#[test]
#[serial]
fn load_profiles_only_returns_subdirectory_profiles() {
    let tmp = tempfile::tempdir().unwrap();
    let profiles_dir = tmp.path().join("vibe-attack/profiles");
    std::fs::create_dir_all(&profiles_dir).unwrap();

    // (a) Valid profile: subdirectory containing pack.yaml — must be returned.
    let good = profiles_dir.join("good_profile");
    std::fs::create_dir_all(&good).unwrap();
    std::fs::write(good.join("pack.yaml"), b"").unwrap();

    // (b) Flat file at profiles root — must be ignored.
    std::fs::write(profiles_dir.join("flat_profile.yaml"), b"").unwrap();

    // (c) Subdirectory with no pack.yaml — must be ignored.
    std::fs::create_dir_all(profiles_dir.join("no_pack")).unwrap();

    unsafe { std::env::set_var("XDG_CONFIG_HOME", tmp.path()); }
    let profiles = vibe_attack::ui::config_app::load_profiles();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME"); }

    assert_eq!(
        profiles,
        vec!["good_profile"],
        "expected exactly ['good_profile']; flat files and dirs without pack.yaml must be excluded"
    );
}
