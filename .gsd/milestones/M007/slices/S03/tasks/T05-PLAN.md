---
estimated_steps: 1
estimated_files: 5
skills_used: []
---

# T05: Document src/ui/ and src/input/ and src/pack/ public items

Add /// doc comments to remaining undocumented pub items in src/ui/ (config_app.rs MAX_LOG_LINES + ConfigApp fields, first_run.rs SetupStep variants, wizard.rs feature-gated inner types if reasonable to document), src/input/ (KeyStep::from_config, MacroCmd enum), src/pack/ (get_profiles_dir).

## Inputs

- `src/ui/, src/input/, src/pack/ with remaining undocumented public items`

## Expected Output

- `All pub items in these modules have /// docs`

## Verification

Audit script reports 0 undocumented pub items under src/ui/, src/input/, src/pack/; cargo doc renders cleanly
