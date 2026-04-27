---
id: S02
parent: M007
milestone: M007
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - (none)
key_decisions:
  - ["Used two distinct // SAFETY: comments per unsafe impl rather than a shared block — each trait impl has a subtly different invariant to justify", "Kept SegCfg alias (rather than removing it) with a comment — alias improves readability at two segmentation-config construction sites", "Placed all justification comments as plain // code comments (not /// doc comments) — lint rationale and safety notes are internal, not for rustdoc", "Each get_socket_path comment names its specific XDG call and references the counterpart by file path for navigability", "cargo check used as clippy substitute — clippy not available in this system apt Rust environment"]
patterns_established:
  - ["// SAFETY: comment directly above each unsafe impl/fn, one per block", "// justification comment directly above each #[allow(...)] annotation", "Cross-reference comments on intentionally duplicated private functions naming the counterpart file"]
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-27T11:47:53.447Z
blocker_discovered: false
---

# S02: Internal consistency — safety comments, alias notes, lint annotations

**All unsafe impls, type aliases, lint allows, and dual-function patterns in src/ are now annotated with justifying comments; one duplicate doc comment collapsed.**

## What Happened

S02 made the codebase internally consistent across five targeted locations, touching six source files with comment-only changes. No logic or public API was altered.

**T01 — SAFETY comments on Dispatcher unsafe impls (src/pipeline/dispatcher.rs)**
The two `unsafe impl Send` and `unsafe impl Sync` blocks for `Dispatcher` had only a generic comment that did not follow Rust's `// SAFETY:` convention and did not explain the soundness invariant. Two distinct `// SAFETY:` comments were added — one per impl — each explaining that rodio's `OutputStream` (held by `SoundPlayer`) is `!Send`, but `Dispatcher` only ever accesses `sound_player` from its single owning thread, so no cross-thread access can occur. The split-comment approach was chosen so each impl is self-contained when read in isolation.

**T02 — SegCfg alias annotation and #[allow] justification (src/pipeline/coordinator.rs, src/pipeline/jsonl.rs)**
The `VadConfig as SegCfg` alias in coordinator.rs exists for local readability: coordinator.rs constructs segmentation-tuned VAD configs at two call sites, and `SegCfg` communicates that intent more clearly than the raw type name. A comment was added above the `use` import explaining this. In jsonl.rs, the `write_utterance` function carries `#[allow(clippy::too_many_arguments)]` because each of its 7 arguments maps 1:1 to a distinct top-level field in the stable JSONL event schema — no meaningful grouping reduces them without creating a hollow bundle type. A one-line justification comment was added above the allow. A full audit of all `#[allow(` annotations across `src/` confirmed only this one instance exists.

**T03 — Cross-referencing comments on dual get_socket_path (src/control/mod.rs, src/control/client.rs)**
Both control files contain a private `get_socket_path` function with intentionally different semantics: the server-side (mod.rs) uses `xdg.place_runtime_file` to create the XDG runtime directory on daemon startup; the client-side (client.rs) uses `xdg.find_runtime_file` for a read-only probe that errors if the daemon isn't running. A two-line comment was added above each function naming the specific XDG call, explaining the implication (create vs. read-only), and pointing to the counterpart file by path so a reader following either function can immediately find the other.

**T04 — Duplicate doc comment collapsed (src/config.rs)**
Lines 258–261 had four consecutive `///` lines — two nearly identical pairs describing `default_config_path`. The first pair omitted the "creating the config directory if needed" detail; the second pair added it but repeated the path description verbatim. All four were collapsed into two accurate lines: a summary retaining the directory-creation detail (which is correct — `place_config_file` does create the directory) and a path-resolution line clarifying the XDG expansion.

**T05 — Full verification pass**
`cargo check --all-targets` exits 0. `cargo test` exits 0 (lib tests and doc tests pass; two hardware-gated integration tests are ignored as expected). All `grep -B1 'unsafe impl'` output shows `// SAFETY:` lines immediately above each impl. All `grep -rn '#[allow(' src/'` output shows a justification comment on the line above the one instance. `grep -B3 'fn get_socket_path'` in both control files shows the cross-reference comment. `grep -n -A2 'fn default_config_path'` in config.rs shows a single coherent two-line doc block.

Note: `cargo clippy` is not available in this system apt Rust environment (no rustup). `cargo check --all-targets` was used as the nearest available substitute. The structural changes (comments only) are verifiable by grep regardless of clippy availability.

## Verification

1. `grep -B1 'unsafe impl' src/pipeline/dispatcher.rs` → both unsafe impl lines immediately preceded by `// SAFETY:` (exit 0)
2. `grep -rn -B1 '#[allow(' src/ --include='*.rs'` → one instance in jsonl.rs:106, justification comment on line 105 (exit 0)
3. `grep -B3 'fn get_socket_path' src/control/mod.rs src/control/client.rs` → cross-reference comment visible above each function (exit 0)
4. `grep -n -A2 'fn default_config_path' src/config.rs` → single two-line /// doc block, no duplicate (exit 0)
5. `cargo check --all-targets` → Finished dev profile, 0 errors, 0 warnings (exit 0)
6. `cargo test` → all tests pass; 2 hardware-gated tests ignored as expected (exit 0)

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

"`cargo clippy -D warnings` could not be run — clippy is not installed in this system apt Rust environment. All structural changes are verifiable by grep. A rustup-managed toolchain would allow full clippy verification as the slice plan specified."

## Follow-ups

None.

## Files Created/Modified

- `src/pipeline/dispatcher.rs` — 
- `src/pipeline/coordinator.rs` — 
- `src/pipeline/jsonl.rs` — 
- `src/control/mod.rs` — 
- `src/control/client.rs` — 
- `src/config.rs` — 
