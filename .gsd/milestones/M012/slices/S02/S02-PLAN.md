# S02: Shared Widget Library

**Goal:** Create src/ui/widgets.rs with reusable widget factory functions using the theme layer
**Demo:** Isolated widget test page (or probe screen) showing each widget in its various states with the new theme.

## Must-Haves

- Not provided.

## Proof Level

- This slice proves: Not provided.

## Integration Closure

Not provided.

## Verification

- Not provided.

## Tasks

- [x] **T01: Implement shared widget library** `est:2h`
  Create widgets.rs with app_header, side_nav, status_footer, led_meter, section_header, primary_button, kbd, banner, status_pill factory functions
  - Files: `src/ui/widgets.rs`, `src/ui/mod.rs`
  - Verify: cargo build --features gui succeeds

## Files Likely Touched

- src/ui/widgets.rs
- src/ui/mod.rs
