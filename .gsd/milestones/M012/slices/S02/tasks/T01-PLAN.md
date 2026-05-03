---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Implement shared widget library

Create widgets.rs with app_header, side_nav, status_footer, led_meter, section_header, primary_button, kbd, banner, status_pill factory functions

## Inputs

- `src/ui/theme.rs`

## Expected Output

- `All widget functions compile`
- `Callable from config_app, wizard, pack_editor`

## Verification

cargo build --features gui succeeds
