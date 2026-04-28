# M009: Pack UX — Editor, Import/Export, Full HD2 Coverage

**Vision:** A user can open the config window, edit macros with a built-in editor, import/export .hdpack files, and fire any of the 80+ Helldivers 2 stratagems by voice out of the box. The pack subsystem is structurally complete; this milestone adds the user-facing surface and the full HD2 content.

## Success Criteria

- cargo test passes at end of every slice
- cargo clippy -D warnings clean for both default and gui feature sets
- ProfileManager and Pack types are exercised by all existing 22 integration tests plus new ones
- PackEditor pure-logic state has unit tests for every CRUD operation (AddMacro, EditMacro, RemoveMacro, MoveMacro, RenameCategory)
- Export → import round-trip produces byte-identical macro content — hermetic test in tests/pack_lifecycle.rs
- tests/pack_hd2_coverage.rs asserts all ship-module category names present and minimum macro counts per category
- TriggerMacro control request routes through the existing dispatcher (never a direct uinput call from editor binary)
- Egui editor panel compiles only under gui feature; default-feature build unaffected
- HD2 bundled pack at profiles/hd2/pack.yaml covers all current stratagem categories with verified key sequences
- Import of a malformed .hdpack produces a clear typed error and leaves profiles dir uncorrupted

## Slices

- [x] **S01: S01** `risk:low` `depends:[]`
  > After this: cargo test --test pack_hd2_coverage passes; grep on profiles/hd2/pack.yaml shows all 6+ ship-module categories present with correct stratagem counts

- [x] **S02: S02** `risk:medium` `depends:[]`
  > After this: cargo test passes including new PackEditor unit tests for AddMacro, EditMacro, RemoveMacro, MoveMacro, RenameCategory, AddCategory, RemoveCategory; round-trip: edit → save → reload → byte equivalence

- [x] **S03: S03** `risk:medium` `depends:[]`
  > After this: vibe-attack-config opens editor panel; user adds a new macro via the form; clicks Save; pack.yaml updated on disk; daemon picks up change via SwitchProfile (already shipping)

- [x] **S04: S04** `risk:low` `depends:[]`
  > After this: User clicks Import → picks a .hdpack → pack appears in profile list; user clicks Export → picks a destination → .hdpack written; round-trip import of exported file produces byte-identical macros

- [ ] **S05: S05** `risk:low` `depends:[]`
  > After this: User opens editor, selects a macro, clicks Test; 1-second confirmation prompt; daemon fires the key sequence via uinput; dispatcher JSONL output shows the triggered macro

- [ ] **S06: UAT — full pack lifecycle and editor flow** `risk:low` `depends:[S01,S03,S04,S05]`
  > After this: S06-UAT.md manual steps all pass; cargo test passes including pack_hd2_coverage and pack_lifecycle

## Boundary Map

## Boundary Map

### Internal boundaries touched
- **profiles/hd2/pack.yaml** — expand from 12 to 80+ macros across all ship-module categories
- **src/pack/mod.rs** — add PackEditor struct with CRUD ops; no changes to Pack/Category/Macro serde format
- **src/control/protocol.rs** — add TriggerMacro { category, name } to ControlRequest
- **src/control/mod.rs** — TriggerMacro server handler routes through existing dispatcher
- **src/pipeline/dispatcher.rs** — accept control-plane-driven trigger as first-class path
- **src/ui/config_app.rs** — editor panel integration, import/export button state
- **src/bin/vibe-attack-config.rs** — editor panel routing; rfd file picker calls
- **tests/pack_hd2_coverage.rs** — new test file
- **tests/pack_lifecycle.rs** — new test file (or extend pack_hd2_bundle.rs)

### Untouched (explicitly out of scope)
- .hdpack ZIP format (backward-compatible; no schema changes)
- Control plane protocol for existing requests
- Macro conditional logic (MCRO-03)
- Per-macro sound feedback (MCRO-04)
- All packaging files
