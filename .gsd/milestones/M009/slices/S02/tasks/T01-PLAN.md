---
estimated_steps: 58
estimated_files: 1
skills_used: []
---

# T01: Define PackEditor struct and implement macro-level CRUD (AddMacro, EditMacro, RemoveMacro)

Create the `PackEditor` struct in `src/pack/mod.rs` as a mutable wrapper around `Pack`. Implement the three macro-level CRUD operations and a `MacroUpdates` helper struct for partial edits.

Skills used: tdd (red-green-refactor for each op), test (run cargo test).

## Why
S02 needs a pure-logic state object the UI (S03) and trigger path (S05) can build on. Macro-level CRUD is the densest and most-used surface, so it ships first with full unit-test coverage.

## Public API
```rust
pub struct PackEditor {
    pack: Pack,
}

#[derive(Debug, Clone, Default)]
pub struct MacroUpdates {
    pub phrase: Option<Option<String>>,    // Some(None) clears the field; None leaves unchanged
    pub if_flag: Option<Option<String>>,
    pub set_flag: Option<Option<String>>,
    pub sound: Option<Option<std::path::PathBuf>>,
    pub keys: Option<Vec<KeyAction>>,
}

impl PackEditor {
    pub fn new(pack: Pack) -> Self;
    pub fn pack(&self) -> &Pack;
    pub fn into_pack(self) -> Pack;
    pub fn add_macro(&mut self, category: &str, macro_config: MacroConfig) -> Result<()>;
    pub fn edit_macro(&mut self, category: &str, macro_name: &str, updates: MacroUpdates) -> Result<()>;
    pub fn remove_macro(&mut self, category: &str, macro_name: &str) -> Result<()>;
}
```

## Error semantics (use `anyhow::bail!` with explicit text)
- `add_macro`: error if category not found; error if a macro with the same name already exists in that category
- `edit_macro`: error if category not found; error if macro_name not found in that category
- `remove_macro`: error if category not found; error if macro_name not found

## Constraints
- Do NOT modify the `MacroConfig` or `Pack`/`Category` serde format. `MacroUpdates` is a sibling helper, not a serde type.
- Macro name uniqueness is enforced **within a category**, not globally.
- Preserve `Vec` ordering — pushing onto `category.macros` is correct for AddMacro; EditMacro mutates in place at the existing index.
- The `MacroUpdates::phrase: Option<Option<String>>` pattern lets callers distinguish 'leave unchanged' (`None`) from 'clear the field' (`Some(None)`). Same for if_flag/set_flag/sound. `keys: Option<Vec<KeyAction>>` replaces the entire vec when Some.

## Tests (in `src/pack/mod.rs::tests`)
- `editor_add_macro_success` — fresh PackEditor, add macro to existing category, assert it lands at the end of `categories[i].macros`
- `editor_add_macro_unknown_category_errors` — assert error message names the missing category
- `editor_add_macro_duplicate_name_errors` — pre-populate with a macro, attempt add with same name, assert error
- `editor_edit_macro_replaces_phrase_and_keys` — set `MacroUpdates { phrase: Some(Some("new".into())), keys: Some(vec![key]), ..Default::default() }`, assert old fields preserved (name, sound, etc.) and new ones applied
- `editor_edit_macro_can_clear_optional_field` — `MacroUpdates { phrase: Some(None), ..Default::default() }` clears phrase to None
- `editor_edit_macro_unknown_category_errors`
- `editor_edit_macro_unknown_macro_errors`
- `editor_remove_macro_success` — assert vec length decreases by 1 and named macro absent
- `editor_remove_macro_unknown_category_errors`
- `editor_remove_macro_unknown_macro_errors`

Use a small fixture helper at the top of the test module that builds a `Pack` with two categories (e.g. "Stratagems" and "Ship Modules") and 2 macros each, similar to the helpers in `tests/pack_hd2_bundle.rs` lines 21-50.

## Verify
```bash
RUSTFLAGS="-D warnings" cargo check --all-targets
cargo test --lib pack:: -- --test-threads=1
```
All new tests pass; no warnings.

## Done when
- `PackEditor`, `MacroUpdates`, and the three methods are defined and exported via `pub`
- 10 new unit tests under `src/pack/mod.rs::tests` pass
- `cargo check --all-targets` clean under `-D warnings`
- No changes to `MacroConfig`/`Pack`/`Category` definitions or serde behavior

## Inputs

- ``src/pack/mod.rs``
- ``src/config.rs``
- ``tests/pack_hd2_bundle.rs``

## Expected Output

- ``src/pack/mod.rs``

## Verification

RUSTFLAGS="-D warnings" cargo check --all-targets && cargo test --lib pack:: -- --test-threads=1

## Observability Impact

CRUD methods return `Result<()>` with `anyhow::Error` messages naming the missing/duplicate identifier — these surface to callers (UI in S03) as actionable diagnostics. No async, no shared state, no logs needed at this layer.
