# M009: Pack UX — Editor, Import/Export, Full HD2 Coverage

**Gathered:** 2026-04-27
**Status:** Draft — ready for grilling and decomposition

## Project Description

The pack subsystem (`src/pack/`) is structurally complete: 22 hermetic integration tests prove YAML round-trip, ZIP export/import, ProfileManager state, and full lifecycle. What's missing is the user-facing surface — there is no in-app macro editor, no UI for importing/exporting `.hdpack` files, and the bundled HD2 pack at `profiles/hd2/pack.yaml` only contains **12 macros** when the requirement (PACK-01) calls for the full HD2 stratagem set (80+).

This milestone closes the pack UX gap: a built-in editor, import/export from the config window, and a complete HD2 pack the user gets for free on first run.

## Why This Milestone

Without this, the project is a daemon plus a 12-macro toy — not a usable VoiceAttack-class tool. PACK-04 (built-in editor) is the highest-leverage requirement remaining: every macro a user wants to add today requires hand-editing YAML by file path. The bundled pack gap (PACK-01) is the second: a user installing fresh expects "fire any HD2 stratagem by voice" to work out of the box, not "fire 12 of them, hand-craft the other 68."

The structural foundation is done. This milestone is pure UX work plus content.

## User-Visible Outcome

### When this milestone is complete, the user can:

- Open the config window, click "Edit Pack", and add/edit/delete macros without touching YAML files
- Click "Import Pack" and select a `.hdpack` file shared by another user; the pack is validated and added to their profiles
- Click "Export Pack" and save their current pack as a `.hdpack` file shareable with others
- Speak any of the 80+ HD2 stratagems immediately after installing — the bundled pack covers the full current stratagem set with verified key sequences

### Entry point / environment

- Entry point: `vibe-attack-config` GUI binary (egui)
- Environment: Linux desktop, Wayland or X11
- Live dependencies involved: filesystem (XDG profiles dir), uinput (test injection from editor), `serde_yaml_ng` (round-trip), `zip` (`.hdpack` packaging — already in tree)

## Completion Class

- Contract complete means: editor state (`PackEditor`) has pure-logic CRUD operations with full unit test coverage; `.hdpack` import validates checksums and rejects malformed files with typed errors; the bundled HD2 pack passes a "full coverage" test asserting all current stratagem categories and counts
- Integration complete means: the egui editor panel renders, accepts edits, saves to disk, and the daemon picks up the saved pack via `SwitchProfile` (already shipping) — without a daemon restart
- Operational complete means: a user can complete the full loop "import shared pack → edit it → export modified version → switch profile" without errors; the editor is usable enough that a non-developer can add a new macro from scratch

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- A fresh user installs vibe-attack, switches to the bundled HD2 profile, and successfully fires a stratagem from each major category (Patriotic Administration, Orbital Cannons, Hangar, Bridge, Engineering Bay, Robotics Workshop) by voice — verified manually
- The user adds a new macro via the editor (phrase: "extra ammo", keys: down, down, up, right), saves, and the dispatcher fires it on the next utterance match — verified manually with the test injection feature
- A `.hdpack` file exported from one install imports cleanly into a second install with byte-identical macro content — verified by a hermetic integration test in `tests/pack_lifecycle.rs`
- The HD2 bundled pack passes `tests/pack_hd2_coverage.rs` asserting category names and minimum macro counts match the live stratagem list

## Architectural Decisions

### Editor model — flat list vs nested categories

**Decision:** Editor state mirrors the on-disk YAML structure: top-level Pack → Categories → Macros. The egui UI shows a left-pane category tree and a right-pane macro form. CRUD operations are typed (`AddMacro`, `EditMacro`, `RemoveMacro`, `MoveMacro`, `RenameCategory`, etc.).

**Rationale:** The on-disk format is already nested; flattening in the editor and re-nesting on save would require a transform layer that adds complexity without simplifying anything user-facing. Categories are an HD2 organizational concept (Stratagem ship modules) the user expects.

**Alternatives Considered:**
- Flat list with a "category" tag — simpler editor code but loses the ship-module grouping users see in-game
- Tabbed editor (one tab per category) — fine for HD2 but doesn't generalize to other game packs

### Test injection from the editor

**Decision:** The editor includes a "Test" button per macro that sends a `TriggerMacro { category, name }` control request to the daemon, which dispatches the key sequence as if it had matched a phrase. No new injection path — reuses the existing dispatcher.

**Rationale:** Lets the user verify a key sequence works before committing it, without speaking the phrase repeatedly. The dispatcher already handles all the timing, dwell, and uinput nuance — the editor must not duplicate that logic.

**Alternatives Considered:**
- Direct uinput from the editor binary — duplicates ~200 LOC of dispatcher logic and breaks the "single dispatcher" architectural invariant
- Spoken-phrase preview only — doesn't help the user verify timing for a macro they're still drafting

### Bundled HD2 pack source of truth

**Decision:** The full 80+ stratagem list is sourced from the in-game stratagem reference (Helldivers 2 official wiki at the time of the milestone). The pack file at `profiles/hd2/pack.yaml` is the authoritative copy; a `tests/pack_hd2_coverage.rs` test asserts category and minimum-count invariants so future edits can't silently drop entries.

**Rationale:** Without a coverage test, the bundled pack will drift. A simple "category exists + min N macros per category" test catches accidental deletion without locking down individual key sequences (which the user may want to customize).

**Alternatives Considered:**
- No coverage test — cheap now, costs us silently when someone breaks the pack later
- Lock down each macro by snapshot — too brittle; users edit the bundled pack as a starting point

---

## Error Handling Strategy

`.hdpack` import errors are typed: `InvalidChecksum`, `MalformedYaml`, `IncompatibleVersion`, `NameCollision`. The UI surfaces each as a clear modal with the actionable next step ("Pack name 'HD2' already exists — rename or replace?"). Editor save errors (disk full, permission denied) leave the in-memory state intact and show a non-modal status message; the user can retry. Test injection errors (daemon not running) surface in a status bar; the editor stays usable for further edits.

## Risks and Unknowns

- Egui table/tree widgets are functional but not polished — the editor will look utilitarian, not slick. Acceptable for v1.
- HD2 stratagem list changes with game patches — coverage test must allow for incremental additions, not lock to an exact count
- `.hdpack` ZIP format already exists in `src/pack/`; verify version field handling for forward compatibility before the import UI ships
- Editing a macro while it's the active phrase the user just spoke could race — needs clean swap-on-save semantics in the dispatcher
- The "Test" button issuing a uinput key sequence could bypass the user's intent if they click it while an in-game window is focused — guard with a 1-second countdown or focus check

## Existing Codebase / Prior Art

- `src/pack/mod.rs` — `Pack`, `Category`, `Macro` types with serde + YAML round-trip; `Pack::load_from_dir`, `save_to_dir`, ZIP `.hdpack` export
- `src/pack/manager.rs` — `ProfileManager` for active-pack tracking, persisted at `manager.yaml`
- `tests/pack_hd2_bundle.rs` — 22 lifecycle tests; extend with full-coverage test
- `profiles/hd2/pack.yaml` — current bundled pack; 12 macros across 3 categories; expand to full 80+ across all ship-module categories
- `src/control/protocol.rs` — control-plane request enum; add `TriggerMacro { category, name }` for editor test
- `src/pipeline/dispatcher.rs` — handles macro execution; new control path mirrors phrase-match path
- `src/ui/config_app.rs` + `vibe-attack-config` egui host — editor panel attaches here

## Relevant Requirements

- **PACK-01** — bundled HD2 pack with all 80+ stratagems — primary objective; current pack has 12
- **PACK-02** — import `.hdpack` files — primary objective; structural foundation done, UI missing
- **PACK-03** — export `.hdpack` files — primary objective; structural foundation done, UI missing
- **PACK-04** — built-in macro editor — primary objective; entire feature
- **PACK-05** — multiple named profiles, runtime switch — partial; profile switch via tray ships in M008 (or already shipping); editor must respect profile boundaries

## Scope

### In Scope

- Egui editor panel: category tree, macro form, add/edit/delete/move/rename operations
- Test-macro button in editor that issues `TriggerMacro` to the daemon
- Import dialog: file picker → checksum validation → name-collision resolution → save to profiles dir
- Export dialog: select profile → write `.hdpack` to user-chosen path
- Full HD2 stratagem pack: every current ship-module category, every current stratagem with verified key sequence
- `tests/pack_hd2_coverage.rs` — category and min-count invariants
- Hermetic integration test for export → import round trip with byte-identical content

### Out of Scope / Non-Goals

- Macro conditional logic / variables (MCRO-03) — out of scope
- Per-macro sound feedback (MCRO-04) — out of scope (already a global setting)
- Pack repository / online sharing — out of scope; users share `.hdpack` files manually
- Pack diffing / merge — out of scope
- Multi-game packs beyond HD2 — out of scope as content; the architecture already supports them
- Internationalization — out of scope

## Technical Constraints

- All existing tests must pass
- `cargo clippy -D warnings` clean for both default and `gui` feature sets
- The `.hdpack` format must remain backward-compatible with packs exported by existing tests
- Editor must compile under `gui` feature only; default-feature builds must be unaffected
- Test injection MUST go through the existing dispatcher — never a direct uinput call from the editor binary

## Integration Points

- Control plane — add `TriggerMacro` request and handler
- Filesystem — XDG profiles dir for read/write; user-chosen paths for import/export
- `serde_yaml_ng` and `zip` — already in tree; no version bumps
- Dispatcher — must accept a control-plane-driven trigger as a first-class path equivalent to phrase match

## Testing Requirements

- Unit tests for `PackEditor` CRUD operations
- Round-trip test: edit → save → reload → byte equivalence
- Integration test: export → import → byte-identical macros
- Coverage test for bundled HD2 pack
- Manual UAT script: install fresh → fire one stratagem from each ship-module category → confirmed working
- Manual UAT script: edit a macro, save, confirm dispatcher uses new key sequence on next match

## Acceptance Criteria

(Per-slice — to be refined during decomposition)

- Editor: every CRUD op has a test
- Import: malformed `.hdpack` produces a clear typed error and does not corrupt the profiles dir
- Export: round-trip produces byte-identical YAML inside the ZIP
- HD2 pack: coverage test passes; manual UAT confirms one stratagem per category fires correctly
- Test injection: triggers the dispatcher with no panic when the daemon is mid-utterance

## Open Questions

- File picker — egui doesn't ship a native one; use `rfd` (already in tree?) or shell out to `zenity` / `kdialog`? — current thinking: `rfd` if not already in tree
- HD2 pack source — manually transcribed from the wiki, or do we ship a "fetch from wiki" tool? — current thinking: manual transcription, locked into the milestone work
- Test-macro button safety — countdown timer, focus check, or just trust the user? — current thinking: 1-second confirmation prompt for first-time use, persistent dismiss

## Suggested Slice Decomposition

- **S01** (low risk, depends:[]): Bundled HD2 pack — full stratagem list + coverage test (no UI work)
- **S02** (medium risk, depends:[]): `PackEditor` pure-logic state + CRUD operations + tests
- **S03** (medium risk, depends:[S02]): Egui editor panel — category tree, macro form, save flow
- **S04** (low risk, depends:[S03]): Import / Export dialogs — file picker, checksum validation, error UX
- **S05** (low risk, depends:[S02]): `TriggerMacro` control request + dispatcher integration + editor Test button
- **S06** (low risk, depends:[S01,S03,S04,S05]): UAT — manual script covering full pack lifecycle and editor flow
