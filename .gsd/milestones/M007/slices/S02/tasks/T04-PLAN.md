---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T04: Collapse duplicate doc comment on default_config_path

In src/config.rs around lines 258–260, two consecutive /// lines both say 'Return the XDG config file path'. Collapse into a single accurate doc comment. Verify against current line numbers (file may have shifted).

## Inputs

- `src/config.rs default_config_path with duplicate /// docs`

## Expected Output

- `src/config.rs default_config_path with one clean doc comment`

## Verification

grep -A1 'fn default_config_path' src/config.rs shows a single coherent /// doc block; cargo check passes
