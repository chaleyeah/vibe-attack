---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Create src/ui/probe.rs with check functions and probe::run()

Create src/ui/probe.rs. Import xdg, std::fs, std::os::unix::fs::OpenOptionsExt, tracing. Define four private check functions: check_config(xdg) -> bool, check_model(xdg) -> bool, check_uinput() -> bool, check_ptt(config_path) -> bool. Each returns false and emits tracing::warn with reason on failure. Define pub fn run() -> FirstRunState that calls all four and constructs FirstRunState::from_checks(...). Use xdg::BaseDirectories::with_prefix('vibe-attack') for XDG paths. Model path: data_home/vibe-attack/models/whisper/ggml-tiny.en.bin. Config path: config_home/vibe-attack/config.yaml. uinput check: OpenOptions::new().read(true).write(true).custom_flags(libc::O_NONBLOCK).open('/dev/uinput') — if open succeeds it's accessible. PTT check: read config file and grep for 'key: KEY_' pattern.

## Inputs

- `src/ui/first_run.rs (FirstRunState API)`
- `config.example.yaml (ptt.key format)`
- `scripts/setup.sh (step names and paths for consistency)`

## Expected Output

- `src/ui/probe.rs with pub fn run() -> FirstRunState`
- `src/ui/mod.rs exports probe module`
- `cargo check --lib exits 0`

## Verification

cargo check --lib passes with probe module added to src/ui/mod.rs
