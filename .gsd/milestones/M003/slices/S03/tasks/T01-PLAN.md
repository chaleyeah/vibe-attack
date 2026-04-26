---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T01: Add wizard module src/ui/wizard.rs with panel routing

Create src/ui/wizard.rs (feature-gated to gui via #[cfg(feature = "gui")] or kept as conditional import). Define a WizardPanel trait or simple show_wizard(ui, state) function that dispatches to the correct panel based on first_incomplete_step(). The dispatcher refreshes state after each action by calling probe::run(). Update vibe-attack-config.rs to call show_wizard instead of the debug label loop.

## Inputs

- `src/ui/first_run.rs (SetupStep enum)`
- `src/ui/probe.rs (probe::run)`

## Expected Output

- `src/ui/wizard.rs with show_wizard() dispatcher`
- `vibe-attack-config.rs calls show_wizard instead of debug loop`

## Verification

cargo check --lib and cargo check --bin vibe-attack-config --features gui both pass (no errors from wizard source)
