# S03: STT accuracy - initial prompt from active pack

**Goal:** Auto-populate Whisper initial_prompt from the active pack's phrase list so Whisper biases toward known vocabulary, reducing hallucinations on short stratagem names.
**Demo:** cargo test passes. Log shows initial_prompt set line on daemon start.

## Must-Haves

- Complete the planned slice outcomes.

## Verification

- Run the task and slice verification checks for this slice.

## Tasks

- [x] **T01: Auto-build Whisper initial_prompt from active pack phrases when not explicitly configured** `est:30 min`
  In coordinator.rs, before SttService::new, build effective_initial_prompt: use config.stt.initial_prompt if set, otherwise join all non-empty config.macros[].phrase values as a comma-separated string. Log phrase_count on use. Pass effective_initial_prompt to SttService::new instead of config.stt.initial_prompt.
  - Files: `src/pipeline/coordinator.rs`
  - Verify: cargo test passes; RUST_LOG=info log shows 'STT initial_prompt auto-built' line on daemon start with HD2 pack active

## Files Likely Touched

- src/pipeline/coordinator.rs
