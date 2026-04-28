// Hermetic round-trip integration tests for PackEditor.
//
// Proves that edits applied through PackEditor survive a full save → reload
// cycle without corruption, and that serde_yaml_ng output is byte-stable
// within a single process run.  No XDG writes, no network, no model files.

use vibe_attack::{
    config::{KeyAction, MacroConfig},
    pack::{Category, MacroUpdates, Pack, PackEditor},
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

fn macro_optional(
    name: &str,
    phrase: Option<&str>,
    if_flag: Option<&str>,
    set_flag: Option<&str>,
    keys: Vec<KeyAction>,
) -> MacroConfig {
    MacroConfig {
        name: name.to_string(),
        phrase: phrase.map(str::to_string),
        if_flag: if_flag.map(str::to_string),
        set_flag: set_flag.map(str::to_string),
        sound: None,
        keys,
    }
}

/// Two categories, four macros with a mix of phrase/if_flag/set_flag/keys with timing.
fn starter_pack() -> Pack {
    Pack {
        name: "RoundtripTest".to_string(),
        author: Some("test-author".to_string()),
        categories: vec![
            Category {
                name: "Stratagems".to_string(),
                macros: vec![
                    macro_simple(
                        "Reinforce",
                        "reinforce",
                        vec![key("KEY_W"), key("KEY_S"), key("KEY_D"), key("KEY_W"), key("KEY_A")],
                    ),
                    macro_simple(
                        "Resupply",
                        "resupply",
                        vec![key("KEY_S"), key("KEY_S"), key("KEY_W"), key("KEY_D")],
                    ),
                ],
            },
            Category {
                name: "Support".to_string(),
                macros: vec![
                    macro_optional(
                        "Hellbomb Arm",
                        Some("arm hellbomb"),
                        Some("hellbomb_present"),
                        Some("hellbomb_armed"),
                        vec![key_timed("KEY_E", 200, 100)],
                    ),
                    macro_optional(
                        "Clear Flag",
                        None,
                        None,
                        Some("hellbomb_armed"),
                        vec![],
                    ),
                ],
            },
        ],
    }
}

// ---------------------------------------------------------------------------
// Deep equality helper (explicit field walk, no PartialEq derive on Pack)
// ---------------------------------------------------------------------------

fn assert_packs_equal(left: &Pack, right: &Pack) {
    assert_eq!(left.name, right.name, "pack.name mismatch");
    assert_eq!(left.author, right.author, "pack.author mismatch");
    assert_eq!(
        left.categories.len(),
        right.categories.len(),
        "pack.categories.len() mismatch: left={}, right={}",
        left.categories.len(),
        right.categories.len()
    );

    for (i, (lcat, rcat)) in left.categories.iter().zip(right.categories.iter()).enumerate() {
        assert_eq!(
            lcat.name, rcat.name,
            "category[{}].name mismatch: '{}' vs '{}'",
            i, lcat.name, rcat.name
        );
        assert_eq!(
            lcat.macros.len(),
            rcat.macros.len(),
            "category[{}] ('{}') macro count mismatch",
            i,
            lcat.name
        );

        for (j, (lm, rm)) in lcat.macros.iter().zip(rcat.macros.iter()).enumerate() {
            let loc = format!("category[{}]['{}'].macros[{}]", i, lcat.name, j);
            assert_eq!(lm.name, rm.name, "{}.name", loc);
            assert_eq!(lm.phrase, rm.phrase, "{}.phrase", loc);
            assert_eq!(lm.if_flag, rm.if_flag, "{}.if_flag", loc);
            assert_eq!(lm.set_flag, rm.set_flag, "{}.set_flag", loc);
            assert_eq!(lm.sound, rm.sound, "{}.sound", loc);
            assert_eq!(lm.keys.len(), rm.keys.len(), "{}.keys.len()", loc);
            for (k, (lk, rk)) in lm.keys.iter().zip(rm.keys.iter()).enumerate() {
                assert_eq!(lk.key, rk.key, "{}.keys[{}].key", loc, k);
                assert_eq!(lk.dwell_ms, rk.dwell_ms, "{}.keys[{}].dwell_ms", loc, k);
                assert_eq!(lk.gap_ms, rk.gap_ms, "{}.keys[{}].gap_ms", loc, k);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Full CRUD sequence: edit → save → reload → structural equality.
///
/// Exercises every PackEditor method in one sequence, then proves the
/// in-memory Pack and the reloaded Pack are structurally identical.
#[test]
fn roundtrip_after_full_crud_sequence() {
    let mut ed = PackEditor::new(starter_pack());

    // 1. Add a new category
    ed.add_category("NewCat").expect("add_category must succeed");

    // 2. Add a macro to the new category
    ed.add_macro(
        "NewCat",
        macro_simple("Eagle Airstrike", "eagle airstrike", vec![
            key("KEY_W"), key("KEY_D"), key("KEY_S"), key("KEY_D"),
        ]),
    )
    .expect("add_macro to NewCat must succeed");

    // 3. Edit an existing macro in Stratagems
    ed.edit_macro(
        "Stratagems",
        "Reinforce",
        MacroUpdates {
            phrase: Some(Some("call for reinforce".to_string())),
            keys: Some(vec![key("KEY_W"), key("KEY_S"), key("KEY_W")]),
            ..Default::default()
        },
    )
    .expect("edit_macro must succeed");

    // 4. Move a macro between categories
    ed.move_macro("Support", "Stratagems", "Clear Flag")
        .expect("move_macro must succeed");

    // 5. Rename a category (in-place, preserving macros)
    ed.rename_category("NewCat", "Eagles").expect("rename_category must succeed");

    // 6. Remove a macro to prepare for category removal
    ed.remove_macro("Eagles", "Eagle Airstrike")
        .expect("remove_macro must succeed");

    // 7. Remove the now-empty category
    ed.remove_category("Eagles").expect("remove_category on empty category must succeed");

    // Capture expected state before save
    let expected = ed.pack().clone();

    // Save to tempdir and reload
    let dir = tempfile::tempdir().expect("tempdir must be created");
    expected.save_to_dir(dir.path()).expect("save_to_dir must succeed");
    let reloaded = Pack::load_from_dir(dir.path()).expect("load_from_dir must succeed");

    assert_packs_equal(&expected, &reloaded);
}

/// Byte-for-byte YAML stability: two saves of the same in-memory Pack must
/// produce identical bytes, guarding against serde_yaml_ng non-determinism.
#[test]
fn roundtrip_yaml_text_stable_within_run() {
    let mut ed = PackEditor::new(starter_pack());

    // Apply a representative edit so the pack is non-trivial
    ed.add_category("Boosters").expect("add_category must succeed");
    ed.add_macro(
        "Boosters",
        macro_simple("Vitality Enhancement", "vitality enhancement", vec![key("KEY_V")]),
    )
    .expect("add_macro must succeed");
    ed.edit_macro(
        "Stratagems",
        "Resupply",
        MacroUpdates {
            phrase: Some(Some("drop resupply".to_string())),
            ..Default::default()
        },
    )
    .expect("edit_macro must succeed");

    let pack = ed.into_pack();

    let dir1 = tempfile::tempdir().expect("tempdir 1 must be created");
    let dir2 = tempfile::tempdir().expect("tempdir 2 must be created");

    pack.save_to_dir(dir1.path()).expect("save to dir1 must succeed");
    pack.save_to_dir(dir2.path()).expect("save to dir2 must succeed");

    let bytes1 = std::fs::read(dir1.path().join("pack.yaml"))
        .expect("reading dir1/pack.yaml must succeed");
    let bytes2 = std::fs::read(dir2.path().join("pack.yaml"))
        .expect("reading dir2/pack.yaml must succeed");

    assert_eq!(
        bytes1, bytes2,
        "two saves of the same Pack must produce byte-identical YAML"
    );
}

/// Verifies that all Option fields (phrase=None, if_flag=Some, set_flag=None,
/// dwell/gap overrides) survive a save → reload cycle exactly.
#[test]
fn roundtrip_preserves_optional_fields() {
    let pack = Pack {
        name: "OptionalFieldsTest".to_string(),
        author: None,
        categories: vec![Category {
            name: "Mixed".to_string(),
            macros: vec![
                // phrase=None, if_flag=Some, set_flag=None, no timing
                macro_optional(
                    "Silent Trigger",
                    None,
                    Some("some_flag"),
                    None,
                    vec![key("KEY_X")],
                ),
                // phrase=Some, if_flag=None, set_flag=None, timing on all keys
                macro_optional(
                    "Timed Action",
                    Some("timed action"),
                    None,
                    None,
                    vec![
                        key_timed("KEY_A", 150, 50),
                        key_timed("KEY_B", 200, 100),
                    ],
                ),
                // phrase=Some, set_flag=Some, mixed keys (some timed, some not)
                macro_optional(
                    "Flag Setter",
                    Some("set flag"),
                    None,
                    Some("result_flag"),
                    vec![key("KEY_W"), key_timed("KEY_S", 300, 200), key("KEY_D")],
                ),
                // no keys at all — valid edge case
                macro_optional("Empty Keys", Some("empty keys"), None, None, vec![]),
            ],
        }],
    };

    // Apply a no-op edit: add then immediately remove a temporary macro.
    let mut ed = PackEditor::new(pack.clone());
    ed.add_macro(
        "Mixed",
        macro_simple("Temp", "temp", vec![key("KEY_T")]),
    )
    .expect("add temp macro must succeed");
    ed.remove_macro("Mixed", "Temp").expect("remove temp macro must succeed");
    let edited = ed.into_pack();

    let dir = tempfile::tempdir().expect("tempdir must be created");
    edited.save_to_dir(dir.path()).expect("save_to_dir must succeed");
    let reloaded = Pack::load_from_dir(dir.path()).expect("load_from_dir must succeed");

    assert_packs_equal(&edited, &reloaded);

    // Extra explicit checks for Option fields
    let macros = &reloaded.categories[0].macros;

    let silent = macros.iter().find(|m| m.name == "Silent Trigger").unwrap();
    assert_eq!(silent.phrase, None, "phrase must remain None");
    assert_eq!(silent.if_flag, Some("some_flag".to_string()), "if_flag must survive");
    assert_eq!(silent.set_flag, None, "set_flag must remain None");

    let timed = macros.iter().find(|m| m.name == "Timed Action").unwrap();
    assert_eq!(timed.keys[0].dwell_ms, Some(150), "dwell_ms must survive");
    assert_eq!(timed.keys[0].gap_ms, Some(50), "gap_ms must survive");
    assert_eq!(timed.keys[1].dwell_ms, Some(200), "second key dwell_ms must survive");

    let setter = macros.iter().find(|m| m.name == "Flag Setter").unwrap();
    assert_eq!(setter.set_flag, Some("result_flag".to_string()), "set_flag must survive");
    assert_eq!(setter.keys[0].dwell_ms, None, "untimed key dwell_ms must remain None");
    assert_eq!(setter.keys[1].dwell_ms, Some(300), "timed key dwell_ms must survive");

    let empty = macros.iter().find(|m| m.name == "Empty Keys").unwrap();
    assert!(empty.keys.is_empty(), "empty keys vec must survive round-trip");
}
