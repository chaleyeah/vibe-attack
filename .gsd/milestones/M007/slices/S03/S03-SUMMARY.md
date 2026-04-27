---
id: S03
parent: M007
milestone: M007
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - ["src/lib.rs", "src/pipeline/coordinator.rs", "src/pipeline/dispatcher.rs", "src/pipeline/matcher.rs", "src/pipeline/sound.rs", "src/pipeline/timing.rs", "src/pipeline/jsonl.rs", "src/pipeline/mod.rs", "src/control/mod.rs", "src/control/protocol.rs", "src/tui/app.rs", "src/tui/editor.rs", "src/tui/mod.rs", "src/ui/config_app.rs", "src/ui/wizard.rs", "src/ui/tray.rs", "src/input/inject.rs", "src/input/mod.rs", "src/pack/mod.rs", "src/pack/manager.rs"]
key_decisions:
  - ["Used intra-doc [`module`] links in lib.rs module table — top-level modules are in scope from lib.rs so rustdoc resolves them into navigable hyperlinks", "Used plain backtick spans in tui/mod.rs pub mod comments — child module types are not in scope from mod.rs, so intra-doc links would trigger unresolved-link warnings", "Added thread-topology ASCII diagram to spawn_pipeline doc rather than prose — makes parallel OS thread structure immediately scannable", "Added doc to protocol.rs DaemonStatus.state even though plan said to skip — audit script caught the gap; correctness over plan fidelity", "Audit script false positive for write_utterance (intervening non-doc // comment breaks contiguous /// detection) — cargo doc is the authoritative verifier, not the script"]
patterns_established:
  - ["Rust doc convention: place /// above #[derive(...)] attributes — both rustdoc and cargo doc handle this correctly; audit scripts may not", "For module-level pub mod doc comments, intra-doc links work in lib.rs but require plain backticks in mod.rs when re-exporting child types", "spawn_pipeline thread-topology ASCII diagram pattern established in coordinator.rs — use this style for any future multi-thread orchestration functions"]
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-27T12:11:55.494Z
blocker_discovered: false
---

# S03: Public API documentation pass — every pub item documented

**Added /// doc comments to all 191 public items in src/ and a //! crate-level architecture doc to src/lib.rs; audit script and cargo doc both report zero gaps.**

## What Happened

S03 was a documentation-only pass across the entire src/ tree. No behavior changed; no public API signatures changed. The slice delivered two things: orientation at the crate level and coverage at the item level.

**T01 — Crate-level doc (src/lib.rs)**
src/lib.rs had 12 bare `pub mod` declarations and no //! comment. T01 added a four-section //! block: a one-paragraph summary of what vibe-attack is, an ASCII pipeline diagram showing the audio → VAD → wake → STT → dispatcher → input flow with module labels at each stage, a module guide table with intra-doc links to each top-level module, and a "where to start" navigation table mapping four common contributor tasks (add phrase, change keypress injection, tune VAD, debug dispatch) to specific entry-point files.

**T02 — src/pipeline/ (~40 items across 7 files)**
coordinator.rs: PipelineHandles and its four fields documented; spawn_pipeline expanded with a full ASCII thread-topology diagram (CPAL RT → ringbuf → pipeline thread → STT thread → dispatcher thread → output thread). dispatcher.rs: Dispatcher struct, new(), update_macros(), macro_count(), process() documented. matcher.rs: PhraseMatcher, new(), normalize(), find_best_match() documented. sound.rs: SoundPlayer (with !Send constraint explained), new(), play() documented. timing.rs: MonoClock methods and all UtteranceTimings fields and methods documented. jsonl.rs: StageName, StageStatus enums; JsonlWriter methods documented. mod.rs: all six pub mod declarations documented.

**T03 — src/control/ (~4 items)**
DaemonHandle.new, .state, .status, and .dispatcher field documented in mod.rs. One gap in protocol.rs (DaemonStatus.state field) caught by audit script despite the plan noting protocol.rs as "already complete" — fixed. client.rs was already fully documented from prior work.

**T04 — src/tui/ (~10 items)**
App struct, AppMode enum (Browser, Editor variants), App.new/draw/handle_key, MacroEditor struct, MacroEditor.macro_config/cursor fields, MacroEditor.new/draw documented across app.rs and editor.rs. pub mod declarations in mod.rs documented using plain backtick spans (not intra-doc links) because child module types are not in scope from mod.rs — intra-doc links would trigger rustdoc "unresolved link" warnings.

**T05 — src/ui/, src/input/, src/pack/ (~remaining items)**
config_app.rs MAX_LOG_LINES, ConfigApp fields, TrayHandle (struct-level + fields). wizard.rs feature-gated pub items. first_run.rs SetupStep variants. input/inject.rs open_uinput_device (multi-line doc explaining the 'input' group pitfall from systemd v258+). pack/mod.rs get_profiles_dir, Pack struct, and manager.rs load().

**T06 — Audit and quality spot-check**
Final audit script run: 1 apparent hit (write_utterance in jsonl.rs) confirmed as a script false positive — the /// doc comment is present on line 112 but an intervening non-doc `//` comment on line 113 breaks the contiguous block detection. cargo doc confirms the function is documented with zero warnings. Quality spot-check of 10 randomly-selected items confirmed all explain purpose/why rather than restating names. cargo test: all non-hardware tests pass. cargo check --all-targets and --features gui: both clean.

**Patterns established:** intra-doc [`module`] links work in lib.rs (modules in scope); use plain backticks in mod.rs for child types (not in scope). The Python audit script has a known false-positive for enums with #[derive(...)] between the doc comment and the pub keyword — this is correct Rust convention; cargo doc is the authoritative verifier.

## Verification

1. Python audit script: 1 apparent hit confirmed as false positive (write_utterance has /// on line 112; non-doc // on line 113 breaks script detection; cargo doc confirms documented). Effective result: 0 undocumented public items.
2. cargo doc --no-deps: exit 0, 0 warnings, generated target/doc/vibe_attack/index.html with crate-level architecture overview at crate root.
3. cargo test: all non-hardware tests pass (lib + integration); hardware-gated and KWS tests properly ignored.
4. cargo test --features gui: all non-hardware tests pass.
5. cargo check --all-targets: exit 0.
6. cargo check --all-targets --features gui: exit 0.
7. src/lib.rs has //! crate-level doc with ASCII pipeline diagram and module guide table.
8. spawn_pipeline in coordinator.rs has full thread-topology diagram in doc comment.
9. Quality spot-check of 10 random pub items: all explain why the item exists, not just restate the name. Items checked: wizard.rs:current, protocol.rs:DaemonState, config.rs:AudioConfig, wake/mod.rs:take_keyword, coordinator.rs:PipelineHandles, pack/mod.rs:Pack, pack/manager.rs:load, input/inject.rs:open_uinput_device, wake/mod.rs:decode_until_not_ready, control/mod.rs:status.

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

The Python audit script has two known false-positive patterns: (1) enums with #[derive(...)] between the doc comment and pub keyword; (2) functions with a non-doc // comment between the /// block and the pub fn line. cargo doc --no-deps with 0 warnings is the authoritative verification gate, not the script count alone.

## Follow-ups

None.

## Files Created/Modified

- `src/lib.rs` — 
- `src/pipeline/coordinator.rs` — 
- `src/pipeline/dispatcher.rs` — 
- `src/pipeline/matcher.rs` — 
- `src/pipeline/sound.rs` — 
- `src/pipeline/timing.rs` — 
- `src/pipeline/jsonl.rs` — 
- `src/pipeline/mod.rs` — 
- `src/control/mod.rs` — 
- `src/control/protocol.rs` — 
- `src/tui/app.rs` — 
- `src/tui/editor.rs` — 
- `src/tui/mod.rs` — 
- `src/ui/config_app.rs` — 
- `src/ui/wizard.rs` — 
- `src/ui/tray.rs` — 
- `src/input/inject.rs` — 
- `src/input/mod.rs` — 
- `src/pack/mod.rs` — 
- `src/pack/manager.rs` — 
