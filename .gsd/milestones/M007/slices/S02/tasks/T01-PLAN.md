---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Add SAFETY comments to unsafe impl Send/Sync on Dispatcher

In src/pipeline/dispatcher.rs, add a `// SAFETY:` comment immediately above each `unsafe impl Send for Dispatcher {}` and `unsafe impl Sync for Dispatcher {}`. The comment must explain that rodio's OutputStream (held by SoundPlayer) is not Send, but Dispatcher only ever invokes SoundPlayer from its single owning thread, making the manual Send/Sync impls sound. Run cargo clippy -D warnings to verify.

## Inputs

- `src/pipeline/dispatcher.rs current unsafe impl Send/Sync without safety comments`

## Expected Output

- `dispatcher.rs with // SAFETY: comments above both unsafe impls explaining the single-owning-thread invariant`

## Verification

grep -B1 'unsafe impl' src/pipeline/dispatcher.rs shows a // SAFETY: line immediately preceding each unsafe impl; cargo clippy --all-targets -- -D warnings clean
