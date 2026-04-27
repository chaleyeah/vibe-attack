---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T02: Annotate the SegCfg alias and #[allow] annotations

In src/pipeline/coordinator.rs, add a comment explaining why VadConfig is aliased as SegCfg (likely historical naming or to avoid conflict with another type — read git blame or the surrounding code to confirm; if the alias has no real reason, remove it instead). In src/pipeline/jsonl.rs, add a one-line comment above `#[allow(clippy::too_many_arguments)]` justifying why the function legitimately needs that many arguments. Audit `grep -rn '#\[allow(' src/` for any other unjustified allows and add justification comments to each.

## Inputs

- `coordinator.rs with `use crate::vad::{VadConfig as SegCfg, ...}` and jsonl.rs with #[allow(clippy::too_many_arguments)]`

## Expected Output

- `coordinator.rs with explanatory comment on SegCfg alias (or alias removed); jsonl.rs and any other allow annotations have justification comments`

## Verification

grep -B1 '#\[allow(' src/ shows a justifying comment above each allow; cargo clippy -D warnings clean
