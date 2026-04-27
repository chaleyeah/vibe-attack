# S03: Public API documentation pass — every pub item documented — UAT

**Milestone:** M007
**Written:** 2026-04-27T12:11:55.494Z

# S03 UAT — Public API Documentation Pass

## Preconditions
- Working copy of hd-linux-voice with S03 changes applied
- Rust toolchain available (`cargo`, `rustdoc`)
- Python 3 available for audit script

## Test Cases

### TC-01: Crate-level doc comment present and renders
1. Open `src/lib.rs` in an editor
2. Confirm the file begins with `//!` doc lines (not bare `pub mod`)
3. Confirm the //! block contains: a one-paragraph summary mentioning "vibe-attack", an ASCII diagram with `audio →` pipeline notation, a module guide table, and a "Where to start" section
4. Run: `cargo doc --no-deps`
5. Open `target/doc/vibe_attack/index.html` in a browser (or inspect the generated file)
**Expected:** cargo doc exits 0 with no warnings; the crate root page shows the architecture overview with all four sections visible

### TC-02: Audit script reports 0 undocumented public items
1. Run the pub-item Python audit script targeting `src/`
2. Confirm it finds 0 items (or only known false positives caused by #[derive] between doc and pub keyword — these should be verified against cargo doc)
**Expected:** Script output shows 0 effective undocumented public items; cargo doc confirms all items are documented

### TC-03: spawn_pipeline has thread-topology diagram
1. Open `src/pipeline/coordinator.rs`
2. Find the `pub fn spawn_pipeline` function
3. Read the preceding /// doc block
**Expected:** Doc block contains an ASCII diagram showing the thread chain: CPAL RT callback → ringbuf → pipeline thread → STT thread → dispatcher thread → output thread, with channel back-pressure noted

### TC-04: Quality spot-check — docs explain why, not just what
1. Pick any 5 of these items and read their /// comment:
   - `src/pipeline/sound.rs` — `SoundPlayer` (should mention !Send constraint and rodio)
   - `src/pipeline/matcher.rs` — `normalize` (should explain what normalization does: lowercase, strip punct, collapse whitespace)
   - `src/control/mod.rs` — `status` (should mention DaemonStatus snapshot and Status queries)
   - `src/input/inject.rs` — `open_uinput_device` (should mention 'input' group, systemd v258+ pitfall)
   - `src/pipeline/dispatcher.rs` — `process` (should mention side effects: sound playback + MacroCmd send)
**Expected:** Each doc comment explains the item's purpose or a non-obvious constraint, not just a restatement of the function name

### TC-05: cargo doc renders without warnings
1. Run: `cargo doc --no-deps 2>&1`
2. Check stderr for any "warning" lines
**Expected:** Zero warnings; exit code 0

### TC-06: cargo doc with gui feature renders without warnings
1. Run: `cargo doc --no-deps --features gui 2>&1`
2. Check stderr for any "warning" lines
**Expected:** Zero warnings; exit code 0

### TC-07: cargo test passes (default features)
1. Run: `cargo test`
**Expected:** All non-hardware-gated tests pass; hardware tests (uinput, KWS) are ignored with explanatory messages; exit code 0

### TC-08: cargo test passes (gui feature)
1. Run: `cargo test --features gui`
**Expected:** All non-hardware-gated tests pass; exit code 0

### TC-09: cargo check clean on both feature sets
1. Run: `cargo check --all-targets`
2. Run: `cargo check --all-targets --features gui`
**Expected:** Both exit 0 with no errors

### TC-10: Module guide table has intra-doc links
1. Run: `cargo doc --no-deps`
2. Open `target/doc/vibe_attack/index.html`
3. Hover over module names in the module guide table
**Expected:** Module names are hyperlinks to their respective module documentation pages

## Edge Cases
- **Script false positive on #[derive] enums:** If the audit script flags an enum like `AppMode`, verify manually that the `///` comment appears above the `#[derive(...)]` attribute — this is correct Rust convention and cargo doc handles it correctly.
- **pub(crate) items:** Confirm the audit script does NOT flag `pub(crate)` items as undocumented — those are internal and not part of the public API surface that S03 targets.
- **write_utterance false positive:** The audit script may flag `jsonl.rs:write_utterance` because a non-doc `//` comment appears between the `///` and the `pub fn`. Verify line 112 of jsonl.rs has the `///` doc and that cargo doc renders it correctly.
