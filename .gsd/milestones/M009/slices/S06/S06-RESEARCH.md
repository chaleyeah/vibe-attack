# S06 — UAT: Full Pack Lifecycle and Editor Flow

**Date:** 2026-04-27
**Slice:** S06 (depends: S01, S03, S04, S05)

## Summary

S06 is a pure verification slice — no new production code is written. All four dependency slices (S01, S03, S04, S05) are confirmed complete and green. The automated test suite passes in full: `pack_hd2_coverage` (2 tests), `pack_lifecycle` (2 tests), `control_integration` (4 tests), `pack_editor_state_roundtrip` (3 tests), and `pack_editor_roundtrip` (6 tests). The `--features gui` build is clean. The sole deliverable of S06 is producing and signing off `S06-UAT.md` — a manual script covering the five user-facing scenarios that cannot be exercised headlessly.

This is straightforward: write the UAT script document with precise manual steps, run `cargo test -- --test-threads=1` one final time to capture the pass evidence, and mark the slice complete.

## Recommendation

One task: write `S06-UAT.md` capturing the manual UAT steps, then execute `cargo test` and record the result as the automated evidence block. The UAT steps match the M009 milestone's "Final Integrated Acceptance" criteria verbatim — no new scope.

The planner should produce a single task (T01) that: (1) writes `S06-UAT.md` with all manual steps and the automated evidence, and (2) marks S06 complete via `gsd_slice_complete`.

## Implementation Landscape

### Key Files

- `.gsd/milestones/M009/slices/S06/S06-UAT.md` — does not yet exist; this is the primary artifact to produce
- `profiles/hd2/pack.yaml` — 75 stratagems across 6 categories; the "fire one stratagem from each category" UAT step reads this to know what to speak
- `tests/pack_hd2_coverage.rs` — automated: `hd2_pack_covers_all_ship_modules`, `hd2_pack_phrases_are_unique` — both passing
- `tests/pack_lifecycle.rs` — automated: `pack_export_then_import_to_round_trips_macros`, `pack_export_imports_sounds_subdirectory` — both passing
- `src/ui/pack_editor.rs` — contains `PackEditorState`, `show_pack_editor`, Test button logic (countdown, `send_command(TestMacro{name})`)
- `src/bin/vibe-attack-config.rs` — wires `show_pack_editor`, drains `imported_pack_name`, calls `load_profiles()` on import
- `src/control/mod.rs` — `TestMacro` handler calls `Dispatcher::fire_named` via `block_in_place`
- `src/pipeline/dispatcher.rs` — `fire_named(&str)` returns `Ok(DispatchOutcome::Fired { score: 1.0 })` or `Err("macro not found: ...")`

### Build Order

1. Write `S06-UAT.md` (the manual script + automated evidence). No code changes required.
2. Run `cargo test -- --test-threads=1` and paste summary into the UAT doc as automated evidence.
3. Call `gsd_slice_complete` for S06.

### Verification Approach

**Automated (run as part of T01):**
```
cargo test -- --test-threads=1
```
Expected: all suites green, 0 failures, 4–7 ignored (privileged/hardware-only).

**Manual UAT steps (captured in S06-UAT.md, executed by a human tester):**

1. **HD2 stratagem fire by voice** — start daemon + config, switch to hd2 profile, speak one stratagem phrase from each of the 6 ship-module categories (Patriotic Administration Center, Orbital Cannons, Hangar, Bridge, Engineering Bay, Robotics Workshop), confirm keystrokes injected via uinput.

2. **Add macro via editor and fire it** — open config window, click hd2 pack, add macro (phrase: "extra ammo", keys: `KEY_DOWN,KEY_DOWN,KEY_UP,KEY_RIGHT`), click Save, speak "extra ammo", confirm keys injected.

3. **Test button (1-second countdown)** — select any macro in editor, click Test button, observe 1-second countdown label, confirm fire, check daemon JSONL output for `DispatchOutcome::Fired`.

4. **Export → Import round-trip (GUI)** — export hd2 pack to `/tmp/hd2-test.hdpack`, import it into a fresh profile slot, confirm profile appears in list, open it and verify all macros present.

5. **Malformed import rejection** — create a corrupted `.hdpack` (truncated ZIP), attempt import, confirm typed error appears inline in editor (not a crash), profiles dir uncorrupted.

## Constraints

- `cargo clippy` is not available as a subcommand in this environment (rustup/clippy component not installed); both `cargo build` and `cargo build --features gui` are clean with zero rustc warnings — this is the accepted substitute per prior slice conventions.
- rfd file dialog paths cannot be driven headlessly — UAT steps 4 and 5 require a physical desktop session (Wayland or X11).
- The Test button is gated by `daemon_running: bool`; UAT step 3 requires the daemon to be running.
- `block_in_place` in the TestMacro handler requires the `multi_thread` Tokio flavor — already confirmed working in `control_integration` tests.
