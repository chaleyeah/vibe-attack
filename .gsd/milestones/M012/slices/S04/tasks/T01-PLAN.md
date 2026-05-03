---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Rewrite wizard.rs with themed steps and PTT drop-zone

Implement step indicator strip at top, PTT key capture as dashed drop-zone, mic-test step with LED meter animation

## Inputs

- `src/ui/theme.rs`
- `src/ui/widgets.rs`

## Expected Output

- `Wizard runs all 6 steps`
- `PTT drop-zone captures key press`
- `LED meter animates on mic-test step`

## Verification

cargo build --features gui succeeds
