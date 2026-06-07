# S02: VAD overhaul - robust onset and tuned defaults

**Goal:** Overhaul VAD onset algorithm from consecutive-frame counter to sliding-window majority vote, tune defaults, add per-frame DEBUG tracing.
**Demo:** cargo test in vad module passes including new onset tests. Config UI sliders visible in ADVANCED pane.

## Must-Haves

- Complete the planned slice outcomes.

## Verification

- Run the task and slice verification checks for this slice.

## Tasks

- [x] **T01: Replaced consecutive-frame onset counter with N-of-M sliding window majority vote in VadSegmenter** `est:90 min`
  Replace start_run_frames consecutive counter with onset_window: VecDeque<bool> majority vote (N of M frames). Add onset_window_frames to VadConfig. Update VadSegmenter::new, push_scored_frame, finish_current_utterance_with_tail, force_flush.
  - Files: `src/vad/mod.rs`
  - Verify: cargo test vad passes including new onset_tolerates_single_dip test

- [x] **T02: Updated VAD config defaults and wired onset_window_ms through config.rs and coordinator.rs** `est:30 min`
  Update VadConfig defaults: start 0.60->0.50, stop 0.35->0.30, end_silence 20->25 frames. Add onset_window_ms to config.rs VadConfig and coordinator.rs seg_cfg construction. Update config.example.yaml and config.yaml.
  - Files: `src/config.rs`, `src/pipeline/coordinator.rs`, `config.example.yaml`, `config.yaml`
  - Verify: cargo test passes

## Files Likely Touched

- src/vad/mod.rs
- src/config.rs
- src/pipeline/coordinator.rs
- config.example.yaml
- config.yaml
