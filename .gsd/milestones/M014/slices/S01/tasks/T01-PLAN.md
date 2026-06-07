---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Replaced broken ⊞ nav icon with 🖥 emoji in widgets.rs

Replace "⊞" (U+229E, not in egui font) with "🖥" in NAV_ITEMS at src/ui/widgets.rs:66. Verify cargo build --features gui succeeds.

## Inputs

- `src/ui/widgets.rs`

## Expected Output

- `src/ui/widgets.rs`

## Verification

cargo build --features gui exits 0

## Observability Impact

None — cosmetic change only.
