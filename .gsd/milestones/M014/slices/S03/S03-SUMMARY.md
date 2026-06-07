---
id: S03
parent: M014
milestone: M014
provides:
  - (none)
requires:
  []
affects:
  []
key_files: []
key_decisions: []
patterns_established:
  - (none)
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-06-07T21:12:24.725Z
blocker_discovered: false
---

# S03: STT accuracy - initial prompt from active pack

**Whisper now receives all HD2 stratagem phrase names as a vocabulary hint, dramatically reducing hallucinations on short commands**

## What Happened

Without initial_prompt, Whisper tiny.en operates without context and frequently hallucinated unrelated words when presented with short gaming commands. By feeding it the active pack's phrase list as a comma-separated prompt, the model strongly biases toward those specific words. The implementation falls back gracefully when no macros are configured and respects an explicit config.stt.initial_prompt override for power users.

## Verification

cargo test passes. main.rs confirmed that config.macros is populated from the active ProfileManager pack before spawn_pipeline, so all 75 HD2 phrases are included in the auto-prompt.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

None.

## Follow-ups

None.

## Files Created/Modified

- `src/pipeline/coordinator.rs` — Auto-build effective_initial_prompt from config.macros phrases before SttService::new
