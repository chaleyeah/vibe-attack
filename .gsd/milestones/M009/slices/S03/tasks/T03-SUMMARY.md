---
id: T03
parent: S03
milestone: M009
key_files:
  - tests/pack_editor_state_roundtrip.rs
key_decisions:
  - Integration test drives PackEditor directly (not PackEditorState) because PackEditorState is #[cfg(feature = gui)]-gated and calls send_command — PackEditor IS the state machine layer; PackEditorState adds only egui UI fields around it
  - remove_category precondition (must be empty) requires explicit remove_macro calls for remaining macros before category removal — move_macro only moves one macro, leaving others in the source category
  - No SwitchProfile exercised in tests — save-to-disk half is the integration-closure proof per task plan; daemon not available in CI
duration: 
verification_result: passed
completed_at: 2026-04-28T02:52:21.155Z
blocker_discovered: false
---

# T03: Added state-machine roundtrip integration tests (3 tests) and confirmed full slice acceptance: all builds clean, zero warnings, 87+ tests pass including pack_editor_state_roundtrip suite

**Added state-machine roundtrip integration tests (3 tests) and confirmed full slice acceptance: all builds clean, zero warnings, 87+ tests pass including pack_editor_state_roundtrip suite**

## What Happened

Created `tests/pack_editor_state_roundtrip.rs` with three integration tests that prove the PackEditor state-machine layer (the mutation calls the egui UI buttons drive) survives a full save → reload cycle.

**`state_roundtrip_after_full_crud_via_state_layer`:** Builds a `fixture_pack()` (two categories, two macros each), wraps in `PackEditor::new`, then drives the full mutation sequence: `add_category("NewCat")`, `add_macro("NewCat", ...)`, `edit_macro` (phrase update), `move_macro("Cat1" → "NewCat", "orig_macro")`, `rename_category("Cat2" → "Cat2Renamed")`, `remove_macro("NewCat", "orig_macro")`, `remove_macro("Cat1", "m1")`, `remove_category("Cat1")`. Creates the profile_dir via `fs::create_dir_all`, calls `pack().save_to_dir`, reloads with `Pack::load_from_dir`, and verifies structural equality via `assert_packs_equal`. The `remove_category` precondition (category must be empty) required removing `m1` from `Cat1` before the category removal — `orig_macro` had been moved out but `m1` remained.

**`state_parse_key_sequence_drives_form_to_save`:** Uses the public `parse_key_sequence("KEY_W, KEY_A, KEY_S, KEY_D")` helper to build a `Vec<KeyAction>`, constructs a `MacroConfig` with those keys, adds it via the editor, saves and reloads, and asserts all 4 keys survive with key strings intact and `dwell_ms`/`gap_ms` remaining `None`.

**`state_save_to_dir_writes_pack_yaml`:** Smoke test that verifies `pack.yaml` exists after save and round-trips cleanly through `serde_yaml_ng::from_str::<Pack>`.

The `assert_packs_equal` helper mirrors the S02 pattern: explicit field walk through name, author, categories.len(), category names, macros.len(), and every `MacroConfig` field (name, phrase, if_flag, set_flag, sound, keys with key/dwell_ms/gap_ms).

**Design note:** `PackEditorState` is `#[cfg(feature = "gui")]`-gated and calls `send_command` (also gui-only), so the integration test targets `PackEditor` directly — the same object `PackEditorState::editor` wraps. This is the correct closure proof: `PackEditor` IS the state machine; `PackEditorState` adds only egui UI fields around it. No SwitchProfile exercised (no daemon in CI), per the task plan.

**Slice acceptance:** `cargo build` (clean), `cargo build --features gui` (clean), `RUSTFLAGS="-D warnings" cargo build --all-targets --features gui` (zero warnings), `cargo test -- --test-threads=1` (all suites green, 0 failures).

## Verification

Ran four verification gates in sequence: (1) `cargo build` — clean, no features; (2) `cargo build --features gui` — clean; (3) `RUSTFLAGS="-D warnings" cargo build --all-targets --features gui` — clean, zero warnings (clippy not installed in this environment; RUSTFLAGS=-D warnings is the effective substitute per T02); (4) `cargo test -- --test-threads=1` — all test suites green, 87 tests passed (lib) + all integration test suites passed including the 3 new tests in pack_editor_state_roundtrip, 0 failures total.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build` | 0 | ✅ pass — clean compile, no features | 2160ms |
| 2 | `cargo build --features gui` | 0 | ✅ pass — clean compile with gui feature | 7250ms |
| 3 | `RUSTFLAGS="-D warnings" cargo build --all-targets --features gui` | 0 | ✅ pass — zero warnings across all targets | 14830ms |
| 4 | `cargo test -- --test-threads=1` | 0 | ✅ pass — all test suites green, 0 failures; 3 new state-roundtrip tests + all prior tests pass | 180000ms |

## Deviations

PackEditorState::new was specified in the task plan but is #[cfg(feature = gui)]-gated and depends on send_command. Tests drive PackEditor directly instead — this is equivalent since PackEditorState::editor IS the state machine, and the verification contract (edit → save → reload → equality) is identical. Also, remove_category('Cat1') required an explicit remove_macro('Cat1', 'm1') first — the task plan mutation sequence did not include this step but the PackEditor API enforces category-must-be-empty, which is correct behavior. The additional remove_macro call was added to satisfy the contract.

## Known Issues

none

## Files Created/Modified

- `tests/pack_editor_state_roundtrip.rs`
