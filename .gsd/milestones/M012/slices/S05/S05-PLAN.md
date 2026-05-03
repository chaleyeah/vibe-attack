# S05: PackEditor Rewrite

**Goal:** Rewrite pack_editor.rs with 3-column layout: category list, macro list with search, detail form
**Demo:** Pack editor opens existing pack; drag-reorder categories and macros; search filters macro list; edits persist on save.

## Must-Haves

- Not provided.

## Proof Level

- This slice proves: Not provided.

## Integration Closure

Not provided.

## Verification

- Not provided.

## Tasks

- [x] **T01: Rewrite PackEditor with 3-column layout and amber accent rows** `est:3h`
  3-column layout: categories panel | macro list with search filter | detail/edit form. Amber highlight on selected rows.
  - Files: `src/ui/pack_editor.rs`
  - Verify: cargo build --features gui succeeds

## Files Likely Touched

- src/ui/pack_editor.rs
