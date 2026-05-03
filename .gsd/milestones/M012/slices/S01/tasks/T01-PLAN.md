---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Implement theme.rs with palette and apply_theme()

Create src/ui/theme.rs defining all color tokens, font registration, and apply_theme(ctx) that sets egui Visuals/Style

## Inputs

- `egui Visuals/Style API`

## Expected Output

- `src/ui/theme.rs compiles`
- `apply_theme() callable from all UI modules`

## Verification

cargo build --features gui succeeds
