# S04: Sound feedback UI - per-macro sound file picker

**Goal:** Add per-macro sound file picker to the pack editor, completing MCRO-04 sound feedback UI.
**Demo:** Open pack editor, edit a macro, click Browse, select a .wav file, save. Reload confirms path retained.

## Must-Haves

- Complete the planned slice outcomes.

## Verification

- Run the task and slice verification checks for this slice.

## Tasks

- [x] **T01: Added SOUND row with Browse/Clear buttons to pack editor macro form, wired to MacroUpdates.sound** `est:45 min`
  Add form_sound: Option<PathBuf> to PackEditorState. Populate from m.sound when selecting a macro. Add SOUND row in form grid with Browse (rfd::FileDialog) and Clear buttons. Pass sound: Some(form_sound) in MacroUpdates on Update Macro. Clear form_sound on Remove Macro.
  - Files: `src/ui/pack_editor.rs`
  - Verify: cargo build --features gui exits 0; cargo test passes

## Files Likely Touched

- src/ui/pack_editor.rs
