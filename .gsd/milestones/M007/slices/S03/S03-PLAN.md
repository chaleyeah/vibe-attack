# S03: Public API documentation pass — every pub item documented

**Goal:** Add /// doc comments to every undocumented public item in src/ (~100 items), with focus on the pipeline, control, tui, and ui modules. Add a //! crate-level doc comment to src/lib.rs orienting a new reader to the audio→VAD→wake→STT→pipeline→input architecture and the control/config/error/pack/ui module boundaries.
**Demo:** cargo test passes; cargo clippy -D warnings clean; the Python undocumented-pub-item audit script from M007-RESEARCH.md reports 0 undocumented public items in src/; src/lib.rs has a //! crate-level doc comment describing the audio → VAD → wake → STT → pipeline → input architecture; spot-check of 10 random pub items shows doc comments explain why the item exists, not just restate the name

## Must-Haves

- The Python audit script from M007-RESEARCH.md reports 0 undocumented public items in src/; src/lib.rs has a //! crate-level doc comment with a labeled pipeline diagram; src/pipeline/coordinator.rs::spawn_pipeline has a doc comment summarizing thread topology; quality spot-check of 10 random pub items confirms doc comments explain why the item exists, not just restate the name; cargo test passes; cargo clippy --all-targets -- -D warnings clean on default and gui features.

## Proof Level

- This slice proves: static — verified by audit script and manual quality spot-check. No runtime behavior changes.

## Integration Closure

No integration boundaries touched.

## Verification

- None.

## Tasks

- [x] **T01: Add crate-level //! doc comment to src/lib.rs** `est:45m`
  Add a //! doc comment at the top of src/lib.rs that orients a new reader. Include: (a) one-paragraph summary of what vibe-attack is, (b) a labeled ASCII or markdown diagram of the audio → VAD → wake → STT → pipeline → input flow showing module boundaries, (c) a brief description of each top-level module (audio, vad, wake, stt, pipeline, input, control, config, error, pack, ui, tui), (d) where to start reading for common tasks (adding a phrase, changing input behavior, debugging dispatch). Verify cargo doc renders without warnings.
  - Files: `src/lib.rs`
  - Verify: cargo doc --no-deps succeeds; the rendered docs include the architecture overview at the crate root; reading src/lib.rs gives a new engineer enough orientation to find any module

- [x] **T02: Document src/pipeline/ public items** `est:2h`
  Add /// doc comments to every undocumented pub item in src/pipeline/ submodules: coordinator.rs (PipelineHandles fields, spawn_pipeline thread topology), dispatcher.rs (Dispatcher methods), matcher.rs (PhraseMatcher, normalize, find_best_match), sound.rs (SoundPlayer + methods), timing.rs (MonoClock + UtteranceTimings methods), jsonl.rs (StageName, StageStatus, JsonlWriter::new/verbosity/write_*). spawn_pipeline doc must summarize the thread topology (audio → VAD thread, VAD → STT thread, STT → dispatch thread, etc.).
  - Files: `src/pipeline/coordinator.rs`, `src/pipeline/dispatcher.rs`, `src/pipeline/matcher.rs`, `src/pipeline/sound.rs`, `src/pipeline/timing.rs`, `src/pipeline/jsonl.rs`
  - Verify: Audit script reports 0 undocumented pub items under src/pipeline/; cargo doc renders cleanly; cargo clippy -D warnings passes

- [x] **T03: Document src/control/ public items** `est:30m`
  Add /// doc comments to every undocumented pub item in src/control/: mod.rs (DaemonHandle::new/state/status), client.rs (send_command/query_status/is_daemon_running). protocol.rs is already well-documented per research — verify and skip if so.
  - Files: `src/control/mod.rs`, `src/control/client.rs`
  - Verify: Audit script reports 0 undocumented pub items under src/control/; cargo doc renders cleanly

- [x] **T04: Document src/tui/ public items** `est:30m`
  Add /// doc comments to every undocumented pub item in src/tui/: app.rs (App, AppMode, App::new/draw/handle_key), editor.rs (MacroEditor + methods).
  - Files: `src/tui/app.rs`, `src/tui/editor.rs`
  - Verify: Audit script reports 0 undocumented pub items under src/tui/; cargo doc renders cleanly

- [ ] **T05: Document src/ui/ and src/input/ and src/pack/ public items** `est:1h`
  Add /// doc comments to remaining undocumented pub items in src/ui/ (config_app.rs MAX_LOG_LINES + ConfigApp fields, first_run.rs SetupStep variants, wizard.rs feature-gated inner types if reasonable to document), src/input/ (KeyStep::from_config, MacroCmd enum), src/pack/ (get_profiles_dir).
  - Files: `src/ui/config_app.rs`, `src/ui/first_run.rs`, `src/ui/wizard.rs`, `src/input/inject.rs`, `src/pack/mod.rs`
  - Verify: Audit script reports 0 undocumented pub items under src/ui/, src/input/, src/pack/; cargo doc renders cleanly

- [ ] **T06: Run audit script and quality spot-check** `est:30m`
  Run the M007-RESEARCH.md Python audit script against src/ — must report 0 undocumented public items. Then randomly select 10 newly-documented pub items, read each /// comment, and confirm it explains the item's purpose (why) not just restates the name (what). If any are superficial, revise. Run cargo test, cargo test --features gui, cargo clippy --all-targets -- -D warnings, cargo clippy --all-targets --features gui -- -D warnings, cargo doc --no-deps. All must pass.
  - Verify: Audit script output is '0 undocumented public items'; spot-check log (10 items reviewed) is captured in slice summary; all cargo invocations exit 0

## Files Likely Touched

- src/lib.rs
- src/pipeline/coordinator.rs
- src/pipeline/dispatcher.rs
- src/pipeline/matcher.rs
- src/pipeline/sound.rs
- src/pipeline/timing.rs
- src/pipeline/jsonl.rs
- src/control/mod.rs
- src/control/client.rs
- src/tui/app.rs
- src/tui/editor.rs
- src/ui/config_app.rs
- src/ui/first_run.rs
- src/ui/wizard.rs
- src/input/inject.rs
- src/pack/mod.rs
