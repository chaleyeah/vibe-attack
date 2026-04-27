---
id: T02
parent: S02
milestone: M007
key_files:
  - src/pipeline/coordinator.rs
  - src/pipeline/jsonl.rs
key_decisions:
  - Added comment above SegCfg alias rather than removing the alias — the alias improves local readability at two construction sites in coordinator.rs and is not an accidental duplicate
  - Placed justification comment as a code comment (// ...) directly above the #[allow()] attribute rather than as a doc comment, consistent with the pattern used for the SAFETY: comments in T01
duration: 
verification_result: passed
completed_at: 2026-04-27T11:42:05.322Z
blocker_discovered: false
---

# T02: Added justification comment above SegCfg alias in coordinator.rs and above #[allow(clippy::too_many_arguments)] in jsonl.rs; all allow annotations across src/ now have explanatory comments

**Added justification comment above SegCfg alias in coordinator.rs and above #[allow(clippy::too_many_arguments)] in jsonl.rs; all allow annotations across src/ now have explanatory comments**

## What Happened

Read coordinator.rs to understand the `VadConfig as SegCfg` alias. The alias exists purely for local readability: coordinator.rs constructs segmentation-tuned configs (two distinct configs: one for general pipeline, one tuned for wake-word commands), and `SegCfg` signals that intent at every construction site better than the raw `VadConfig` name. Added a one-line comment immediately above the `use` import explaining this.\n\nRead jsonl.rs to understand the `write_utterance` function's argument list. The function takes 7 arguments because each maps 1:1 to a distinct top-level field in the stable JSONL event contract (D-19..D-22): utterance_id, text, audio_ms, stt_ms, timings (a struct), start_frame_idx, end_frame_idx. No natural grouping would reduce the count without creating an ad-hoc bundle type that carries no semantic value. Added a one-line comment above the `#[allow(clippy::too_many_arguments)]` explaining this.\n\nAudited all `#[allow(` annotations across `src/` with `grep -rn '#\\[allow(' src/` — only one instance exists (jsonl.rs:106), and it now has its justification comment. `cargo check` passed cleanly in 0.40s.

## Verification

grep -rn -B1 '#[allow(' src/ confirmed every allow annotation has a justification comment immediately above it. cargo check finished with 0 errors and 0 warnings in 0.40s.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `grep -rn -B1 '#[allow(' src/ --include='*.rs'` | 0 | ✅ pass — one allow found, justification comment present on line above | 50ms |
| 2 | `cargo check` | 0 | ✅ pass — Finished dev profile with no errors or warnings | 400ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/pipeline/coordinator.rs`
- `src/pipeline/jsonl.rs`
