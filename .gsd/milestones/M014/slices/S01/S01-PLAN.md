# S01: Fix broken DEVICES nav icon

**Goal:** Fix the broken DEVICES nav icon — replace U+229E "⊞" (not in egui's bundled font) with 🖥 emoji which renders correctly via NotoEmoji.
**Demo:** Launch vibe-attack-config and observe the DEVICES nav item renders a recognisable icon.

## Must-Haves

- Complete the planned slice outcomes.

## Verification

- Run the task and slice verification checks for this slice.

## Tasks

- [x] **T01: Replaced broken ⊞ nav icon with 🖥 emoji in widgets.rs** `est:15 min`
  Replace "⊞" (U+229E, not in egui font) with "🖥" in NAV_ITEMS at src/ui/widgets.rs:66. Verify cargo build --features gui succeeds.
  - Files: `src/ui/widgets.rs`
  - Verify: cargo build --features gui exits 0

## Files Likely Touched

- src/ui/widgets.rs
