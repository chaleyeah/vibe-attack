---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T04: Document src/tui/ public items

Add /// doc comments to every undocumented pub item in src/tui/: app.rs (App, AppMode, App::new/draw/handle_key), editor.rs (MacroEditor + methods).

## Inputs

- `src/tui/ with ~10 undocumented public items per research`

## Expected Output

- `All pub items in src/tui/ have /// docs`

## Verification

Audit script reports 0 undocumented pub items under src/tui/; cargo doc renders cleanly
