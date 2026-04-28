# S01: Full HD2 stratagem pack + coverage test

**Goal:** Expand profiles/hd2/pack.yaml from 12 macros across 3 categories to 75+ macros across all 6 ship-module categories (Patriotic Administration Center, Orbital Cannons, Hangar, Bridge, Engineering Bay, Robotics Workshop) with verified arrow-key sequences, and add a new hermetic integration test (tests/pack_hd2_coverage.rs) that asserts category presence and minimum macro counts so future edits cannot silently drop entries.
**Demo:** cargo test --test pack_hd2_coverage passes; grep on profiles/hd2/pack.yaml shows all 6+ ship-module categories present with correct stratagem counts

## Must-Haves

- cargo test --test pack_hd2_coverage passes; pack.yaml contains all 6 ship-module categories named exactly as per the milestone roadmap; minimum macro counts per category enforced by the test (Administration >= 10, Orbital >= 12, Hangar >= 10, Bridge >= 5, Engineering Bay >= 8, Robotics Workshop >= 6); total macro count >= 75; no category is empty; cargo test (full suite) passes including pack_hd2_bundle regression check; RUSTFLAGS="-D warnings" cargo check --all-targets clean.

## Proof Level

- This slice proves: Not provided.

## Integration Closure

Not provided.

## Verification

- Not provided.

## Tasks

- [x] **T01: Expand profiles/hd2/pack.yaml to full 6-category HD2 stratagem coverage** `est:2h`
  Rewrite profiles/hd2/pack.yaml so it contains 75+ Helldivers 2 stratagems organized into the 6 ship-module categories the roadmap requires: Patriotic Administration Center (>=10 support weapons & defensive items), Orbital Cannons (>=12 orbital strikes), Hangar (>=10 eagle stratagems), Bridge (>=5 vehicle/mech stratagems), Engineering Bay (>=8 defensive structures & supply), Robotics Workshop (>=6 autonomous units). Keep YAML schema identical to the existing file (top-level name + author + categories[].macros[] with name/phrase/keys; KeyAction objects with `key:` evdev names). Use arrow-key sequences (KEY_UP, KEY_DOWN, KEY_LEFT, KEY_RIGHT) for consistency with the existing 12 entries — do NOT mix with KEY_W/KEY_A/KEY_S/KEY_D. Phrases must be unique within the pack and lowercase, matching the in-game stratagem name (e.g. phrase: orbital railcannon strike). Preserve existing 12 entries verbatim where they remain canonical (Machine Gun, Anti-Material Rifle, Stalwart, Orbital Gatling Barrage, Orbital Airburst Strike, Orbital 120mm HE Barrage, Eagle Strafing Run, Eagle Airstrike, Eagle Cluster Bomb). Source key sequences from the canonical Helldivers 2 stratagem reference (e.g. Reinforce ↑↓→←↑, Resupply ↓↓↑→, 500kg Bomb ↑→↓↓↓, Orbital Railcannon Strike →↑↓↓→). Validate YAML by running `cargo run --bin vibe-attack -- --help` (which loads no profiles, but compiles) AND by running `cargo test --test pack_hd2_bundle` — the bundle test loads pack.yaml in several places so it will fail on a malformed file. After expansion, the file should be approximately 200-350 lines of YAML.
  - Files: `profiles/hd2/pack.yaml`
  - Verify: cargo test --test pack_hd2_bundle && python3 -c "import yaml; d=yaml.safe_load(open('profiles/hd2/pack.yaml')); cats=d['categories']; assert len(cats)>=6, len(cats); names={c['name'] for c in cats}; assert {'Patriotic Administration Center','Orbital Cannons','Hangar','Bridge','Engineering Bay','Robotics Workshop'}<=names, names; total=sum(len(c['macros']) for c in cats); assert total>=75, total; print(f'OK {total} macros across {len(cats)} categories')"

- [ ] **T02: Add tests/pack_hd2_coverage.rs hermetic coverage test** `est:45m`
  Create a new integration test file tests/pack_hd2_coverage.rs that loads profiles/hd2/pack.yaml directly via Pack::load_from_dir() and asserts the bundled HD2 pack covers all current ship-module categories. The test must NOT mutate XDG_CONFIG_HOME, NOT touch the network, NOT depend on model files or audio, and NOT use ProfileManager — it loads the static fixture from the repo path (use `Path::new("profiles/hd2")` relative to the cargo workspace root; integration tests run with CWD = workspace root). Write a single #[test] fn `hd2_pack_covers_all_ship_modules()` that: (1) loads the pack, (2) asserts pack.name == "Helldivers 2", (3) asserts the 6 expected category names are all present (use a HashSet difference for a clear assertion message), (4) asserts per-category minimum macro counts (Administration >=10, Orbital >=12, Hangar >=10, Bridge >=5, Engineering Bay >=8, Robotics Workshop >=6), (5) asserts total flatten().len() >= 75, (6) asserts no category has zero macros (loop and assert each category.macros.len() > 0 with a name in the message). Add a second helper #[test] `hd2_pack_phrases_are_unique()` that flattens and asserts all phrase strings are unique (HashSet collect + len equality). Use the existing patterns in tests/pack_hd2_bundle.rs as a style reference (vibe_attack::pack::Pack import, anyhow::Result return). Do NOT add #[serial] — no env var mutation. Add /// doc comments to both test fns per D002 (every pub item documented; tests are not pub but the module-level //! comment should describe purpose).
  - Files: `tests/pack_hd2_coverage.rs`
  - Verify: cargo test --test pack_hd2_coverage -- --nocapture && cargo test && RUSTFLAGS="-D warnings" cargo check --all-targets

## Files Likely Touched

- profiles/hd2/pack.yaml
- tests/pack_hd2_coverage.rs
