# S04: Main config app wired up

**Goal:** Populate ConfigApp from real data: profiles from XDG profiles directory, live mic level from a CPAL background thread, log lines via an mpsc channel. If no audio device exists, show 0.0 level gracefully without panicking.
**Demo:** After wizard completes, the main config view shows real profile names, a live mic level bar that responds to audio input, and log lines appearing as the daemon runs

## Must-Haves

- Profiles panel lists all .yaml files from XDG profiles dir; mic level updates ~10Hz when device present; mic level shows 0.0 with 'no device' label when absent; log feed scrolls to bottom on new lines; no stub values remain in production code paths

## Proof Level

- This slice proves: Manual launch on dev machine with profiles dir populated; mic level bar responds to voice input; disconnect audio device and confirm no crash

## Integration Closure

ConfigApp populated by a single loader called after wizard completion; CPAL thread runs continuously; log channel fed from tracing subscriber or direct send

## Verification

- CPAL thread logs device name on start and error reason on failure; profile load logs count of profiles found

## Tasks

- [x] **T01: Load real profiles from XDG profiles dir** `est:25m`
  Add a load_profiles() function in src/ui/config_app.rs or a new src/ui/loader.rs. It should: resolve the XDG profiles directory via xdg::BaseDirectories::with_prefix('vibe-attack').get_config_home() and join 'profiles', read all *.yaml files, extract the profile name (stem of the filename), return Vec<String>. Log the count. In vibe-attack-config.rs, call load_profiles() after wizard completion and set ConfigApp.profiles. Re-load on 'Refresh' button click.
  - Files: `src/ui/config_app.rs`, `src/bin/vibe-attack-config.rs`
  - Verify: Place a test.yaml in ~/.config/vibe-attack/profiles/ and launch; profile list shows 'test'

- [x] **T02: Add CPAL mic level thread with atomic level** `est:45m`
  Add a MicLevelState struct in src/ui/config_app.rs (or new module). It holds: Arc<AtomicU32> for mic level (f32 as bits), a thread JoinHandle, and a 'no device' flag. spawn_mic_level_thread() opens the default CPAL input device, builds an input stream that computes RMS of each buffer, stores it in the atomic as f32::to_bits(), and keeps running. If the device is not found or stream fails, set a 'no_device' bool and return. The egui update() loop reads the atomic each frame and renders a ProgressBar. Target update rate ~10Hz (buffer size / sample rate).
  - Files: `src/ui/config_app.rs`, `src/bin/vibe-attack-config.rs`
  - Verify: Launch on dev machine: mic level bar moves when speaking; disconnect/no-device: bar stays at 0.0 with 'no device' label, no panic

- [x] **T03: Add log channel feed to main config view** `est:40m`
  Add a log mpsc channel to VibeAttackConfigApp: sender side is passed to vibe-attack-config.rs as a simple fn add_log(msg). In the update() loop, drain all pending messages from the receiver into ConfigApp.log_lines (respecting MAX_LOG_LINES cap). Add a tracing subscriber layer that writes to the channel (using a custom Layer from tracing_subscriber) so daemon log events appear in the UI. ScrollArea auto-scrolls to bottom when new lines arrive (track last_line_count, scroll when it changes).
  - Files: `src/bin/vibe-attack-config.rs`, `src/ui/config_app.rs`
  - Verify: Launch app: log lines appear in the scrollable area; each probe::run() call generates a visible log line; area auto-scrolls on new lines

## Files Likely Touched

- src/ui/config_app.rs
- src/bin/vibe-attack-config.rs
