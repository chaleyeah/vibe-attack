//! Hermetic coverage tests for the bundled Helldivers 2 stratagem pack.
//!
//! Loads `profiles/hd2/pack.yaml` directly from the repo (CWD = workspace root
//! when integration tests run) via `Pack::load_from_dir`.  No XDG mutation,
//! no network, no model files, no ProfileManager.

use std::collections::HashSet;
use std::path::Path;

use vibe_attack::pack::Pack;

/// Asserts that the bundled HD2 pack covers all six ship-module categories,
/// that each category meets its minimum macro count, and that the total
/// macro count is at least 75.
#[test]
fn hd2_pack_covers_all_ship_modules() {
    let pack = Pack::load_from_dir(Path::new("profiles/hd2"))
        .expect("profiles/hd2/pack.yaml must be loadable from workspace root");

    // (1) Pack name
    assert_eq!(pack.name, "Helldivers 2", "pack name must be 'Helldivers 2'");

    // (2) All six expected categories must be present
    let expected_categories: HashSet<&str> = [
        "Patriotic Administration Center",
        "Orbital Cannons",
        "Hangar",
        "Bridge",
        "Engineering Bay",
        "Robotics Workshop",
    ]
    .iter()
    .copied()
    .collect();

    let actual_categories: HashSet<&str> = pack
        .categories
        .iter()
        .map(|c| c.name.as_str())
        .collect();

    let missing: Vec<&&str> = expected_categories.difference(&actual_categories).collect();
    assert!(
        missing.is_empty(),
        "missing ship-module categories: {:?}",
        missing
    );

    // (3) Per-category minimum macro counts
    let minimums: &[(&str, usize)] = &[
        ("Patriotic Administration Center", 10),
        ("Orbital Cannons", 12),
        ("Hangar", 10),
        ("Bridge", 5),
        ("Engineering Bay", 8),
        ("Robotics Workshop", 6),
    ];

    for (cat_name, min_count) in minimums {
        let cat = pack
            .categories
            .iter()
            .find(|c| c.name == *cat_name)
            .unwrap_or_else(|| panic!("category '{}' not found in pack", cat_name));
        assert!(
            cat.macros.len() >= *min_count,
            "category '{}' has {} macros but needs >= {}",
            cat_name,
            cat.macros.len(),
            min_count
        );
    }

    // (4) Total macro count >= 75
    let total = pack.flatten().len();
    assert!(
        total >= 75,
        "total macro count is {} but must be >= 75",
        total
    );

    // (5) No category has zero macros
    for cat in &pack.categories {
        assert!(
            !cat.macros.is_empty(),
            "category '{}' has zero macros",
            cat.name
        );
    }
}

/// Asserts that every phrase string in the bundled HD2 pack is unique —
/// duplicate phrases would cause ambiguous voice trigger matching.
#[test]
fn hd2_pack_phrases_are_unique() {
    let pack = Pack::load_from_dir(Path::new("profiles/hd2"))
        .expect("profiles/hd2/pack.yaml must be loadable from workspace root");

    let all_macros = pack.flatten();

    let phrases: Vec<&str> = all_macros
        .iter()
        .filter_map(|m| m.phrase.as_deref())
        .collect();

    let unique: HashSet<&&str> = phrases.iter().collect();

    assert_eq!(
        unique.len(),
        phrases.len(),
        "duplicate phrases found in HD2 pack (total={}, unique={})",
        phrases.len(),
        unique.len()
    );
}
