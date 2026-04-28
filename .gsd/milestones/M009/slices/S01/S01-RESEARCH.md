# S01: Full HD2 stratagem pack + coverage test — Research

**Date:** 2026-04-27

## Summary

The current bundled HD2 pack (`profiles/hd2/pack.yaml`) contains only 12 macros across 3 categories (Patriotic Administration Center, Orbital Cannons, Hangar). Helldivers 2 has 80+ stratagems organized into 6+ ship-module categories: Patriotic Administration Center, Orbital Cannons, Hangar, Bridge, Engineering Bay, and Robotics Workshop (with premium stratagem variants). The slice involves expanding pack.yaml to cover all categories with verified key sequences, then creating a new hermetic test file (`tests/pack_hd2_coverage.rs`) that verifies the pack contains all categories and meets minimum macro counts per category.

The task is low-risk because it is purely additive: existing YAML structure and serde types are compatible with larger packs. The test patterns are already proven in `tests/pack_hd2_bundle.rs` (9 test cases covering serialization, flatten, import/export, ProfileManager integration). The main effort is research and accurate transcription of HD2 stratagem names and key sequences, followed by a straightforward test that asserts category presence and counts.

## Recommendation

**Approach: Expand pack.yaml in phases, then write unit-style coverage test.**

1. Research and transcribe all 80+ HD2 stratagem names and key sequences from the wiki/training knowledge (W=UP, A=LEFT, S=DOWN, D=RIGHT).
2. Organize into 6 categories matching ship-module unlock order:
   - Patriotic Administration Center (support weapons, defensive stratagems) — expand from 3 to ~13
   - Orbital Cannons (orbital strikes) — expand from 3 to ~15
   - Hangar (eagle airstrikes) — expand from 3 to ~12
   - Bridge (vehicle/mech stratagems) — new category with ~8
   - Engineering Bay (defensive structures, ammo supply) — new category with ~10
   - Robotics Workshop (autonomous units) — new category with ~8
3. Verify each macro's key sequence matches the wiki and test locally by playing one or two stratagems.
4. Create `tests/pack_hd2_coverage.rs` with a single test that:
   - Loads `profiles/hd2/pack.yaml` using `Pack::load_from_dir()`
   - Asserts all 6 categories are present by name
   - Asserts minimum macro counts per category (e.g., 10+ for Administration, 10+ for Orbital, etc.)
   - Asserts total macro count is 75+
   - Optionally asserts no category is empty
5. Run `cargo test --test pack_hd2_coverage` to verify; update pack.yaml until test passes.

This approach isolates content curation from test logic, keeps the test simple and maintainable, and leaves room for future stratagem additions without changing the test structure.

## Implementation Landscape

### Key Files

- **`profiles/hd2/pack.yaml`** — Currently 12 macros in 3 categories. Needs expansion to 80+ stratagems across 6 ship-module categories. Format is stable YAML with flat key list per macro (no nested objects). Author and name fields will remain unchanged.

- **`src/pack/mod.rs`** — `Pack { name, author, categories: Vec<Category> }` struct; `Category { name, macros: Vec<MacroConfig> }`; `MacroConfig { name, phrase, keys: Vec<KeyAction>, if_flag, set_flag, sound }`. All types are already serde-compatible. No changes needed; this file defines the YAML schema that pack.yaml must follow.

- **`tests/pack_hd2_bundle.rs`** — Existing integration test with 22 test cases covering round-trip serialization, flatten, export/import (ZIP), and ProfileManager. Patterns to follow:
  - Use `tempfile::tempdir()` for hermetic temp dirs.
  - Use `#[serial]` for tests that mutate env vars (like `XDG_CONFIG_HOME`).
  - Use helper functions like `key()`, `macro_simple()`, `hd2_pack()` for test fixtures.
  - Assert on both structure (names, counts) and content (phrases, key sequences, flags).

- **`tests/pack_hd2_coverage.rs`** — New file to create. Single integration test that loads the bundled pack from disk and asserts coverage completeness. This test is lower-effort than pack_hd2_bundle because it:
  - Does not mock or build packs in-memory; loads the real `profiles/hd2/pack.yaml`.
  - Does not test export/import; focuses only on structure and content.
  - Does not use ProfileManager; calls `Pack::load_from_dir()` directly.

### Build Order

1. **Research HD2 stratagems and construct stratagem list** (2–3 hours)
   - Source: official Helldivers 2 wiki or in-game stratagem selection menu.
   - Map each stratagem to its key sequence (W/A/S/D notation).
   - Group by ship module category.
   - Verify key sequences by cross-referencing gameplay footage or player guides.

2. **Expand pack.yaml** (1–2 hours)
   - Add macros to each existing category.
   - Create Bridge, Engineering Bay, Robotics Workshop categories.
   - Validate YAML syntax (`cargo check` should pass).

3. **Create tests/pack_hd2_coverage.rs** (30–45 minutes)
   - Write a single integration test that loads `profiles/hd2/pack.yaml`.
   - Assert all 6 categories are present.
   - Assert minimum macro counts.
   - Assert no category is empty and pack is non-empty.

4. **Verify** (15–30 minutes)
   - Run `cargo test --test pack_hd2_coverage`.
   - Run `cargo test --test pack_hd2_bundle` to ensure no regressions.
   - Run `cargo test` to check all tests pass.
   - Run `cargo clippy -D warnings --all-targets` (or RUSTFLAGS substitute).

### Verification Approach

**Local verification:**
```bash
# Load pack.yaml and verify it is valid YAML
cargo test --test pack_hd2_coverage -- --nocapture

# Ensure no regressions in existing pack tests
cargo test --test pack_hd2_bundle -- --nocapture

# Full test suite
cargo test

# Clippy check (local substitute: RUSTFLAGS="-D warnings" cargo check --all-targets)
RUSTFLAGS="-D warnings" cargo check --all-targets
```

**Manual gameplay verification (optional):**
- Launch Helldivers 2 game session.
- Test 1–2 newly added stratagems by voice to confirm key sequences are correct.
- This is not strictly required; wiki sources are authoritative.

**Hermetic test coverage:**
- The test loads pack.yaml from the profiles directory.
- Does not depend on external network, model files, or audio.
- Does not mutate the profiles directory (read-only operation).
- Runs serially if needed; likely no env var mutations required.

## Constraints

- **YAML structure is immutable:** The Pack/Category/MacroConfig types are defined in `src/pack/mod.rs` and serialized using serde-yaml-ng. The structure must match the existing YAML shape (nested categories with name and macros array).
- **File paths must follow XDG convention:** Profiles are loaded from `$XDG_CONFIG_HOME/vibe-attack/profiles/hd2/` (MEM006 pattern). The test may need to set `XDG_CONFIG_HOME` to a temp dir if it uses `Pack::load_from_dir()` and expects a specific path, but since the test calls `Pack::load_from_dir(Path)` directly, no env var is needed.
- **Key names must match uinput key constants:** Keys in the YAML must be valid uinput key names (e.g., `KEY_UP`, `KEY_DOWN`, `KEY_LEFT`, `KEY_RIGHT`, `KEY_W`, `KEY_S`, `KEY_A`, `KEY_D`). The existing pack uses `KEY_UP`, `KEY_DOWN`, `KEY_LEFT`, `KEY_RIGHT`. Check if `KEY_W`, etc., are valid; if not, stick to arrow keys.
- **Test must be hermetic:** No reliance on external files, network, or user config. The test loads pack.yaml as a static fixture.
- **No schema changes:** The export/import ZIP format is backward-compatible (MEM036); no changes to the .hdpack schema.

## Common Pitfalls

- **Inconsistent key naming:** The pack currently uses `KEY_UP`, `KEY_DOWN`, `KEY_LEFT`, `KEY_RIGHT`. Helldivers 2 uses W, A, S, D for directional inputs. Ensure the key names in the YAML match valid uinput event codes (likely `KEY_W`, `KEY_A`, `KEY_S`, `KEY_D` or arrow keys). Verify in uinput headers or evdev crate documentation.

- **Category name mismatches:** The test will assert category names (e.g., "Patriotic Administration Center"). The names must be exact and case-sensitive. Do not assume category names without verifying against the game or official wiki.

- **Missing or incomplete stratagem coverage:** Do not leave any category empty. The test will fail if a category with zero macros exists. If a category has fewer stratagems than others, still populate it with all available stratagems in that module.

- **Key sequence transcription errors:** Each stratagem has a specific 3–5 key sequence. Transcribing incorrectly will cause the in-game test to fail (if performed). Use a reliable source (wiki, official guide, or gameplay video) and double-check sequences against multiple sources.

- **Phrase field mismatches:** The `phrase` field is used for voice command matching. Ensure phrases are descriptive, unique within the pack, and match the in-game stratagem name (or a common shorthand). Do not leave phrase fields empty or duplicate them across macros.

- **Test assuming specific counts:** The test should use minimum counts (e.g., `>= 75` macros total, `>= 10` per major category) rather than exact counts. This allows for future additions without failing the test. Avoid hardcoding exact numbers like "88 macros" because the game may add stratagems over time.

- **Forgetting to run full test suite:** After expanding pack.yaml and adding the new test, always run `cargo test` (not just the new test) to ensure no regressions in pack_hd2_bundle.rs or other integration tests. ProfileManager tests may reference the pack structure.

- **Environmental issues with #[serial]:** If the test uses `#[serial]` and env vars, ensure they are cleaned up even if the test panics. Use `std::env::remove_var()` in a finally-like block or avoid mutating env vars in favor of passing paths directly to `Pack::load_from_dir()`.

