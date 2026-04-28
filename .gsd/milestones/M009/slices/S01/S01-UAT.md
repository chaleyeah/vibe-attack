# S01: Full HD2 stratagem pack + coverage test — UAT

**Milestone:** M009
**Written:** 2026-04-28T02:26:41.227Z

# S01: Full HD2 stratagem pack + coverage test — UAT

**Milestone:** M009
**Written:** 2026-04-27

## UAT Type

- UAT mode: artifact-driven
- Why this mode is sufficient: S01 is a pure data + test slice — no runtime daemon, no UI. The cargo test suite is the complete acceptance gate.

## Preconditions

- Working Rust toolchain with cargo available
- Workspace root is /home/chadmin/Github/hd-linux-voice
- profiles/hd2/pack.yaml is the file under test

## Smoke Test

Run `cargo test --test pack_hd2_coverage` — both tests must pass in under 2 seconds.

## Test Cases

### 1. Coverage test passes: all 6 categories present with minimum macro counts

1. From workspace root, run: `cargo test --test pack_hd2_coverage -- --nocapture`
2. **Expected:** `running 2 tests`, `test hd2_pack_covers_all_ship_modules ... ok`, `test hd2_pack_phrases_are_unique ... ok`, exit 0

### 2. All 6 ship-module categories are present in pack.yaml

1. Run: `python3 -c "import yaml; d=yaml.safe_load(open('profiles/hd2/pack.yaml')); names={c['name'] for c in d['categories']}; required={'Patriotic Administration Center','Orbital Cannons','Hangar','Bridge','Engineering Bay','Robotics Workshop'}; missing=required-names; assert not missing, f'Missing: {missing}'; print('OK all 6 categories present')"`
2. **Expected:** `OK all 6 categories present`

### 3. Total macro count >= 75

1. Run: `python3 -c "import yaml; d=yaml.safe_load(open('profiles/hd2/pack.yaml')); total=sum(len(c['macros']) for c in d['categories']); assert total>=75, total; print(f'OK {total} macros')"`
2. **Expected:** `OK 75 macros` (or higher)

### 4. Full test suite remains green

1. Run: `cargo test -- --test-threads=1`
2. **Expected:** All tests pass; no regressions in pack_hd2_bundle or other integration tests

### 5. Clean compile with warnings-as-errors

1. Run: `RUSTFLAGS="-D warnings" cargo check --all-targets`
2. **Expected:** `Finished` with no warnings, exit 0

## Edge Cases

### Category name exactly matches expected strings

1. Run: `python3 -c "import yaml; d=yaml.safe_load(open('profiles/hd2/pack.yaml')); names=[c['name'] for c in d['categories']]; print(names)"`
2. **Expected:** All 6 names appear exactly as: `Patriotic Administration Center`, `Orbital Cannons`, `Hangar`, `Bridge`, `Engineering Bay`, `Robotics Workshop` — no typos, no extra spaces

### All phrases are unique (no duplicate voice triggers)

1. Run: `cargo test --test pack_hd2_coverage hd2_pack_phrases_are_unique -- --nocapture`
2. **Expected:** `test hd2_pack_phrases_are_unique ... ok`

### All key sequences use arrow keys only (no WASD)

1. Run: `python3 -c "import yaml; d=yaml.safe_load(open('profiles/hd2/pack.yaml')); wasd=[(c['name'],m['name'],k) for c in d['categories'] for m in c['macros'] for k in m['keys'] if k.get('key','') in {'KEY_W','KEY_A','KEY_S','KEY_D'}]; assert not wasd, f'WASD found: {wasd}'; print('OK no WASD keys')"`
2. **Expected:** `OK no WASD keys`

## Failure Signals

- `hd2_pack_covers_all_ship_modules` test failure naming absent categories → a category was removed or renamed in pack.yaml
- `hd2_pack_phrases_are_unique` test failure → duplicate phrase string introduced
- `cargo check` warnings → new code introduced without cleaning up

## Not Proven By This UAT

- Runtime daemon correctly dispatches macros from the HD2 pack (covered by S05/S06)
- Import/export round-trip of the pack (covered by S04)
- Editor UI rendering the HD2 pack categories (covered by S03)
- Actual key sequence accuracy against live Helldivers 2 game (requires game + hardware)

## Notes for Tester

The per-category counts in the test are hard minimums. The test will still pass if future slices add more macros to any category — it only fails if counts drop below the minimums. The Spear stratagem sequence (↓↓↑↓↓) differs from some community references; the canonical in-game input was used.
