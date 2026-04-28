---
id: T01
parent: S03
milestone: M009
key_files:
  - src/ui/pack_editor.rs
  - src/ui/mod.rs
  - src/bin/vibe-attack-config.rs
key_decisions:
  - pack_editor module declared without #[cfg(feature = "gui")] in mod.rs (matching wizard, not tray) so parse_key_sequence tests run under default build
  - Removed unused KeyAction import inside mod inner — will be re-added in T02 when form logic consumes it
duration: 
verification_result: passed
completed_at: 2026-04-28T02:45:34.863Z
blocker_discovered: false
---

# T01: Scaffolded src/ui/pack_editor.rs with PackEditorState, parse_key_sequence, and wired pack_editor field into VibeAttackConfigApp with profile-click loading

**Scaffolded src/ui/pack_editor.rs with PackEditorState, parse_key_sequence, and wired pack_editor field into VibeAttackConfigApp with profile-click loading**

## What Happened

Created `src/ui/pack_editor.rs` following the exact `#[cfg(feature = "gui")] mod inner { ... } pub use inner::*;` pattern from `wizard.rs`. Inside `inner`, defined `PackEditorState` owning a `PackEditor`, `profile_dir: PathBuf`, `selected_category/selected_macro: Option<String>`, all five form-state strings, `show_rename_warning: bool`, and `last_error: Option<String>`. Exposed `PackEditorState::new(editor, profile_dir)` and a stub `show_pack_editor` that renders a heading and the pack name, plus inline error display from `last_error`. Implemented `parse_key_sequence` as a pure function outside the `#[cfg(feature = "gui")]` gate so it compiles and tests run under the default (no-features) build. Added four unit tests: `parse_key_sequence_single`, `parse_key_sequence_multiple_with_whitespace`, `parse_key_sequence_empty_errors`, `parse_key_sequence_trailing_comma`. Updated `src/ui/mod.rs` with `pub mod pack_editor;` (no cfg gate, matching `wizard` — both are always compiled at the module level, with eframe usage gated inside). Updated `vibe-attack-config.rs`: imported `show_pack_editor` and `PackEditorState`; added `pack_editor: Option<PackEditorState>` field initialized to `None`; rewrote the profiles-list loop so clicking a profile name calls `Pack::load_from_dir`, wraps in `PackEditor::new`, and stores `Some(PackEditorState::new(...))` in `app.pack_editor`; renders `show_pack_editor` below the profiles list when `pack_editor.is_some()`. One minor adaptation: the `pub mod pack_editor` in `mod.rs` was intentionally left without `#[cfg(feature = \"gui\")]` (unlike `tray`) so that `parse_key_sequence` tests run without the gui feature — this matches how `wizard` is declared.

## Verification

Ran three verification commands: (1) `cargo build` (no features) — clean compile in 2.28s, no leaked egui imports. (2) `cargo build --features gui` — clean compile in 3.99s after removing an unused `KeyAction` import inside the stub. (3) `cargo test --lib ui::pack_editor -- --test-threads=1` — 4/4 tests passed (parse_key_sequence_single, parse_key_sequence_multiple_with_whitespace, parse_key_sequence_empty_errors, parse_key_sequence_trailing_comma).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build` | 0 | ✅ pass | 2280ms |
| 2 | `cargo build --features gui` | 0 | ✅ pass | 3990ms |
| 3 | `cargo test --lib ui::pack_editor -- --test-threads=1` | 0 | ✅ pass — 4 passed, 0 failed | 1370ms |

## Deviations

The task plan suggested the `pub mod pack_editor;` line should match `tray` (which has `#[cfg(feature = \"gui\")]`), but `wizard` (the stated reference pattern) has no such gate and the tests require the module be always compiled. Followed the `wizard` pattern instead. Removed an unused `KeyAction` import from the `inner` block since the T01 stub doesn't consume it — will restore in T02 when form logic arrives.

## Known Issues

none

## Files Created/Modified

- `src/ui/pack_editor.rs`
- `src/ui/mod.rs`
- `src/bin/vibe-attack-config.rs`
