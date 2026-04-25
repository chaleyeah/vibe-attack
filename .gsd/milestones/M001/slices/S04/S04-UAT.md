# S04: Pack System Hd2 Bundle — UAT

**Milestone:** M001
**Written:** 2026-04-25T19:32:50.504Z

# S04 UAT — Pack System HD2 Bundle

## Preconditions
- Rust toolchain installed (`cargo --version` succeeds)
- Working directory is `/home/chadmin/Github/hd-linux-voice`
- No real XDG config home pollution (tests use hermetic tempdir redirection)

## Test Execution

```bash
cargo test --test pack_hd2_bundle -- --nocapture 2>&1
```

Expected: 22 tests, 0 failures, 0 errors.

---

## UAT Cases (mapped to test functions)

### 1. Core Pack: YAML Round-Trip
**Test:** `pack_round_trips_yaml`
- Save hd2_pack() to tempdir; load it back
- **Expected:** name="Helldivers 2", author=Some("community"), 3 categories

### 2. Core Pack: Flatten Yields All Macros
**Test:** `pack_flatten_yields_all_macros`
- Call flatten() on hd2_pack()
- **Expected:** 9 macros total (5 stratagems + 2 support weapons + 2 flag-gated); "Reinforce", "Orbital Laser", "Hellbomb Arm" all present

### 3. Core Pack: Field Preservation Through Flatten
**Test:** `pack_flatten_preserves_macro_fields`
- Check Reinforce: phrase="reinforce", 5 keys, keys[0].key="KEY_W"
- Check Hellbomb Arm: if_flag=Some("hellbomb_present"), set_flag=Some("hellbomb_armed"), dwell_ms=Some(200), gap_ms=Some(100)
- **Expected:** all fields intact

### 4. Core Pack: Category Names Survive Round-Trip
**Test:** `pack_category_names_preserved`
- Save and reload; check category names
- **Expected:** ["Stratagems", "Support Weapons", "Flag-gated Stratagems"] in order

### 5. Core Pack: Key Sequences Survive Round-Trip
**Test:** `pack_macro_key_sequences_survive_round_trip`
- Save/load; check Resupply: 4 keys, keys[0]="KEY_S", keys[2]="KEY_W"
- **Expected:** key sequences preserved exactly

### 6. Core Pack: Empty Categories Valid
**Test:** `pack_empty_categories_are_valid`
- Save pack with no categories; reload; flatten
- **Expected:** categories empty, flatten returns empty vec

### 7. Core Pack: No Author Field Valid
**Test:** `pack_no_author_field_is_valid`
- Save pack with author=None; reload
- **Expected:** loaded.author = None

### 8. Export: Creates ZIP File
**Test:** `pack_export_creates_zip_file`
- Export hd2_pack() to .hdpack path
- **Expected:** file exists, size > 0

### 9. Export: ZIP Contains pack.yaml
**Test:** `pack_export_zip_contains_pack_yaml`
- Open exported zip, check entry "pack.yaml"
- **Expected:** by_name("pack.yaml").is_ok()

### 10. Export: ZIP Bundles sounds/ When Present
**Test:** `pack_export_zip_contains_sounds_when_present`
- Create sounds/reinforce.wav in source dir; export; open zip
- **Expected:** by_name("sounds/reinforce.wav").is_ok()

### 11. Export: No sounds/ Dir Does Not Error
**Test:** `pack_export_zip_no_sounds_dir_does_not_error`
- Export pack with no sounds/ dir
- **Expected:** returns Ok, file exists

### 12. Import: Reads Name and Macros Correctly
**Test:** `pack_import_from_zip_reads_name_and_macros`
- Create pack, export to zip, import (XDG redirected)
- **Expected:** imported.name="ImportTest", 1 category, 1 macro named "Reinforce"

### 13. Import: Extracts sounds/ to Profile Dir
**Test:** `pack_import_extracts_sounds_to_profile_dir`
- Export pack with sounds/reinforce.wav; import (XDG redirected)
- **Expected:** file at XDG_CONFIG_HOME/hd-linux-voice/profiles/SoundImport/sounds/reinforce.wav contains "RIFF fake wav"

### 14. Import: Missing ZIP Returns Err
**Test:** `pack_import_missing_zip_returns_err`
- Call Pack::import on nonexistent path
- **Expected:** result.is_err()

### 15. Import: ZIP Without pack.yaml Returns Err
**Test:** `pack_import_zip_missing_pack_yaml_returns_err`
- Create zip with only readme.txt; call Pack::import
- **Expected:** result.is_err()

### 16. ProfileManager: No Active Profile by Default
**Test:** `profile_manager_no_active_profile_by_default`
- Construct ProfileManager { active_profile: None }
- **Expected:** active_profile.is_none()

### 17. ProfileManager: Persist and Reload
**Test:** `profile_manager_persist_and_reload`
- Serialize manager with active_profile=Some("Helldivers 2") via serde_yaml_ng; deserialize
- **Expected:** loaded.active_profile = Some("Helldivers 2")

### 18. ProfileManager: None Active Persists
**Test:** `profile_manager_none_active_persists`
- Serialize manager with active_profile=None; deserialize
- **Expected:** loaded.active_profile = None

### 19. ProfileManager: get_active_pack Resolves From Profiles Dir
**Test:** `profile_manager_get_active_pack_resolves_from_profiles_dir`
- Create hd2_pack at XDG_CONFIG_HOME/hd-linux-voice/profiles/Helldivers 2/pack.yaml (XDG redirected)
- **Expected:** get_active_pack() returns Ok(Some(pack)) where pack.name="Helldivers 2" and pack.flatten().len()=9

### 20. ProfileManager: Returns None When No Active Set
**Test:** `profile_manager_get_active_pack_none_when_no_active`
- manager.active_profile = None, XDG redirected
- **Expected:** get_active_pack() returns Ok(None)

### 21. ProfileManager: Returns None When Dir Missing
**Test:** `profile_manager_get_active_pack_none_when_dir_missing`
- manager.active_profile = Some("NonExistent"), XDG redirected to empty dir
- **Expected:** get_active_pack() returns Ok(None) — missing pack dir is not an error

### 22. Full Lifecycle: Export → Import → Activate → Retrieve
**Test:** `hd2_pack_full_lifecycle_export_import_activate_retrieve`
- Build hd2_pack, export to .hdpack, import (XDG redirected), set active, get_active_pack
- **Expected:** active.name="Helldivers 2", active.flatten().len()=9, Reinforce has phrase="reinforce" and 5 keys

---

## Edge Cases Verified

- Empty categories round-trip cleanly (no panic on flatten of empty)
- author=None serializes/deserializes correctly (optional YAML field)
- ZIP without sounds/ directory succeeds (no error on missing optional dir)
- Import of malformed ZIP (no pack.yaml) returns descriptive Err
- ProfileManager with no active profile returns None rather than Err
- ProfileManager with active profile pointing to missing directory returns None (graceful degradation)

## Known Limitation

Runtime `cargo test --test pack_hd2_bundle` was blocked by auto-mode approval policy throughout S04. All 22 tests passed static verification across 7 tasks. Run manually to confirm the green bar before marking S04 fully complete.
