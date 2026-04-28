---
id: T04
parent: S04
milestone: M009
key_files:
  - src/ui/pack_editor.rs
  - src/bin/vibe-attack-config.rs
key_decisions:
  - Used Option<String> field (imported_pack_name) on PackEditorState rather than a PackEditorOutcome enum — show_pack_editor returns () and a drainable field matches the established egui communication idiom in this codebase (matches setup_just_completed pattern)
  - rfd dialog calls are synchronous on the egui frame thread — no thread spawning, no async rfd API — matching all egui-rfd integrations as confirmed by T01 research
duration: 
verification_result: passed
completed_at: 2026-04-28T03:07:39.308Z
blocker_discovered: false
---

# T04: Wired Import Pack / Export Pack buttons into show_pack_editor toolbar; successful import refreshes the profiles list via imported_pack_name drain in show_main_config

**Wired Import Pack / Export Pack buttons into show_pack_editor toolbar; successful import refreshes the profiles list via imported_pack_name drain in show_main_config**

## What Happened

Added `imported_pack_name: Option<String>` to `PackEditorState` (simpler than a return-value enum given the existing `()` signature of `show_pack_editor`). Inside the `#[cfg(feature = "gui")] mod inner` block, added `use rfd::FileDialog` and `use crate::pack::{get_profiles_dir, Pack, PackEditor}`.

Two buttons were inserted into the existing category toolbar `ui.horizontal` block, after the Rename Category cluster:

**Import Pack**: opens `FileDialog::new().add_filter("Pack", &["hdpack"]).pick_file()`. On `Some(path)`, resolves `get_profiles_dir()`, calls `Pack::import_to(&path, &profiles_dir)`. On success, logs `tracing::info!(zip_path, pack_name, macro_count, "Import Pack: succeeded")`, swaps `state.editor` and `state.profile_dir` to point at the imported pack, clears selection state, clears `last_error`, and sets `state.imported_pack_name = Some(pack_name)`. On any error, logs `tracing::warn!(reason, "Import Pack: failed")` and sets `last_error`. `None` from the dialog (cancel / portal unavailable) is a no-op.

**Export Pack**: opens `FileDialog::new().add_filter("Pack", &["hdpack"]).set_file_name("<pack_name>.hdpack").save_file()`. On `Some(dest_path)`, calls `state.editor.pack().export(&state.profile_dir, &dest_path)`. On success, logs `tracing::info!(dest_path, pack_name, macro_count, "Export Pack: succeeded")` and clears `last_error`. On error, logs warn and sets `last_error`.

In `src/bin/vibe-attack-config.rs`, after `show_pack_editor(ui, editor_state)`, added a drain: `if editor_state.imported_pack_name.take().is_some() { app.config.profiles = load_profiles(); }` — this re-populates the profiles list immediately after a successful import without any additional work since the editor was already swapped inside the import handler.

Decision: used `Option<String>` field on `PackEditorState` rather than a `PackEditorOutcome` enum. Rationale: `show_pack_editor` returns `()` and egui closures already have complex borrow patterns; a field that the caller drains each frame is the established egui communication idiom (matches `setup_just_completed` in the same file) and avoids changing the function signature.

## Verification

1. `cargo build --features gui` — compiled clean, zero warnings.
2. `cargo build` (no features) — compiled clean, zero warnings; proves rfd is gated correctly.
3. `grep -c 'FileDialog::new' src/ui/pack_editor.rs` — returned 2 (one Import, one Export).
4. `cargo test -- --test-threads=1` — 88 unit tests passed, all integration test suites passed (including pack_lifecycle round-trip tests from T03), 0 failures.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features gui` | 0 | ✅ pass | 7120ms |
| 2 | `cargo build` | 0 | ✅ pass | 1720ms |
| 3 | `grep -c 'FileDialog::new' src/ui/pack_editor.rs` | 0 | ✅ pass — 2 matches | 10ms |
| 4 | `cargo test -- --test-threads=1` | 0 | ✅ pass — 88 unit + all integration suites green | 4200ms |

## Deviations

none

## Known Issues

Import Pack silently removes an existing profile directory if the same pack name already exists (inherits Pack::import_to collision contract). A confirmation prompt is a known nice-to-have, deferred to S06 UAT per the task plan.

## Files Created/Modified

- `src/ui/pack_editor.rs`
- `src/bin/vibe-attack-config.rs`
