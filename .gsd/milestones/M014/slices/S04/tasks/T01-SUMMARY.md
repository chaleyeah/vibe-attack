---
id: T01
parent: S04
milestone: M014
key_files:
  - src/ui/pack_editor.rs
key_decisions:
  - (none)
duration: 
verification_result: passed
completed_at: 2026-06-07T21:14:07.153Z
blocker_discovered: false
---

# T01: Added SOUND row with Browse/Clear buttons to pack editor macro form, wired to MacroUpdates.sound

**Added SOUND row with Browse/Clear buttons to pack editor macro form, wired to MacroUpdates.sound**

## What Happened

Added form_sound: Option<PathBuf> to PackEditorState. When a macro is selected, form_sound is populated from m.sound. The SOUND form row shows the filename (or 'none'), a Browse button that opens rfd::FileDialog filtered to audio types, and a Clear button when a path is set. On Update Macro, sound: Some(form_sound.clone()) is passed through MacroUpdates so the path is persisted to pack.yaml. form_sound is cleared on Remove Macro. The SoundPlayer/Dispatcher already plays the file at runtime — this completes MCRO-04.

## Verification

cargo build --features gui exits 0. cargo test: all tests pass with no regressions.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features gui && cargo test` | 0 | pass | 25000ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/ui/pack_editor.rs`
