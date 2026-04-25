/// Integration tests proving the pack-system HD2 bundle works end-to-end.
///
/// Covers: Pack serialisation round-trip, flatten, export/import (ZIP), ProfileManager
/// persistence, and a realistic Helldivers-2 stratagem macro-pack fixture that exercises
/// every field of MacroConfig (phrase, if_flag, set_flag, sound, keys with dwell/gap
/// overrides).  All tests are hermetic — no XDG writes, no network, no model files.

use std::io::Write;

use vibe_attack::{
    config::{KeyAction, MacroConfig},
    pack::{Category, Pack},
    pack::manager::ProfileManager,
};

// ---------------------------------------------------------------------------
// Fixture helpers
// ---------------------------------------------------------------------------

fn key(name: &str) -> KeyAction {
    KeyAction { key: name.to_string(), dwell_ms: None, gap_ms: None }
}

fn key_timed(name: &str, dwell_ms: u64, gap_ms: u64) -> KeyAction {
    KeyAction { key: name.to_string(), dwell_ms: Some(dwell_ms), gap_ms: Some(gap_ms) }
}

fn macro_simple(name: &str, phrase: &str, keys: Vec<KeyAction>) -> MacroConfig {
    MacroConfig {
        name: name.to_string(),
        phrase: Some(phrase.to_string()),
        if_flag: None,
        set_flag: None,
        sound: None,
        keys,
    }
}

fn macro_with_flags(
    name: &str,
    phrase: &str,
    if_flag: Option<&str>,
    set_flag: Option<&str>,
    keys: Vec<KeyAction>,
) -> MacroConfig {
    MacroConfig {
        name: name.to_string(),
        phrase: Some(phrase.to_string()),
        if_flag: if_flag.map(str::to_string),
        set_flag: set_flag.map(str::to_string),
        sound: None,
        keys,
    }
}

/// Build a realistic Helldivers 2 stratagem pack with multiple categories.
/// Stratagem key sequences: W=up, S=down, A=left, D=right.
fn hd2_pack() -> Pack {
    let stratagems = Category {
        name: "Stratagems".to_string(),
        macros: vec![
            macro_simple(
                "Reinforce",
                "reinforce",
                vec![
                    key("KEY_W"), key("KEY_S"), key("KEY_D"),
                    key("KEY_W"), key("KEY_A"),
                ],
            ),
            macro_simple(
                "Resupply",
                "resupply",
                vec![
                    key("KEY_S"), key("KEY_S"), key("KEY_W"), key("KEY_D"),
                ],
            ),
            macro_simple(
                "Eagle Airstrike",
                "eagle airstrike",
                vec![
                    key("KEY_W"), key("KEY_D"), key("KEY_S"), key("KEY_D"),
                ],
            ),
            macro_simple(
                "Orbital Laser",
                "orbital laser",
                vec![
                    key("KEY_D"), key("KEY_W"), key("KEY_D"), key("KEY_S"),
                    key("KEY_D"),
                ],
            ),
            macro_simple(
                "SOS Beacon",
                "sos beacon",
                vec![
                    key("KEY_W"), key("KEY_S"), key("KEY_D"), key("KEY_W"),
                ],
            ),
        ],
    };

    let support_weapons = Category {
        name: "Support Weapons".to_string(),
        macros: vec![
            macro_simple(
                "Anti-Tank Mines",
                "anti tank mines",
                vec![
                    key("KEY_S"), key("KEY_W"), key("KEY_D"), key("KEY_A"),
                ],
            ),
            macro_simple(
                "Machine Gun Sentry",
                "machine gun sentry",
                vec![
                    key("KEY_S"), key("KEY_W"), key("KEY_D"), key("KEY_W"),
                    key("KEY_A"),
                ],
            ),
        ],
    };

    let flags = Category {
        name: "Flag-gated Stratagems".to_string(),
        macros: vec![
            macro_with_flags(
                "Hellbomb Arm",
                "arm hellbomb",
                Some("hellbomb_present"),
                Some("hellbomb_armed"),
                vec![key_timed("KEY_E", 200, 100)],
            ),
            macro_with_flags(
                "Clear Hellbomb Flag",
                "hellbomb gone",
                None,
                Some("hellbomb_armed"),
                vec![],
            ),
        ],
    };

    Pack {
        name: "Helldivers 2".to_string(),
        author: Some("community".to_string()),
        categories: vec![stratagems, support_weapons, flags],
    }
}

// ---------------------------------------------------------------------------
// Core Pack tests
// ---------------------------------------------------------------------------

#[test]
fn pack_round_trips_yaml() {
    let dir = tempfile::tempdir().unwrap();
    let pack = hd2_pack();

    pack.save_to_dir(dir.path()).expect("save must succeed");

    let yaml_path = dir.path().join("pack.yaml");
    assert!(yaml_path.exists(), "pack.yaml must be written");

    let loaded = Pack::load_from_dir(dir.path()).expect("load must succeed");
    assert_eq!(loaded.name, "Helldivers 2");
    assert_eq!(loaded.author, Some("community".to_string()));
    assert_eq!(loaded.categories.len(), 3);
}

#[test]
fn pack_flatten_yields_all_macros() {
    let pack = hd2_pack();
    let flat = pack.flatten();
    // 5 stratagems + 2 support weapons + 2 flag-gated = 9
    assert_eq!(flat.len(), 9, "flatten must include all macros across categories");

    let names: Vec<&str> = flat.iter().map(|m| m.name.as_str()).collect();
    assert!(names.contains(&"Reinforce"));
    assert!(names.contains(&"Orbital Laser"));
    assert!(names.contains(&"Hellbomb Arm"));
}

#[test]
fn pack_flatten_preserves_macro_fields() {
    let pack = hd2_pack();
    let flat = pack.flatten();

    let reinforce = flat.iter().find(|m| m.name == "Reinforce").unwrap();
    assert_eq!(reinforce.phrase, Some("reinforce".to_string()));
    assert_eq!(reinforce.keys.len(), 5);
    assert_eq!(reinforce.keys[0].key, "KEY_W");

    let hellbomb = flat.iter().find(|m| m.name == "Hellbomb Arm").unwrap();
    assert_eq!(hellbomb.if_flag, Some("hellbomb_present".to_string()));
    assert_eq!(hellbomb.set_flag, Some("hellbomb_armed".to_string()));
    assert_eq!(hellbomb.keys[0].dwell_ms, Some(200));
    assert_eq!(hellbomb.keys[0].gap_ms, Some(100));
}

#[test]
fn pack_category_names_preserved() {
    let dir = tempfile::tempdir().unwrap();
    let pack = hd2_pack();
    pack.save_to_dir(dir.path()).unwrap();

    let loaded = Pack::load_from_dir(dir.path()).unwrap();
    let cat_names: Vec<&str> = loaded.categories.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(cat_names, ["Stratagems", "Support Weapons", "Flag-gated Stratagems"]);
}

#[test]
fn pack_macro_key_sequences_survive_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let pack = hd2_pack();
    pack.save_to_dir(dir.path()).unwrap();

    let loaded = Pack::load_from_dir(dir.path()).unwrap();
    let flat = loaded.flatten();

    let resupply = flat.iter().find(|m| m.name == "Resupply").unwrap();
    assert_eq!(resupply.keys.len(), 4);
    assert_eq!(resupply.keys[0].key, "KEY_S");
    assert_eq!(resupply.keys[2].key, "KEY_W");
}

#[test]
fn pack_empty_categories_are_valid() {
    let dir = tempfile::tempdir().unwrap();
    let pack = Pack {
        name: "Empty".to_string(),
        author: None,
        categories: vec![],
    };
    pack.save_to_dir(dir.path()).unwrap();

    let loaded = Pack::load_from_dir(dir.path()).unwrap();
    assert!(loaded.categories.is_empty());
    assert!(loaded.flatten().is_empty());
}

#[test]
fn pack_no_author_field_is_valid() {
    let dir = tempfile::tempdir().unwrap();
    let pack = Pack {
        name: "No Author".to_string(),
        author: None,
        categories: vec![],
    };
    pack.save_to_dir(dir.path()).unwrap();

    let loaded = Pack::load_from_dir(dir.path()).unwrap();
    assert_eq!(loaded.author, None);
}

// ---------------------------------------------------------------------------
// Export / Import (ZIP) tests
// ---------------------------------------------------------------------------

#[test]
fn pack_export_creates_zip_file() {
    let dir = tempfile::tempdir().unwrap();
    let pack = hd2_pack();
    pack.save_to_dir(dir.path()).unwrap();

    let zip_path = dir.path().join("hd2.hdpack");
    pack.export(dir.path(), &zip_path).expect("export must succeed");

    assert!(zip_path.exists(), ".hdpack file must be created");
    let metadata = std::fs::metadata(&zip_path).unwrap();
    assert!(metadata.len() > 0, "zip must not be empty");
}

#[test]
fn pack_export_zip_contains_pack_yaml() {
    let dir = tempfile::tempdir().unwrap();
    let pack = hd2_pack();
    pack.save_to_dir(dir.path()).unwrap();

    let zip_path = dir.path().join("hd2.hdpack");
    pack.export(dir.path(), &zip_path).unwrap();

    let file = std::fs::File::open(&zip_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    assert!(
        archive.by_name("pack.yaml").is_ok(),
        "zip must contain pack.yaml"
    );
}

#[test]
fn pack_export_zip_contains_sounds_when_present() {
    let dir = tempfile::tempdir().unwrap();
    let sounds_dir = dir.path().join("sounds");
    std::fs::create_dir_all(&sounds_dir).unwrap();
    std::fs::write(sounds_dir.join("reinforce.wav"), b"RIFF fake wav").unwrap();

    let pack = hd2_pack();
    pack.save_to_dir(dir.path()).unwrap();

    let zip_path = dir.path().join("hd2.hdpack");
    pack.export(dir.path(), &zip_path).unwrap();

    let file = std::fs::File::open(&zip_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    assert!(
        archive.by_name("sounds/reinforce.wav").is_ok(),
        "sounds/ must be bundled in zip"
    );
}

#[test]
fn pack_export_zip_no_sounds_dir_does_not_error() {
    let dir = tempfile::tempdir().unwrap();
    let pack = Pack {
        name: "NoSounds".to_string(),
        author: None,
        categories: vec![],
    };
    pack.save_to_dir(dir.path()).unwrap();

    let zip_path = dir.path().join("nosounds.hdpack");
    pack.export(dir.path(), &zip_path).expect("export without sounds/ must succeed");
    assert!(zip_path.exists());
}

#[test]
fn pack_import_from_zip_reads_name_and_macros() {
    // Build a zip in a temp dir, import it, and verify the imported pack content.
    // import() extracts to XDG_CONFIG_HOME/vibe-attack/profiles/<name>.
    // We redirect XDG_CONFIG_HOME to a second temp dir to stay hermetic.
    let source_dir = tempfile::tempdir().unwrap();
    let import_root = tempfile::tempdir().unwrap();

    let pack = Pack {
        name: "ImportTest".to_string(),
        author: Some("tester".to_string()),
        categories: vec![Category {
            name: "Test".to_string(),
            macros: vec![macro_simple(
                "Reinforce",
                "reinforce",
                vec![key("KEY_W"), key("KEY_S")],
            )],
        }],
    };
    pack.save_to_dir(source_dir.path()).unwrap();

    let zip_path = source_dir.path().join("importtest.hdpack");
    pack.export(source_dir.path(), &zip_path).unwrap();

    // Redirect XDG so import() writes into our temp dir, not the real config home.
    std::env::set_var("XDG_CONFIG_HOME", import_root.path());
    let imported = Pack::import(&zip_path).expect("import must succeed");
    std::env::remove_var("XDG_CONFIG_HOME");

    assert_eq!(imported.name, "ImportTest");
    assert_eq!(imported.author, Some("tester".to_string()));
    assert_eq!(imported.categories.len(), 1);

    let flat = imported.flatten();
    assert_eq!(flat.len(), 1);
    assert_eq!(flat[0].name, "Reinforce");
}

#[test]
fn pack_import_extracts_sounds_to_profile_dir() {
    let source_dir = tempfile::tempdir().unwrap();
    let import_root = tempfile::tempdir().unwrap();

    let sounds_dir = source_dir.path().join("sounds");
    std::fs::create_dir_all(&sounds_dir).unwrap();
    std::fs::write(sounds_dir.join("reinforce.wav"), b"RIFF fake wav").unwrap();

    let pack = Pack {
        name: "SoundImport".to_string(),
        author: None,
        categories: vec![],
    };
    pack.save_to_dir(source_dir.path()).unwrap();

    let zip_path = source_dir.path().join("sounds.hdpack");
    pack.export(source_dir.path(), &zip_path).unwrap();

    std::env::set_var("XDG_CONFIG_HOME", import_root.path());
    Pack::import(&zip_path).expect("import must succeed");
    std::env::remove_var("XDG_CONFIG_HOME");

    let extracted_wav = import_root
        .path()
        .join("vibe-attack/profiles/SoundImport/sounds/reinforce.wav");
    assert!(extracted_wav.exists(), "sounds/reinforce.wav must be extracted");
    assert_eq!(
        std::fs::read(&extracted_wav).unwrap(),
        b"RIFF fake wav"
    );
}

#[test]
fn pack_import_missing_zip_returns_err() {
    let result = Pack::import(std::path::Path::new("/tmp/nonexistent_hd2_pack_xyz.hdpack"));
    assert!(result.is_err(), "missing zip must return Err");
}

#[test]
fn pack_import_zip_missing_pack_yaml_returns_err() {
    let dir = tempfile::tempdir().unwrap();
    let zip_path = dir.path().join("bad.hdpack");

    // Create a zip without pack.yaml
    let file = std::fs::File::create(&zip_path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::FileOptions::default();
    zip.start_file("readme.txt", options).unwrap();
    zip.write_all(b"no pack yaml here").unwrap();
    zip.finish().unwrap();

    let result = Pack::import(&zip_path);
    assert!(result.is_err(), "zip without pack.yaml must return Err");
}

// ---------------------------------------------------------------------------
// ProfileManager tests
// ---------------------------------------------------------------------------

#[test]
fn profile_manager_no_active_profile_by_default() {
    let manager = ProfileManager { active_profile: None };
    assert!(manager.active_profile.is_none());
}

#[test]
fn profile_manager_persist_and_reload() {
    let dir = tempfile::tempdir().unwrap();
    let manager_path = dir.path().join("manager.yaml");

    let manager = ProfileManager {
        active_profile: Some("Helldivers 2".to_string()),
    };

    let f = std::fs::File::create(&manager_path).unwrap();
    serde_yaml_ng::to_writer(f, &manager).unwrap();

    let f = std::fs::File::open(&manager_path).unwrap();
    let loaded: ProfileManager = serde_yaml_ng::from_reader(f).unwrap();

    assert_eq!(loaded.active_profile, Some("Helldivers 2".to_string()));
}

#[test]
fn profile_manager_none_active_persists() {
    let dir = tempfile::tempdir().unwrap();
    let manager_path = dir.path().join("manager.yaml");

    let manager = ProfileManager { active_profile: None };
    let f = std::fs::File::create(&manager_path).unwrap();
    serde_yaml_ng::to_writer(f, &manager).unwrap();

    let f = std::fs::File::open(&manager_path).unwrap();
    let loaded: ProfileManager = serde_yaml_ng::from_reader(f).unwrap();
    assert_eq!(loaded.active_profile, None);
}

#[test]
fn profile_manager_get_active_pack_resolves_from_profiles_dir() {
    let dir = tempfile::tempdir().unwrap();
    let profiles_dir = dir.path().join("vibe-attack/profiles");
    let hd2_dir = profiles_dir.join("Helldivers 2");
    std::fs::create_dir_all(&hd2_dir).unwrap();

    let pack = hd2_pack();
    pack.save_to_dir(&hd2_dir).unwrap();

    // Redirect XDG so get_profiles_dir() returns our temp dir
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    let manager = ProfileManager {
        active_profile: Some("Helldivers 2".to_string()),
    };
    let result = manager.get_active_pack();
    std::env::remove_var("XDG_CONFIG_HOME");

    let active = result.expect("get_active_pack must succeed").expect("must have a pack");
    assert_eq!(active.name, "Helldivers 2");
    assert_eq!(active.flatten().len(), 9);
}

#[test]
fn profile_manager_get_active_pack_none_when_no_active() {
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    let manager = ProfileManager { active_profile: None };
    let result = manager.get_active_pack();
    std::env::remove_var("XDG_CONFIG_HOME");

    assert!(result.unwrap().is_none());
}

#[test]
fn profile_manager_get_active_pack_none_when_dir_missing() {
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    let manager = ProfileManager {
        active_profile: Some("NonExistent".to_string()),
    };
    let result = manager.get_active_pack();
    std::env::remove_var("XDG_CONFIG_HOME");

    assert!(result.unwrap().is_none());
}

// ---------------------------------------------------------------------------
// Full end-to-end: export HD2 pack, import it, set as active, retrieve it
// ---------------------------------------------------------------------------

#[test]
fn hd2_pack_full_lifecycle_export_import_activate_retrieve() {
    let export_dir = tempfile::tempdir().unwrap();
    let config_root = tempfile::tempdir().unwrap();

    // 1. Build and export the HD2 pack
    let pack = hd2_pack();
    pack.save_to_dir(export_dir.path()).unwrap();
    let zip_path = export_dir.path().join("helldivers2.hdpack");
    pack.export(export_dir.path(), &zip_path).unwrap();

    // 2. Import it (extracts to profiles/Helldivers 2/)
    std::env::set_var("XDG_CONFIG_HOME", config_root.path());
    let imported = Pack::import(&zip_path).expect("import must succeed");
    assert_eq!(imported.name, "Helldivers 2");

    // 3. Set as active profile and retrieve it
    let manager = ProfileManager {
        active_profile: Some("Helldivers 2".to_string()),
    };
    let active = manager
        .get_active_pack()
        .expect("get_active_pack must succeed")
        .expect("active pack must be present");
    std::env::remove_var("XDG_CONFIG_HOME");

    assert_eq!(active.name, "Helldivers 2");
    let flat = active.flatten();
    assert_eq!(flat.len(), 9);

    // Verify key stratagem is intact after the full lifecycle
    let reinforce = flat.iter().find(|m| m.name == "Reinforce").unwrap();
    assert_eq!(reinforce.phrase, Some("reinforce".to_string()));
    assert_eq!(reinforce.keys.len(), 5);
}
