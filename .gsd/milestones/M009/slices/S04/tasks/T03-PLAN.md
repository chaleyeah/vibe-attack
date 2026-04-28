---
estimated_steps: 14
estimated_files: 1
skills_used: []
---

# T03: Write tests/pack_lifecycle.rs round-trip integration test

Add a new integration test file `tests/pack_lifecycle.rs` that proves the export → import round-trip is byte/field-equivalent for a non-trivial fixture pack. Use `Pack::import_to()` so the test does not mutate `XDG_CONFIG_HOME` (and is therefore not subject to MEM005-style parallel flake).

Why: the milestone success criterion includes `Export → import round-trip produces byte-identical macro content — hermetic test in tests/pack_lifecycle.rs`. This is the slice's primary objective stopping condition for the backend — once it passes, the UI wiring in T04 only adds dialog plumbing on top of a verified contract.

Key constraints:
- File path: `tests/pack_lifecycle.rs` (the milestone roadmap names this exact path; do not pick a different name).
- Build at least 2 tests:
  1. `pack_export_then_import_to_round_trips_macros` — build a fixture Pack with multiple categories and macros that exercise every MacroConfig field (phrase, if_flag, set_flag, sound: None, keys with and without dwell_ms/gap_ms overrides). Save it to a source tempdir. Call `pack.export(source_dir, &zip_path)`. Call `Pack::import_to(&zip_path, &dest_tempdir)`. Reload the imported pack from `dest_tempdir.join(&pack.name)` via `Pack::load_from_dir`. Assert the reloaded pack has identical: name, author, category count, category names in order, macro counts, macro names in order, every MacroConfig field of every macro.
  2. `pack_export_imports_sounds_subdirectory` — write a small dummy file under `source_dir/sounds/test.wav`, export, import_to, then assert `dest_dir/<pack_name>/sounds/test.wav` exists and has the same byte content. This locks in the sounds-bundling behaviour (already in `Pack::export`'s add_dir_to_zip) at the integration level.
- Use `tempfile::tempdir()` for both source and dest dirs — no shared state.
- Do NOT mutate XDG_CONFIG_HOME and do NOT use `#[serial]`. The whole point of using `import_to` is that the test stays hermetic and parallel-safe.
- Use the helper style of `tests/pack_hd2_bundle.rs` for fixture builders (`fn key`, `fn macro_simple`) — copy them inline if needed (these are tiny). Do NOT add a shared test util crate for two functions.
- The fixture Pack name MUST NOT collide with any name used by a test that runs in the same suite under env-var mutation — pick something obviously test-local like `LifecycleRoundTripFixture`.
- Since the test reads ONLY paths created inside its own tempdirs, it does not depend on any tracked fixture file. (Reminder from auto-mode: planned tests must only read from tracked or test-created paths — this test creates its own.)

Failure modes (Q5): zip write failure → io::Error propagated, test panics with clear message; field mismatch → assert_eq! prints expected vs actual.
Negative tests (Q7): not required for this slice — malformed zip handling is already covered by existing pack_hd2_bundle.rs tests.

## Inputs

- ``src/pack/mod.rs``
- ``tests/pack_hd2_bundle.rs``

## Expected Output

- ``tests/pack_lifecycle.rs``

## Verification

cargo test --test pack_lifecycle -- --test-threads=1 — both tests must pass; cargo test -- --test-threads=1 — full suite stays green.
