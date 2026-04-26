---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Add FirstRunState unit tests and load_profiles tempdir test

Add #[cfg(test)] block to src/ui/first_run.rs testing: from_checks all-false returns empty steps_remaining; from_checks all-true returns empty steps_remaining and is_setup_complete=true; from_checks partial returns expected step ordering. Add #[cfg(test)] block to src/ui/config_app.rs testing load_profiles: returns empty vec when dir absent; returns sorted stems when yaml files present; ignores non-yaml files. Use tempdir + XDG hermetic isolation with #[serial].

## Inputs

- `src/ui/first_run.rs`
- `src/ui/config_app.rs`

## Expected Output

- `5+ tests in first_run.rs`
- `3+ tests in config_app.rs covering load_profiles`
- `All pass under cargo test --lib`

## Verification

cargo test --lib ui::first_run and cargo test --lib ui::config_app exit 0
