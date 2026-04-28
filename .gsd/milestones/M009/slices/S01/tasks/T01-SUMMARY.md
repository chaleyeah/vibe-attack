---
id: T01
parent: S01
milestone: M009
key_files:
  - profiles/hd2/pack.yaml
key_decisions:
  - Used KEY_UP/KEY_DOWN/KEY_LEFT/KEY_RIGHT exclusively (no WASD) per task plan constraint
  - Spear sequence chosen as ↓↓↑↓↓ (canonical in-game input)
  - Bridge category includes Reinforce and Resupply (cross-module stratagems that ship with the Bridge in-game)
duration: 
verification_result: passed
completed_at: 2026-04-28T02:23:09.372Z
blocker_discovered: false
---

# T01: Expanded profiles/hd2/pack.yaml from 12 to 75 stratagems across all 6 ship-module categories using canonical HD2 arrow-key sequences

**Expanded profiles/hd2/pack.yaml from 12 to 75 stratagems across all 6 ship-module categories using canonical HD2 arrow-key sequences**

## What Happened

Rewrote profiles/hd2/pack.yaml to contain exactly 75 Helldivers 2 stratagems organized into the 6 required ship-module categories. The existing 12 entries (Machine Gun, Anti-Material Rifle, Stalwart, Orbital Gatling Barrage, Orbital Airburst Strike, Orbital 120mm HE Barrage, Eagle Strafing Run, Eagle Airstrike, Eagle Cluster Bomb) were preserved verbatim where canonical. Each category now meets its minimum: Patriotic Administration Center (17 ≥ 10), Orbital Cannons (13 ≥ 12), Hangar (12 ≥ 10), Bridge (9 ≥ 5), Engineering Bay (12 ≥ 8), Robotics Workshop (12 ≥ 6). All key sequences use KEY_UP/KEY_DOWN/KEY_LEFT/KEY_RIGHT evdev names exclusively — no WASD mixing. All 75 phrases are unique and lowercase. The YAML schema is unchanged: top-level name + author + categories[].macros[] with name/phrase/keys where each key is a KeyAction {key: evdev_name}. Notable additions: Orbital Railcannon Strike (→↑↓↓→), Eagle 500kg Bomb (↑→↓↓↓), Reinforce (↑↓→←↑), Resupply (↓↓↑→), Patriot/Emancipator Exosuits, and full sentry complement. A malformed Spear entry (typo {key} instead of {key: KEY_UP}) in an early draft was caught and fixed before the bundle test run.

## Verification

Ran full T01 verification command: `cargo test --test pack_hd2_bundle` (22/22 pass) followed by the Python assertion script that checks: ≥6 categories, all 6 required category names present, total ≥75 macros. Both passed. Also ran a local Python validation confirming all phrases are unique and all per-category minimums are met.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test pack_hd2_bundle` | 0 | ✅ pass | 80ms |
| 2 | `python3 -c "import yaml; d=yaml.safe_load(open('profiles/hd2/pack.yaml')); cats=d['categories']; assert len(cats)>=6; names={c['name'] for c in cats}; assert {'Patriotic Administration Center','Orbital Cannons','Hangar','Bridge','Engineering Bay','Robotics Workshop'}<=names; total=sum(len(c['macros']) for c in cats); assert total>=75; print(f'OK {total} macros across {len(cats)} categories')"` | 0 | ✅ pass — OK 75 macros across 6 categories | 50ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `profiles/hd2/pack.yaml`
