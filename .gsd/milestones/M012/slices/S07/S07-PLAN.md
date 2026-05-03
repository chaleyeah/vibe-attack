# S07: Integration + Screenshot Capture

**Goal:** Integration: clean release build, full test suite pass, screenshot gallery in ui/screenshots/
**Demo:** Clean build, tests green, screenshot gallery showing all 12 screens vs reference renders.

## Must-Haves

- Not provided.

## Proof Level

- This slice proves: Not provided.

## Integration Closure

Not provided.

## Verification

- Not provided.

## Tasks

- [x] **T01: Release build, test suite, screenshot gallery** `est:1h`
  cargo build --release --features gui, run full test suite with --test-threads=1, capture screenshots to ui/screenshots/
  - Files: `ui/screenshots/`
  - Verify: Release build succeeds, 0 test failures, screenshots present

## Files Likely Touched

- ui/screenshots/
