# S05: Integration smoke tests

**Goal:** Hermetic tests for probe logic, wizard state transitions, config copy action, rewrite_ptt_key, and load_profiles. All pass in CI without a display server.
**Demo:** cargo test covers probe module, FirstRunState transitions, and binary smoke; all pass in CI without a display server

## Must-Haves

- probe unit tests pass with XDG isolation; FirstRunState steps_remaining and is_setup_complete covered; rewrite_ptt_key three-branch coverage; load_profiles tested with tempdir; all pass via cargo test --lib

## Proof Level

- This slice proves: cargo test --lib exits 0 with all new tests passing

## Integration Closure

All tests run under cargo test --lib without display server or external devices

## Verification

- None — test-only slice

## Tasks

- [x] **T01: Add FirstRunState unit tests and load_profiles tempdir test** `est:35m`
  Add #[cfg(test)] block to src/ui/first_run.rs testing: from_checks all-false returns empty steps_remaining; from_checks all-true returns empty steps_remaining and is_setup_complete=true; from_checks partial returns expected step ordering. Add #[cfg(test)] block to src/ui/config_app.rs testing load_profiles: returns empty vec when dir absent; returns sorted stems when yaml files present; ignores non-yaml files. Use tempdir + XDG hermetic isolation with #[serial].
  - Files: `src/ui/first_run.rs`, `src/ui/config_app.rs`
  - Verify: cargo test --lib ui::first_run and cargo test --lib ui::config_app exit 0

- [x] **T02: Verify rewrite_ptt_key tests run and add edge case coverage** `est:20m`
  The wizard module already has three rewrite_ptt_key tests gated to #[cfg(feature = 'gui')]. Since cargo test --lib without --features gui skips them, add equivalent non-feature-gated tests for the pure rewrite_ptt_key logic. Extract rewrite_ptt_key to a pub(crate) function in a non-feature-gated module (src/ui/ptt_config.rs) and test it without the gui feature. Alternatively: verify the existing tests cover all branches and document that they require --features gui.
  - Files: `src/ui/wizard.rs`
  - Verify: cargo test --lib ui::wizard::tests exits 0 with 0 tests (feature not active) is acceptable; cargo test --lib --features gui would run them but gui build fails on headless kernel; document this limitation

## Files Likely Touched

- src/ui/first_run.rs
- src/ui/config_app.rs
- src/ui/wizard.rs
