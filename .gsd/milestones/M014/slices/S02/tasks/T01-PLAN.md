---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Replaced consecutive-frame onset counter with N-of-M sliding window majority vote in VadSegmenter

Replace start_run_frames consecutive counter with onset_window: VecDeque<bool> majority vote (N of M frames). Add onset_window_frames to VadConfig. Update VadSegmenter::new, push_scored_frame, finish_current_utterance_with_tail, force_flush.

## Inputs

- `src/vad/mod.rs`

## Expected Output

- `src/vad/mod.rs`

## Verification

cargo test vad passes including new onset_tolerates_single_dip test

## Observability Impact

tracing::debug! per-frame VAD score added to push_frame_silero
