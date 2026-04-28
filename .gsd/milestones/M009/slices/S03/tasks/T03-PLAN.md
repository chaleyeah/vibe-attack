---
estimated_steps: 5
estimated_files: 1
skills_used: []
---

# T03: Add state-machine roundtrip integration test and verify full slice acceptance

Create `tests/pack_editor_state_roundtrip.rs` that proves the editor state machine performs an end-to-end edit → save → reload → equivalence cycle. Reuse the helper-shape pattern from `tests/pack_editor_roundtrip.rs` (S02): define a private `fn assert_packs_equal(left: &Pack, right: &Pack)` that walks `name`, `author`, `categories.len()`, then for each category walks `name`, `macros.len()`, then for each macro walks every `MacroConfig` field (name, phrase, if_flag, set_flag, sound, keys with KeyAction.key/dwell_ms/gap_ms). Define a `fn fixture_pack() -> Pack` with two categories and two macros each, mirroring the S02 fixture. Tests required:
- `state_roundtrip_after_full_crud_via_state_layer`: build a fixture Pack, wrap in `PackEditor::new`, then `PackEditorState::new(editor, tempdir.path().join("hd2").to_path_buf())`. Drive the SAME mutation calls the UI buttons would issue: `state.editor.add_category("NewCat")`, `state.editor.add_macro("NewCat", ...)`, `state.editor.edit_macro("NewCat", "m1", MacroUpdates { phrase: Some(Some("new phrase".into())), ..Default::default() })`, `state.editor.move_macro("Cat1", "NewCat", "orig_macro")`, `state.editor.rename_category("Cat2", "Cat2Renamed")`, `state.editor.remove_macro("NewCat", "orig_macro")`, `state.editor.remove_category("Cat1")`. Then `std::fs::create_dir_all(&profile_dir)?` and `state.editor.pack().save_to_dir(&profile_dir)?`. Reload via `Pack::load_from_dir(&profile_dir)?` and call `assert_packs_equal(state.editor.pack(), &reloaded)`.
- `state_parse_key_sequence_drives_form_to_save`: use the public `parse_key_sequence` helper to build a Vec<KeyAction> from a comma-separated string, construct a MacroConfig with those keys, add it via the editor, save to a fresh tempdir, reload, and assert the macro's `keys` survive byte-for-byte (key string + None dwell + None gap).
- `state_save_to_dir_writes_pack_yaml`: simple smoke test asserting that after `state.editor.pack().save_to_dir(tempdir)`, `tempdir.join("pack.yaml")` exists and `serde_yaml_ng::from_str::<Pack>(&fs::read_to_string(...))` round-trips.
After writing the test file, run the full slice acceptance suite and confirm all green: `cargo build` (no features), `cargo build --features gui`, `RUSTFLAGS="-D warnings" cargo clippy --all-targets --features gui -- -D warnings`, `cargo test -- --test-threads=1`. Address any clippy lints surfaced by T01/T02 code that escaped earlier verification. The task is complete only when the full `cargo test` reports the prior 78 tests still passing PLUS the new unit and integration tests, with zero warnings under the strict RUSTFLAGS. Do NOT exercise SwitchProfile in the integration test (no daemon is running in CI); the save-to-disk half is the integration-closure proof for this slice.

## Inputs

- ``src/ui/pack_editor.rs``
- ``src/pack/mod.rs``
- ``src/config.rs``
- ``tests/pack_editor_roundtrip.rs``

## Expected Output

- ``tests/pack_editor_state_roundtrip.rs``

## Verification

cargo build && cargo build --features gui && RUSTFLAGS="-D warnings" cargo clippy --all-targets --features gui -- -D warnings && cargo test -- --test-threads=1

## Observability Impact

The integration test asserts on disk-level state (the save path's externally visible output) rather than internal logs. Failure of any state-roundtrip assertion will name the exact field that diverges via `assert_packs_equal`, providing precise diagnostic localization for future regressions.
