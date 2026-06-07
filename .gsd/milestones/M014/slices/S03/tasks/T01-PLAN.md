---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Auto-build Whisper initial_prompt from active pack phrases when not explicitly configured

In coordinator.rs, before SttService::new, build effective_initial_prompt: use config.stt.initial_prompt if set, otherwise join all non-empty config.macros[].phrase values as a comma-separated string. Log phrase_count on use. Pass effective_initial_prompt to SttService::new instead of config.stt.initial_prompt.

## Inputs

- `src/pipeline/coordinator.rs`
- `src/config.rs`

## Expected Output

- `src/pipeline/coordinator.rs`

## Verification

cargo test passes; RUST_LOG=info log shows 'STT initial_prompt auto-built' line on daemon start with HD2 pack active

## Observability Impact

tracing::info! logs phrase_count and confirms prompt was auto-built
