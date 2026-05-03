---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Rewrite PackEditor with 3-column layout and amber accent rows

3-column layout: categories panel | macro list with search filter | detail/edit form. Amber highlight on selected rows.

## Inputs

- `src/ui/theme.rs`
- `src/ui/widgets.rs`

## Expected Output

- `3-column layout renders`
- `Search filters macro list`
- `Edits persist on save`

## Verification

cargo build --features gui succeeds
