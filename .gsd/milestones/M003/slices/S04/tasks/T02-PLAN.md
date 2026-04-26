---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T02: Add CPAL mic level thread with atomic level

Add a MicLevelState struct in src/ui/config_app.rs (or new module). It holds: Arc<AtomicU32> for mic level (f32 as bits), a thread JoinHandle, and a 'no device' flag. spawn_mic_level_thread() opens the default CPAL input device, builds an input stream that computes RMS of each buffer, stores it in the atomic as f32::to_bits(), and keeps running. If the device is not found or stream fails, set a 'no_device' bool and return. The egui update() loop reads the atomic each frame and renders a ProgressBar. Target update rate ~10Hz (buffer size / sample rate).

## Inputs

- `src/audio/mod.rs (CPAL patterns)`
- `Cargo.toml (cpal version)`

## Expected Output

- `MicLevelState struct with Arc<AtomicU32>`
- `spawn_mic_level_thread() returns MicLevelState`
- `ProgressBar renders level 0.0..=1.0 in main config view`
- `'no device' label shown when CPAL fails to open a device`

## Verification

Launch on dev machine: mic level bar moves when speaking; disconnect/no-device: bar stays at 0.0 with 'no device' label, no panic
