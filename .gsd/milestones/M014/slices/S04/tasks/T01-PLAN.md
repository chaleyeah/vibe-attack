---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Added SOUND row with Browse/Clear buttons to pack editor macro form, wired to MacroUpdates.sound

Add form_sound: Option<PathBuf> to PackEditorState. Populate from m.sound when selecting a macro. Add SOUND row in form grid with Browse (rfd::FileDialog) and Clear buttons. Pass sound: Some(form_sound) in MacroUpdates on Update Macro. Clear form_sound on Remove Macro.

## Inputs

- `src/ui/pack_editor.rs`
- `src/pack/mod.rs`

## Expected Output

- `src/ui/pack_editor.rs`

## Verification

cargo build --features gui exits 0; cargo test passes

## Observability Impact

None — UI-only change. Sound plays at runtime via existing Dispatcher/SoundPlayer path.
