---
id: S01
parent: M009
milestone: M009
provides:
  - ["profiles/hd2/pack.yaml with 75 stratagems across 6 ship-module categories", "tests/pack_hd2_coverage.rs hermetic regression guard"]
requires:
  []
affects:
  - ["S06"]
key_files:
  - ["profiles/hd2/pack.yaml", "tests/pack_hd2_coverage.rs"]
key_decisions:
  - ["All stratagem key sequences use KEY_UP/KEY_DOWN/KEY_LEFT/KEY_RIGHT evdev names — no WASD mixing", "Bridge category includes Reinforce and Resupply (cross-module stratagems that ship with the Bridge in-game)", "Spear sequence chosen as ↓↓↑↓↓ (canonical in-game input)", "Coverage test loads fixture via Path::new(\"profiles/hd2\") with no XDG_CONFIG_HOME mutation — no #[serial] needed", "HashSet difference used for missing-category assertion to produce a clear failure message naming absent categories"]
patterns_established:
  - ["Hermetic pack coverage tests: load from repo fixture path, assert by HashSet difference for clear failure messages, no env mutation, no #[serial]", "Integration test style: vibe_attack::pack::Pack import, .expect() for fixture loads (not anyhow Result), module-level //! doc comment per D002"]
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-28T02:26:41.227Z
blocker_discovered: false
---

# S01: Full HD2 stratagem pack + coverage test

**Expanded profiles/hd2/pack.yaml from 12 to 75 stratagems across all 6 ship-module categories, with a hermetic coverage test that guards against silent regressions.**

## What Happened

T01 rewrote profiles/hd2/pack.yaml to contain exactly 75 Helldivers 2 stratagems organized into the 6 required ship-module categories. The existing 12 canonical entries were preserved verbatim. All per-category minimums are met: Patriotic Administration Center (17 ≥ 10), Orbital Cannons (13 ≥ 12), Hangar (12 ≥ 10), Bridge (9 ≥ 5), Engineering Bay (12 ≥ 8), Robotics Workshop (12 ≥ 6). All key sequences use KEY_UP/KEY_DOWN/KEY_LEFT/KEY_RIGHT evdev names exclusively — no WASD mixing. All 75 phrases are unique and lowercase. The YAML schema is unchanged from M001. A malformed Spear entry ({key} YAML anchor typo instead of {key: KEY_UP}) was caught during T01 by the pack_hd2_bundle regression test and fixed before completion.

T02 added tests/pack_hd2_coverage.rs with two hermetic integration tests. The file was modeled on tests/pack_hd2_bundle.rs: same vibe_attack::pack::Pack import, no anyhow Result return (uses .expect() for fixture loads), no #[serial] since no XDG_CONFIG_HOME mutation. The test loads via Path::new("profiles/hd2") which works because integration tests run with CWD = workspace root. hd2_pack_covers_all_ship_modules() asserts: pack name, all 6 category names present (HashSet difference for clear failure messages), per-category minimum counts, total ≥ 75, and no category has zero macros. hd2_pack_phrases_are_unique() flattens all phrases and asserts Vec len == HashSet len. A module-level //! doc comment describes the test file's purpose per D002.

## Verification

cargo test --test pack_hd2_coverage -- --nocapture: 2 tests passed (hd2_pack_covers_all_ship_modules, hd2_pack_phrases_are_unique). cargo test -- --test-threads=1: full suite passed including all pack_hd2_bundle and pack_hd2_coverage tests. RUSTFLAGS=\"-D warnings\" cargo check --all-targets: clean exit, 0 warnings.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

none

## Known Limitations

Key sequence accuracy for the HD2 stratagems has not been verified against live gameplay — sequences were sourced from the canonical community stratagem reference. The Spear in particular (↓↓↑↓↓) has some community disagreement; the in-game canonical input was used.

## Follow-ups

S02 (PackEditor CRUD) and S03 (Egui editor panel) can proceed independently — neither depends on S01 beyond the existing pack infrastructure which was already in place before this slice.

## Files Created/Modified

- `profiles/hd2/pack.yaml` — Expanded from 12 to 75 stratagems across 6 ship-module categories
- `tests/pack_hd2_coverage.rs` — New hermetic integration test asserting category presence, minimum counts, total >= 75, and phrase uniqueness
