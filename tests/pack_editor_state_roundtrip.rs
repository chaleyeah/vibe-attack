// State-machine roundtrip integration tests for PackEditorState / PackEditor.
//
// Proves that editor mutation calls (add_category, add_macro, edit_macro,
// move_macro, rename_category, remove_macro, remove_category) survive a full
// save → reload cycle without corruption.  No egui, no daemon, no XDG writes.
//
// PackEditorState wraps PackEditor with egui UI fields; the state machine
// (mutation calls) lives on PackEditor itself.  These tests drive PackEditor
// directly — the path the UI buttons exercise — and verify the disk-level
// output via Pack::save_to_dir / Pack::load_from_dir.

use std::fs;

use vibe_attack::{
    config::{KeyAction, MacroConfig},
    pack::{Category, MacroUpdates, Pack, PackEditor},
    ui::pack_editor::parse_key_sequence,
};

// ---------------------------------------------------------------------------
// Fixture helpers
// ---------------------------------------------------------------------------

fn key(name: &str) -> KeyAction {
    KeyAction { key: name.to_string(), dwell_ms: None, gap_ms: None }
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

/// Two categories, two macros each — mirrors the S02 fixture shape.
fn fixture_pack() -> Pack {
    Pack {
        name: "StateRoundtripTest".to_string(),
        author: Some("state-test-author".to_string()),
        categories: vec![
            Category {
                name: "Cat1".to_string(),
                macros: vec![
                    macro_simple(
                        "m1",
                        "phrase one",
                        vec![key("KEY_W"), key("KEY_S")],
                    ),
                    macro_simple(
                        "orig_macro",
                        "orig phrase",
                        vec![key("KEY_A"), key("KEY_D")],
                    ),
                ],
            },
            Category {
                name: "Cat2".to_string(),
                macros: vec![
                    macro_simple("m3", "phrase three", vec![key("KEY_Q")]),
                    macro_simple("m4", "phrase four", vec![key("KEY_E")]),
                ],
            },
        ],
    }
}

// ---------------------------------------------------------------------------
// Deep equality helper — explicit field walk, no PartialEq derive on Pack
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
            "category[{}] ('{}') macro count mismatch: left={}, right={}",
            i,
            lcat.name,
            lcat.macros.len(),
            rcat.macros.len()
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

/// Full CRUD sequence through the PackEditor state-machine layer — the same
/// calls the egui UI buttons issue — then save → reload → structural equality.
#[test]
fn state_roundtrip_after_full_crud_via_state_layer() {
    let mut editor = PackEditor::new(fixture_pack());

    // The profile_dir mirrors what PackEditorState::new receives.
    let base = tempfile::tempdir().expect("tempdir must be created");
    let profile_dir = base.path().join("hd2");

    // Drive the same mutation calls the UI buttons would issue.
    editor.add_category("NewCat").expect("add_category must succeed");

    editor
        .add_macro(
            "NewCat",
            macro_simple("new_m", "new macro phrase", vec![key("KEY_N")]),
        )
        .expect("add_macro to NewCat must succeed");

    editor
        .edit_macro(
            "NewCat",
            "new_m",
            MacroUpdates {
                phrase: Some(Some("new phrase".into())),
                ..Default::default()
            },
        )
        .expect("edit_macro must succeed");

    editor
        .move_macro("Cat1", "NewCat", "orig_macro")
        .expect("move_macro must succeed");

    editor
        .rename_category("Cat2", "Cat2Renamed")
        .expect("rename_category must succeed");

    editor
        .remove_macro("NewCat", "orig_macro")
        .expect("remove_macro must succeed");

    // Cat1 still has m1 (orig_macro was moved out); remove m1 before removing the category.
    editor.remove_macro("Cat1", "m1").expect("remove m1 from Cat1 must succeed");

    editor.remove_category("Cat1").expect("remove_category on empty Cat1 must succeed");

    // Save to disk — mirrors PackEditorState::save() calling pack.save_to_dir.
    fs::create_dir_all(&profile_dir).expect("create_dir_all must succeed");
    editor.pack().save_to_dir(&profile_dir).expect("save_to_dir must succeed");

    // Reload and assert equivalence.
    let reloaded = Pack::load_from_dir(&profile_dir).expect("load_from_dir must succeed");
    assert_packs_equal(editor.pack(), &reloaded);
}

/// parse_key_sequence → MacroConfig → add_macro → save → reload → keys survive.
///
/// Exercises the public parse_key_sequence helper (used by the Update Macro
/// button) and proves the key strings plus None dwell/gap survive a disk cycle.
#[test]
fn state_parse_key_sequence_drives_form_to_save() {
    let pack = Pack {
        name: "KeySeqTest".to_string(),
        author: None,
        categories: vec![Category {
            name: "Actions".to_string(),
            macros: vec![],
        }],
    };

    let mut editor = PackEditor::new(pack);

    // parse_key_sequence is the path the form's keys field takes.
    let keys = parse_key_sequence("KEY_W, KEY_A, KEY_S, KEY_D").expect("parse must succeed");
    assert_eq!(keys.len(), 4);
    assert!(keys.iter().all(|k| k.dwell_ms.is_none() && k.gap_ms.is_none()));

    let mc = MacroConfig {
        name: "DirectionCycle".to_string(),
        phrase: Some("rotate".to_string()),
        if_flag: None,
        set_flag: None,
        sound: None,
        keys: keys.clone(),
    };

    editor.add_macro("Actions", mc).expect("add_macro must succeed");

    let dir = tempfile::tempdir().expect("tempdir must be created");
    editor.pack().save_to_dir(dir.path()).expect("save_to_dir must succeed");

    let reloaded = Pack::load_from_dir(dir.path()).expect("load_from_dir must succeed");

    let found = reloaded.categories[0]
        .macros
        .iter()
        .find(|m| m.name == "DirectionCycle")
        .expect("macro must exist after reload");

    assert_eq!(found.keys.len(), 4, "all 4 keys must survive reload");

    // Byte-for-byte: key strings must be unchanged; dwell/gap must remain None.
    let expected_keys = ["KEY_W", "KEY_A", "KEY_S", "KEY_D"];
    for (i, (actual, expected_key)) in found.keys.iter().zip(expected_keys.iter()).enumerate() {
        assert_eq!(actual.key, *expected_key, "keys[{}].key mismatch", i);
        assert_eq!(actual.dwell_ms, None, "keys[{}].dwell_ms must be None", i);
        assert_eq!(actual.gap_ms, None, "keys[{}].gap_ms must be None", i);
    }
}

/// Smoke test: save_to_dir produces a valid pack.yaml that round-trips through
/// serde_yaml_ng without information loss.
#[test]
fn state_save_to_dir_writes_pack_yaml() {
    let mut editor = PackEditor::new(fixture_pack());
    editor.add_category("Extra").expect("add_category must succeed");
    editor
        .add_macro("Extra", macro_simple("bonus", "bonus phrase", vec![key("KEY_B")]))
        .expect("add_macro must succeed");

    let dir = tempfile::tempdir().expect("tempdir must be created");
    editor.pack().save_to_dir(dir.path()).expect("save_to_dir must succeed");

    let yaml_path = dir.path().join("pack.yaml");
    assert!(yaml_path.exists(), "pack.yaml must exist after save_to_dir");

    let content = fs::read_to_string(&yaml_path).expect("reading pack.yaml must succeed");
    assert!(!content.is_empty(), "pack.yaml must not be empty");

    let reparsed: Pack =
        serde_yaml_ng::from_str(&content).expect("pack.yaml must round-trip through serde_yaml_ng");

    assert_packs_equal(editor.pack(), &reparsed);
}
