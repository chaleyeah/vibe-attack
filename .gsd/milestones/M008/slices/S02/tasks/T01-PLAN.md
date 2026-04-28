---
estimated_steps: 8
estimated_files: 2
skills_used: []
---

# T01: Extend ConfigApp with mode/threshold/device/PTT state + apply_from_config

Add the pure-logic state fields S02 needs onto `ConfigApp` so the egui panel and I/O helpers can be built against them in T02 and T03. Keep this task strictly to in-memory state and pure functions — no egui imports, no socket calls, no filesystem I/O. The new fields are: `mode: ActivationMode`, `threshold_pct: u8` (0–100 integer slider domain; converted to/from `Config.stt.confidence_threshold: f32` at I/O time per MEM gotcha about float drift), `input_device: Option<String>` (mirrors `Config.audio.device`), `ptt_binding: String` (mirrors `Config.ptt.key`), `status_message: Option<String>` (UI status bar), `daemon_running: bool` (driven each frame from `control::client::is_daemon_running()`). Add an `apply_from_config(&mut self, cfg: &Config)` method that copies the four config-derived fields onto the struct and rounds threshold via `(cfg.stt.confidence_threshold * 100.0).round().clamp(0.0, 100.0) as u8`. Add `set_status(&mut self, msg: impl Into<String>)` that writes to `status_message`. ActivationMode is re-exported from `vibe_attack::control::protocol::ActivationMode`; import it in `config_app.rs` (and add `#[derive(PartialEq)]` upstream on ActivationMode if it is missing — verify before changing). Default values when no config has been loaded: `mode = ActivationMode::Ptt`, `threshold_pct = 80`, `input_device = None`, `ptt_binding = String::new()`, `status_message = None`, `daemon_running = false`. Update `ConfigApp::new()` and the `Default` impl. Write unit tests in the existing `#[cfg(test)] mod tests` block: (a) `apply_from_config` round-trips a Config with `stt.confidence_threshold = 0.8` to `threshold_pct = 80`; (b) `apply_from_config` clamps `confidence_threshold = 1.5` to `threshold_pct = 100` and `-0.2` to `0`; (c) `apply_from_config` rounds 0.835 to 84 (no truncation); (d) ActivationMode round-trips through the field; (e) `set_status` writes the message. Do NOT touch `vibe-attack-config.rs` in this task — it still calls `show_main_config(ui, &self.config)` against the old read-only signature. T03 will rewrite that call site.

Must-haves:
- ConfigApp has all six new public fields with the documented types
- apply_from_config method exists and is unit-tested for clamping, rounding, round-trip
- ActivationMode derives `PartialEq` (verify; add if missing — it is needed for egui radio comparison in T03)
- No egui, cpal, std::fs, or socket calls introduced
- `cargo test --lib` passes; `cargo clippy --all-targets -- -D warnings` clean (default features); `cargo clippy --all-targets --features gui -- -D warnings` clean

Assumption: ActivationMode is currently `#[derive(Debug, Clone, Copy, Serialize, Deserialize)]` per MEM030/S01. If `PartialEq` is already derived, leave it. If not, derive it (it is a 2-variant unit enum so this is safe).

## Inputs

- ``src/ui/config_app.rs``
- ``src/control/protocol.rs``
- ``src/config.rs``

## Expected Output

- ``src/ui/config_app.rs``
- ``src/control/protocol.rs``

## Verification

cargo test --lib config_app && cargo clippy --all-targets -- -D warnings && cargo clippy --all-targets --features gui -- -D warnings

## Observability Impact

Adds status_message and daemon_running fields that downstream UI tasks render. No runtime boundary crossed in this task.
