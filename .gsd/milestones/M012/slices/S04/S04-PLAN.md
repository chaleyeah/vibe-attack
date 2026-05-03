# S04: Wizard Rewrite

**Goal:** Rewrite wizard.rs with step indicator strip, PTT dashed drop-zone, and styled steps
**Demo:** Wizard runs to completion from a clean state; PTT capture drop-zone captures a key; mic-test LED meter animates.

## Must-Haves

- Not provided.

## Proof Level

- This slice proves: Not provided.

## Integration Closure

Not provided.

## Verification

- Not provided.

## Tasks

- [x] **T01: Rewrite wizard.rs with themed steps and PTT drop-zone** `est:2h`
  Implement step indicator strip at top, PTT key capture as dashed drop-zone, mic-test step with LED meter animation
  - Files: `src/ui/wizard.rs`
  - Verify: cargo build --features gui succeeds

## Files Likely Touched

- src/ui/wizard.rs
