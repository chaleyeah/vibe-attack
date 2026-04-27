---
estimated_steps: 1
estimated_files: 6
skills_used: []
---

# T02: Document src/pipeline/ public items

Add /// doc comments to every undocumented pub item in src/pipeline/ submodules: coordinator.rs (PipelineHandles fields, spawn_pipeline thread topology), dispatcher.rs (Dispatcher methods), matcher.rs (PhraseMatcher, normalize, find_best_match), sound.rs (SoundPlayer + methods), timing.rs (MonoClock + UtteranceTimings methods), jsonl.rs (StageName, StageStatus, JsonlWriter::new/verbosity/write_*). spawn_pipeline doc must summarize the thread topology (audio → VAD thread, VAD → STT thread, STT → dispatch thread, etc.).

## Inputs

- `src/pipeline/ submodules with ~40 undocumented public items per research`

## Expected Output

- `All pub items in src/pipeline/ have explanatory /// docs; spawn_pipeline has a thread-topology summary`

## Verification

Audit script reports 0 undocumented pub items under src/pipeline/; cargo doc renders cleanly; cargo clippy -D warnings passes
