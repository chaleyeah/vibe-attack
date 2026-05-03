---
estimated_steps: 19
estimated_files: 1
skills_used: []
---

# T01: Converted outer /// doc comments in pack_editor.rs to //! inner form to fix clippy::empty_line_after_doc_comments lint

Convert the outer `///` doc comments at the top of `src/ui/pack_editor.rs` (lines 1-5) into `//!` inner module doc comments. The current form — outer `///` block followed by a blank line and then `#[cfg(feature = "gui")]` — triggers `clippy::empty_line_after_doc_comments` which is fatal under `-D warnings` in CI. Clippy's own suggestion is to make these inner doc comments since they document the module itself, not the re-export below them. After editing, verify locally with `cargo build --all-targets` (default features) and `cargo build --all-targets --features gui` since `cargo clippy` is not installed in this dev environment (per MEM038/MEM073). The CI clippy job will be the authoritative check once the test tag is pushed in T02.

Steps:
1. Read `src/ui/pack_editor.rs` lines 1-10 to confirm current state.
2. Replace lines 1-5 (the `///` block plus the blank line) with `//!`-prefixed inner doc comments. The `//!` form does NOT require a blank line before subsequent items, so the blank line on line 6 may either remain (it's harmless before `#[cfg(...)]`) or be kept — preserve current spacing before `#[cfg(feature = "gui")]` if you want a minimal diff.
3. Run `cargo build --all-targets` and confirm exit 0 with no warnings printed.
4. Run `cargo build --all-targets --features gui` and confirm exit 0 with no warnings printed.
5. `git add src/ui/pack_editor.rs` and create a single commit with message `fix(ui): convert pack_editor module doc comments to inner form`.

Must-haves:
- Lines 1-5 of `src/ui/pack_editor.rs` become `//!` inner doc comments preserving the original prose verbatim.
- Both `cargo build` variants (default and `--features gui`) exit 0 with no warnings.
- Exactly one new commit on the current branch (`main`) with the fix; no other files modified.

Failure modes:
- Editor accidentally drops the inner-doc prefix on a line, breaking the module's documentation rendering — verify all 3 prose lines retain `//!` prefix.
- `//!` placed inside `mod inner { ... }` instead of at file top — must remain at file-scope to document the outer module.
- Build picks up unrelated cached warnings — run `cargo clean` only if a stale warning is suspected; otherwise trust the fresh build output.

Negative tests:
- Re-introducing a blank line between the `//!` block and `#[cfg(feature = "gui")]` should still pass (clippy's lint is specific to outer `///` followed by blank). Do not need to test this, but understand the lint scope before editing.

Load profile: N/A — single-file local edit.

Observability impact: none — the edit is to comment syntax only; no runtime behavior changes.

## Inputs

- ``src/ui/pack_editor.rs``
- ``.github/workflows/ci.yml``

## Expected Output

- ``src/ui/pack_editor.rs``

## Verification

cargo build --all-targets 2>&1 | grep -E '^(warning|error)' | wc -l | grep -qx 0 && cargo build --all-targets --features gui 2>&1 | grep -E '^(warning|error)' | wc -l | grep -qx 0 && grep -q '^//! Pack editor panel' src/ui/pack_editor.rs

## Observability Impact

none — comment-only edit; runtime behavior unchanged.
