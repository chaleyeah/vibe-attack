---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Expand profiles/hd2/pack.yaml to full 6-category HD2 stratagem coverage

Rewrite profiles/hd2/pack.yaml so it contains 75+ Helldivers 2 stratagems organized into the 6 ship-module categories the roadmap requires: Patriotic Administration Center (>=10 support weapons & defensive items), Orbital Cannons (>=12 orbital strikes), Hangar (>=10 eagle stratagems), Bridge (>=5 vehicle/mech stratagems), Engineering Bay (>=8 defensive structures & supply), Robotics Workshop (>=6 autonomous units). Keep YAML schema identical to the existing file (top-level name + author + categories[].macros[] with name/phrase/keys; KeyAction objects with `key:` evdev names). Use arrow-key sequences (KEY_UP, KEY_DOWN, KEY_LEFT, KEY_RIGHT) for consistency with the existing 12 entries — do NOT mix with KEY_W/KEY_A/KEY_S/KEY_D. Phrases must be unique within the pack and lowercase, matching the in-game stratagem name (e.g. phrase: orbital railcannon strike). Preserve existing 12 entries verbatim where they remain canonical (Machine Gun, Anti-Material Rifle, Stalwart, Orbital Gatling Barrage, Orbital Airburst Strike, Orbital 120mm HE Barrage, Eagle Strafing Run, Eagle Airstrike, Eagle Cluster Bomb). Source key sequences from the canonical Helldivers 2 stratagem reference (e.g. Reinforce ↑↓→←↑, Resupply ↓↓↑→, 500kg Bomb ↑→↓↓↓, Orbital Railcannon Strike →↑↓↓→). Validate YAML by running `cargo run --bin vibe-attack -- --help` (which loads no profiles, but compiles) AND by running `cargo test --test pack_hd2_bundle` — the bundle test loads pack.yaml in several places so it will fail on a malformed file. After expansion, the file should be approximately 200-350 lines of YAML.

## Inputs

- ``profiles/hd2/pack.yaml``
- ``src/pack/mod.rs``
- ``src/config.rs``

## Expected Output

- ``profiles/hd2/pack.yaml``

## Verification

cargo test --test pack_hd2_bundle && python3 -c "import yaml; d=yaml.safe_load(open('profiles/hd2/pack.yaml')); cats=d['categories']; assert len(cats)>=6, len(cats); names={c['name'] for c in cats}; assert {'Patriotic Administration Center','Orbital Cannons','Hangar','Bridge','Engineering Bay','Robotics Workshop'}<=names, names; total=sum(len(c['macros']) for c in cats); assert total>=75, total; print(f'OK {total} macros across {len(cats)} categories')"
