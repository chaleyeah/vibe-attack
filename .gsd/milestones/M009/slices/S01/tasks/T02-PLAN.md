---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T02: Add tests/pack_hd2_coverage.rs hermetic coverage test

Create a new integration test file tests/pack_hd2_coverage.rs that loads profiles/hd2/pack.yaml directly via Pack::load_from_dir() and asserts the bundled HD2 pack covers all current ship-module categories. The test must NOT mutate XDG_CONFIG_HOME, NOT touch the network, NOT depend on model files or audio, and NOT use ProfileManager — it loads the static fixture from the repo path (use `Path::new("profiles/hd2")` relative to the cargo workspace root; integration tests run with CWD = workspace root). Write a single #[test] fn `hd2_pack_covers_all_ship_modules()` that: (1) loads the pack, (2) asserts pack.name == "Helldivers 2", (3) asserts the 6 expected category names are all present (use a HashSet difference for a clear assertion message), (4) asserts per-category minimum macro counts (Administration >=10, Orbital >=12, Hangar >=10, Bridge >=5, Engineering Bay >=8, Robotics Workshop >=6), (5) asserts total flatten().len() >= 75, (6) asserts no category has zero macros (loop and assert each category.macros.len() > 0 with a name in the message). Add a second helper #[test] `hd2_pack_phrases_are_unique()` that flattens and asserts all phrase strings are unique (HashSet collect + len equality). Use the existing patterns in tests/pack_hd2_bundle.rs as a style reference (vibe_attack::pack::Pack import, anyhow::Result return). Do NOT add #[serial] — no env var mutation. Add /// doc comments to both test fns per D002 (every pub item documented; tests are not pub but the module-level //! comment should describe purpose).

## Inputs

- ``profiles/hd2/pack.yaml``
- ``tests/pack_hd2_bundle.rs``
- ``src/pack/mod.rs``

## Expected Output

- ``tests/pack_hd2_coverage.rs``

## Verification

cargo test --test pack_hd2_coverage -- --nocapture && cargo test && RUSTFLAGS="-D warnings" cargo check --all-targets
