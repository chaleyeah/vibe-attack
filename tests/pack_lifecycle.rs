// Integration tests: export → import_to round-trip for Pack.
//
// These tests are hermetic — no XDG_CONFIG_HOME mutation, no #[serial].
// Both use tempfile::tempdir() for isolation and Pack::import_to() so they
// are safe to run in parallel with the rest of the suite.

use vibe_attack::{
    config::{KeyAction, MacroConfig},
    pack::{Category, Pack},
};

// ---------------------------------------------------------------------------
// Fixture helpers (inlined from pack_hd2_bundle.rs pattern)
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

fn macro_flagged(
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

/// Build a fixture pack that exercises every MacroConfig field:
/// phrase, if_flag, set_flag, sound: None, keys with and without dwell_ms/gap_ms.
fn lifecycle_fixture() -> Pack {
    Pack {
        name: "LifecycleRoundTripFixture".to_string(),
        author: Some("test-author".to_string()),
        categories: vec![
            Category {
                name: "Basic Actions".to_string(),
                macros: vec![
                    macro_simple(
                        "Move Forward",
                        "move forward",
                        vec![key("KEY_W"), key("KEY_W")],
                    ),
                    macro_simple(
                        "Move Back",
                        "move back",
                        vec![key("KEY_S")],
                    ),
                    macro_simple(
                        "Strafe Left",
                        "strafe left",
                        vec![key("KEY_A"), key_timed("KEY_A", 150, 80)],
                    ),
                ],
            },
            Category {
                name: "Flag-Gated".to_string(),
                macros: vec![
                    macro_flagged(
                        "Arm Device",
                        "arm device",
                        Some("device_present"),
                        Some("device_armed"),
                        vec![key_timed("KEY_E", 200, 100)],
                    ),
                    macro_flagged(
                        "Clear Flag",
                        "clear flag",
                        None,
                        Some("device_armed"),
                        vec![],
                    ),
                ],
            },
            Category {
                name: "No-Keys".to_string(),
                macros: vec![
                    MacroConfig {
                        name: "Silent Trigger".to_string(),
                        phrase: Some("silent trigger".to_string()),
                        if_flag: Some("quiet_mode".to_string()),
                        set_flag: None,
                        sound: None,
                        keys: vec![],
                    },
                ],
            },
        ],
    }
}

// ---------------------------------------------------------------------------
// Test 1: macro content round-trips through export → import_to
// ---------------------------------------------------------------------------

#[test]
fn pack_export_then_import_to_round_trips_macros() {
    let source_dir = tempfile::tempdir().unwrap();
    let dest_dir = tempfile::tempdir().unwrap();

    let pack = lifecycle_fixture();

    // Save the pack to the source tempdir so export() can find pack.yaml.
    pack.save_to_dir(source_dir.path()).expect("save_to_dir must succeed");

    let zip_path = source_dir.path().join("lifecycle.hdpack");
    pack.export(source_dir.path(), &zip_path).expect("export must succeed");

    // import_to: no XDG mutation, hermetic dest.
    Pack::import_to(&zip_path, dest_dir.path()).expect("import_to must succeed");

    // Reload from the extracted directory to verify on-disk state.
    let pack_dir = dest_dir.path().join(&pack.name);
    let reloaded = Pack::load_from_dir(&pack_dir).expect("load_from_dir must succeed");

    // Top-level fields.
    assert_eq!(reloaded.name, pack.name);
    assert_eq!(reloaded.author, pack.author);

    // Category count and order.
    assert_eq!(reloaded.categories.len(), pack.categories.len());
    let orig_cat_names: Vec<&str> = pack.categories.iter().map(|c| c.name.as_str()).collect();
    let reloaded_cat_names: Vec<&str> = reloaded.categories.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(reloaded_cat_names, orig_cat_names);

    // Macro counts and names per category.
    for (orig_cat, rel_cat) in pack.categories.iter().zip(reloaded.categories.iter()) {
        assert_eq!(rel_cat.macros.len(), orig_cat.macros.len(),
            "macro count must match in category '{}'", orig_cat.name);
        let orig_names: Vec<&str> = orig_cat.macros.iter().map(|m| m.name.as_str()).collect();
        let rel_names: Vec<&str> = rel_cat.macros.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(rel_names, orig_names,
            "macro names must match in order in category '{}'", orig_cat.name);
    }

    // Deep field equality for every macro.
    let orig_flat = pack.flatten();
    let rel_flat = reloaded.flatten();
    assert_eq!(rel_flat.len(), orig_flat.len());

    for (orig, rel) in orig_flat.iter().zip(rel_flat.iter()) {
        assert_eq!(rel.name, orig.name, "macro name");
        assert_eq!(rel.phrase, orig.phrase, "macro phrase (name={})", orig.name);
        assert_eq!(rel.if_flag, orig.if_flag, "macro if_flag (name={})", orig.name);
        assert_eq!(rel.set_flag, orig.set_flag, "macro set_flag (name={})", orig.name);
        assert_eq!(rel.sound, orig.sound, "macro sound (name={})", orig.name);
        assert_eq!(rel.keys.len(), orig.keys.len(),
            "key count (macro={})", orig.name);
        for (i, (ok, rk)) in orig.keys.iter().zip(rel.keys.iter()).enumerate() {
            assert_eq!(rk.key, ok.key, "key[{}].key (macro={})", i, orig.name);
            assert_eq!(rk.dwell_ms, ok.dwell_ms, "key[{}].dwell_ms (macro={})", i, orig.name);
            assert_eq!(rk.gap_ms, ok.gap_ms, "key[{}].gap_ms (macro={})", i, orig.name);
        }
    }
}

// ---------------------------------------------------------------------------
// Test 2: sounds/ subdirectory is bundled and extracted intact
// ---------------------------------------------------------------------------

#[test]
fn pack_export_imports_sounds_subdirectory() {
    let source_dir = tempfile::tempdir().unwrap();
    let dest_dir = tempfile::tempdir().unwrap();

    // Write a dummy wav under source_dir/sounds/
    let sounds_dir = source_dir.path().join("sounds");
    std::fs::create_dir_all(&sounds_dir).unwrap();
    let wav_bytes = b"RIFF\x24\x00\x00\x00WAVEfmt ";
    std::fs::write(sounds_dir.join("test.wav"), wav_bytes).unwrap();

    let pack = Pack {
        name: "LifecycleRoundTripFixture".to_string(),
        author: None,
        categories: vec![],
    };
    pack.save_to_dir(source_dir.path()).unwrap();

    let zip_path = source_dir.path().join("lifecycle_sounds.hdpack");
    pack.export(source_dir.path(), &zip_path).expect("export must succeed");

    Pack::import_to(&zip_path, dest_dir.path()).expect("import_to must succeed");

    let extracted_wav = dest_dir
        .path()
        .join("LifecycleRoundTripFixture")
        .join("sounds")
        .join("test.wav");

    assert!(extracted_wav.exists(), "sounds/test.wav must be extracted");
    assert_eq!(
        std::fs::read(&extracted_wav).unwrap(),
        wav_bytes,
        "extracted wav bytes must be identical to original"
    );
}
