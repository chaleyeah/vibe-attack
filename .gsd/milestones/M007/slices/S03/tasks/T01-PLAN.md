---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Add crate-level //! doc comment to src/lib.rs

Add a //! doc comment at the top of src/lib.rs that orients a new reader. Include: (a) one-paragraph summary of what vibe-attack is, (b) a labeled ASCII or markdown diagram of the audio → VAD → wake → STT → pipeline → input flow showing module boundaries, (c) a brief description of each top-level module (audio, vad, wake, stt, pipeline, input, control, config, error, pack, ui, tui), (d) where to start reading for common tasks (adding a phrase, changing input behavior, debugging dispatch). Verify cargo doc renders without warnings.

## Inputs

- `src/lib.rs (12 lines, no //! doc currently)`

## Expected Output

- `src/lib.rs with a substantive //! crate-level doc comment`

## Verification

cargo doc --no-deps succeeds; the rendered docs include the architecture overview at the crate root; reading src/lib.rs gives a new engineer enough orientation to find any module
