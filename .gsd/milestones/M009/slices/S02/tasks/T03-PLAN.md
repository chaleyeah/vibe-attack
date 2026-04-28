---
estimated_steps: 27
estimated_files: 1
skills_used: []
---

# T03: Add hermetic round-trip integration test (edit → save → reload → structural equality)

Create `tests/pack_editor_roundtrip.rs` proving that PackEditor edits survive a full save/load cycle without corruption. This is the slice's integration-level proof that the pure-logic state machine composes correctly with the existing `Pack::save_to_dir` / `Pack::load_from_dir` boundary.

Skills used: test, verify-before-complete.

## Why
Unit tests in T01/T02 prove each CRUD op in isolation; this test proves that the *outcome* of a sequence of edits round-trips through serde_yaml_ng correctly — catching any field-ordering or serde-format regression that unit tests would miss.

## Test plan
Create one integration test file `tests/pack_editor_roundtrip.rs` with these tests:

1. `roundtrip_after_full_crud_sequence` — Build a starter Pack via local fixture helper (2 categories, 4 macros, mix of phrase/if_flag/set_flag/keys with timing). Wrap in PackEditor. Run a representative sequence: add_category("NewCat"), add_macro("NewCat", ...), edit_macro on existing, move_macro between categories, rename_category, remove_macro, remove_category (after emptying). Save to tempdir, reload, re-serialize to YAML, and assert that the reloaded Pack deep-equals the editor's in-memory pack (use field-by-field assertions: name, author, categories.len(), then per-category name + macro vec equality).

2. `roundtrip_yaml_text_stable_within_run` — Edit a Pack, save to one tempdir, save the same in-memory Pack to a second tempdir, read both `pack.yaml` files as bytes, and assert byte-for-byte equality. This guards against non-determinism in serde_yaml_ng output within a single process run.

3. `roundtrip_preserves_optional_fields` — Build a Pack containing macros with `phrase = None`, `if_flag = Some("flag")`, `set_flag = None`, `sound = None`, varied keys (some with dwell/gap overrides, some without). Run a no-op edit (add+remove same macro), save, reload, assert all Option fields preserved exactly.

## Constraints
- File must live at `tests/pack_editor_roundtrip.rs` (tracked in git, integration-test conventions)
- Use `tempfile::tempdir()` for the save/load destination; never write to the real `XDG_CONFIG_HOME` (no env mutation needed since we pass `dir.path()` directly to `save_to_dir`)
- No `#[serial]` attribute needed — these tests do not mutate process env (per MEM053 pattern)
- No fixtures from `.gitignore`d paths — define the starter Pack inline using small helper fns at the top of the test file (mirror the `key()`, `macro_simple()` helpers from `tests/pack_hd2_bundle.rs` lines 21-50)
- For deep equality, write a private `assert_packs_equal(left: &Pack, right: &Pack)` helper that walks categories and macros explicitly. Do NOT add a `PartialEq` derive to `Pack`/`Category`/`MacroConfig` — that would touch the canonical config types which are out of scope for this slice.

## Verify
```bash
RUSTFLAGS="-D warnings" cargo check --all-targets
cargo test --test pack_editor_roundtrip -- --test-threads=1
cargo test -- --test-threads=1
```
All three new tests pass; full suite still passes (no regression in pack_hd2_bundle, pack_hd2_coverage, etc.).

## Done when
- `tests/pack_editor_roundtrip.rs` exists, contains the 3 named tests, and they pass
- Full `cargo test -- --test-threads=1` is green
- `cargo check --all-targets` clean under `-D warnings`
- Per slice success criteria: 'round-trip: edit → save → reload → byte equivalence' is demonstrated by `roundtrip_yaml_text_stable_within_run` (byte equality of save outputs) plus `roundtrip_after_full_crud_sequence` (structural equality after save→load)

## Inputs

- ``src/pack/mod.rs``
- ``src/config.rs``
- ``tests/pack_hd2_bundle.rs``

## Expected Output

- ``tests/pack_editor_roundtrip.rs``

## Verification

RUSTFLAGS="-D warnings" cargo check --all-targets && cargo test --test pack_editor_roundtrip -- --test-threads=1 && cargo test -- --test-threads=1

## Observability Impact

Test failures will print the diverging field path (via the explicit `assert_packs_equal` helper's per-field assertions) so a future agent can localize a serde regression to the exact field that drifted. Byte-equality test pinpoints serializer non-determinism if it ever surfaces.
