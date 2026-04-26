---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T02: Verify rewrite_ptt_key tests run and add edge case coverage

The wizard module already has three rewrite_ptt_key tests gated to #[cfg(feature = 'gui')]. Since cargo test --lib without --features gui skips them, add equivalent non-feature-gated tests for the pure rewrite_ptt_key logic. Extract rewrite_ptt_key to a pub(crate) function in a non-feature-gated module (src/ui/ptt_config.rs) and test it without the gui feature. Alternatively: verify the existing tests cover all branches and document that they require --features gui.

## Inputs

- `src/ui/wizard.rs (rewrite_ptt_key tests)`

## Expected Output

- `Existing three tests documented as requiring --features gui`
- `No duplicate test logic needed`

## Verification

cargo test --lib ui::wizard::tests exits 0 with 0 tests (feature not active) is acceptable; cargo test --lib --features gui would run them but gui build fails on headless kernel; document this limitation
