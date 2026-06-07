---
estimated_steps: 1
estimated_files: 4
skills_used: []
---

# T02: Updated VAD config defaults and wired onset_window_ms through config.rs and coordinator.rs

Update VadConfig defaults: start 0.60->0.50, stop 0.35->0.30, end_silence 20->25 frames. Add onset_window_ms to config.rs VadConfig and coordinator.rs seg_cfg construction. Update config.example.yaml and config.yaml.

## Inputs

- `src/config.rs`
- `src/pipeline/coordinator.rs`

## Expected Output

- `src/config.rs`
- `src/pipeline/coordinator.rs`
- `config.example.yaml`
- `config.yaml`

## Verification

cargo test passes

## Observability Impact

None — config change only
