---
estimated_steps: 17
estimated_files: 1
skills_used: []
---

# T02: Add config.yaml load/save helpers with full-Config round-trip

Add two helpers in `src/ui/config_app.rs` that read and write `~/.config/vibe-attack/config.yaml` while round-tripping the full `Config` struct (all sub-structs use `#[serde(deny_unknown_fields)]` per `src/config.rs:8` — partial writes will fail to deserialize on next load). T01 fields are surfaced; this task wires file I/O for them.

Signatures (do not deviate):
- `pub fn load_config_into_app(app: &mut ConfigApp, path_override: Option<&std::path::Path>) -> anyhow::Result<Config>` — calls `crate::config::load(path_override)`, then `app.apply_from_config(&cfg)`, then returns the loaded `Config` for the caller to retain (T03 caches it on `VibeAttackConfigApp`).
- `pub fn save_app_to_config(app: &ConfigApp, current: &Config, path_override: Option<&std::path::Path>) -> anyhow::Result<Config>` — clones `current`, mutates the four owned fields (`stt.confidence_threshold = app.threshold_pct as f32 / 100.0`, `audio.device = app.input_device.clone()`, `ptt.key = app.ptt_binding.clone()`; mode is NOT in Config — see assumption below), serializes via `serde_yaml_ng::to_string`, atomically writes via temp-file + rename to the resolved path (`path_override` or `default_config_path()`), and returns the mutated `Config`. Use `std::fs::write` to a sibling `.tmp` file then `std::fs::rename` for atomic replace.

Assumption to document inline: `ActivationMode` is a runtime-only mode flag in M008; `Config` does not yet have a `mode` field. The Save path therefore sends `SetMode` over the control socket (T03) but does NOT persist mode to YAML. This is consistent with the milestone scope — adding a `mode` field to YAML would change the schema and is out of scope. Add a code comment in `save_app_to_config` documenting this.

Unit tests (use the existing `XDG_CONFIG_HOME` + `serial_test::serial` pattern from `load_profiles` tests at `src/ui/config_app.rs:99`):
- `load_config_into_app_populates_state`: write a minimal valid config.yaml to a tempdir, call `load_config_into_app(&mut app, Some(path))`, assert `app.threshold_pct`, `app.input_device`, `app.ptt_binding` match expected values.
- `save_app_to_config_round_trips`: load a config, mutate `app.threshold_pct = 50` and `app.input_device = Some("plughw:CARD=Test".to_string())`, call save, then `crate::config::load(Some(path))` and assert the new values landed.
- `save_app_to_config_preserves_unknown_macros`: load a config containing 2 macros, save without touching macros, reload, assert all 2 macros survived round-trip — proves we did not lose adjacent fields.
- `save_app_to_config_atomic`: assert the .tmp file does not remain after a successful save (no leftover sibling files in the directory).

Must-haves:
- Helpers are public and live in `src/ui/config_app.rs` (no new module — keep T03's footprint small)
- Full `Config` round-trip preserves macros, vad, wake, pipeline, timing sub-structs (proven by test)
- Atomic write via tmp+rename — partial saves on crash are not visible to next load
- Tests use tempdir + XDG_CONFIG_HOME override + `serial_test::serial` (per MEM008)
- `cargo test --lib config_app` passes; clippy clean for both default and gui features

For the test fixture YAML, use the minimum required fields per `Config` definition: `ptt.key`, `timing.dwell_ms`, `timing.gap_ms`. Everything else can rely on `#[serde(default)]` impls. Reference `config.example.yaml` at the repo root for a known-good template if needed.

## Inputs

- ``src/ui/config_app.rs``
- ``src/config.rs``
- ``config.example.yaml``

## Expected Output

- ``src/ui/config_app.rs``

## Verification

cargo test --lib config_app && cargo clippy --all-targets -- -D warnings && cargo clippy --all-targets --features gui -- -D warnings

## Observability Impact

save_app_to_config returns Result so callers can route errors into ConfigApp.status_message in T03. No new tracing — error surfacing is the egui panel's job.
