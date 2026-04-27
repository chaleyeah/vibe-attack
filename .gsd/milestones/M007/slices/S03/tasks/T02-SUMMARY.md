---
id: T02
parent: S03
milestone: M007
key_files:
  - src/pipeline/coordinator.rs
  - src/pipeline/dispatcher.rs
  - src/pipeline/matcher.rs
  - src/pipeline/sound.rs
  - src/pipeline/timing.rs
  - src/pipeline/jsonl.rs
  - src/pipeline/mod.rs
key_decisions:
  - Added thread-topology ASCII diagram to spawn_pipeline doc rather than a prose description — makes the parallel OS thread structure immediately scannable and matches the //! comment style already in coordinator.rs
  - Added /// doc to pub mod declarations in mod.rs in addition to the //! comments already in each submodule, so both surfaces (module page and parent re-export) carry orientation text
duration: 
verification_result: passed
completed_at: 2026-04-27T11:54:09.705Z
blocker_discovered: false
---

# T02: Added /// doc comments to all undocumented pub items in src/pipeline/ (coordinator, dispatcher, matcher, sound, timing, jsonl, mod) and expanded spawn_pipeline with a full thread-topology diagram

**Added /// doc comments to all undocumented pub items in src/pipeline/ (coordinator, dispatcher, matcher, sound, timing, jsonl, mod) and expanded spawn_pipeline with a full thread-topology diagram**

## What Happened

The pipeline submodules had ~40 undocumented public items across 7 files. I ran a Python audit script first to enumerate exactly which items needed docs, then worked file-by-file:

**coordinator.rs** — added doc to `PipelineHandles` struct and its four fields (`pipeline`, `output`, `stt`, `dispatcher`), then expanded the existing `spawn_pipeline` doc to include a full ASCII thread-topology diagram showing: CPAL RT callback → ringbuf → pipeline thread → VAD/wake → STT thread → dispatcher thread → output thread (stdout JSONL). The diagram also calls out bounded channel back-pressure and the drop-oldest VAD→STT semantics.

**dispatcher.rs** — added doc to `Dispatcher` struct (explaining it receives STT transcripts and fires macros, and that live registry updates work without restart), `new()` (threshold and timing defaults), `update_macros()`, `macro_count()`, and `process()` (explaining the side effects: sound playback + MacroCmd send).

**matcher.rs** — added doc to `PhraseMatcher` struct (Levenshtein-similarity scoring), `new()`, `normalize()` (what the normalization does: lowercase, strip punct, collapse whitespace), and `find_best_match()` (iterator of (macro_id, phrase) pairs, returns best-scoring candidate).

**sound.rs** — added doc to `SoundPlayer` struct (explaining it wraps rodio, is `!Send`, and why), `new()`, and `play()` (fire-and-forget via detached Sink).

**timing.rs** — added doc to `MonoClock::start_now()`, `elapsed()`, `elapsed_ms()`; added field docs to `UtteranceTimings::created_wall_time_ms`, `vad_done_ms`, `stt_done_ms`, `output_done_ms`; added docs to `UtteranceTimings::new()`, `elapsed_ms()`, `mark_vad_done()`, `mark_stt_done()`, `mark_output_done()`.

**jsonl.rs** — added doc to `StageName` and `StageStatus` enums; added docs to `JsonlWriter::new()` (noting MonoClock anchor), `verbosity()`, `write_stage()`, `write_dispatch()`, `write_no_match()`, and `write_event()`.

**mod.rs** — added `///` doc comment on each of the six `pub mod` declarations.

Audit re-run after all edits: 0 undocumented public items in src/pipeline/. `cargo doc --no-deps` completed with zero warnings. `cargo test` shows 40 lib tests passing, 0 failures.

## Verification

Python audit script reports 0 undocumented pub items in src/pipeline/. cargo doc --no-deps: exit 0, no warnings. cargo check: exit 0. cargo test: 40 lib tests passed, 0 failed (pre-existing pack failure appears to have been fixed upstream; current run shows clean pass).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `python3 audit_pipeline.py (inline)` | 0 | ✅ pass — 0 undocumented pub items in src/pipeline/ | 50ms |
| 2 | `cargo doc --no-deps` | 0 | ✅ pass — no warnings, generated target/doc/vibe_attack/index.html | 730ms |
| 3 | `cargo check` | 0 | ✅ pass | 450ms |
| 4 | `cargo test (lib + integration)` | 0 | ✅ pass — 40 lib tests, 0 failures | 3100ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/pipeline/coordinator.rs`
- `src/pipeline/dispatcher.rs`
- `src/pipeline/matcher.rs`
- `src/pipeline/sound.rs`
- `src/pipeline/timing.rs`
- `src/pipeline/jsonl.rs`
- `src/pipeline/mod.rs`
