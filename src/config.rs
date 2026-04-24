use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Top-level daemon configuration.
/// Loaded from $XDG_CONFIG_HOME/hd-linux-voice/config.yaml (D-13, D-14).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub ptt: PttConfig,
    pub timing: TimingConfig,
    #[serde(default)]
    pub audio: AudioConfig,
    #[serde(default)]
    pub pipeline: PipelineConfig,
    #[serde(default)]
    pub vad: VadConfig,
    #[serde(default)]
    pub stt: SttConfig,
    #[serde(default)]
    pub wake: WakeConfig,
    #[serde(default)]
    pub macros: Vec<MacroConfig>,
}

impl Config {
    pub fn validate_model_paths(&self) -> Result<()> {
        if self.stt.enabled {
            let model_path = self
                .stt
                .model_path
                .as_ref()
                .context("stt.enabled is true but stt.model_path is not set")?;
            ensure_file_exists(model_path, "stt.model_path")?;
        }

        if self.wake.enabled {
            let required = [
                (self.wake.encoder.as_ref(), "wake.encoder"),
                (self.wake.decoder.as_ref(), "wake.decoder"),
                (self.wake.joiner.as_ref(), "wake.joiner"),
                (self.wake.tokens.as_ref(), "wake.tokens"),
                (self.wake.keywords.as_ref(), "wake.keywords"),
            ];

            for (path, field) in required {
                let path = path.with_context(|| format!("{field} is required when wake.enabled is true"))?;
                ensure_file_exists(path, field)?;
            }
        }

        Ok(())
    }
}

fn ensure_file_exists(path: &PathBuf, field: &str) -> Result<()> {
    std::fs::metadata(path).with_context(|| {
        format!(
            "{field} points to a missing file: {}\n\
             Provide a valid local path (no downloads are performed automatically).",
            path.display()
        )
    })?;
    Ok(())
}

/// Audio capture device selection.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AudioConfig {
    /// CPAL device name to use for audio capture.
    /// Leave unset to use the system default input device.
    /// Run with --list-devices to see available names.
    /// Example (Linux/ALSA): "plughw:CARD=Gamin,DEV=0"
    pub device: Option<String>,
}

/// Push-to-talk configuration (ACT-01).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PttConfig {
    /// evdev key name, e.g. "KEY_F13" or "KEY_GRAVE".
    /// Parsed to evdev::KeyCode at daemon startup.
    pub key: String,
}

/// Global key timing defaults (D-06 / MCRO-01 / MCRO-02).
/// Per-key overrides in MacroConfig::keys take precedence.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TimingConfig {
    /// How long each key is held down (milliseconds). Default: 50.
    pub dwell_ms: u64,
    /// Gap between consecutive key events (milliseconds). Default: 30.
    pub gap_ms: u64,
}

impl Default for TimingConfig {
    fn default() -> Self {
        TimingConfig { dwell_ms: 50, gap_ms: 30 }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineVerbosity {
    Summary,
    Stages,
}

impl Default for PipelineVerbosity {
    fn default() -> Self {
        PipelineVerbosity::Summary
    }
}

/// Pipeline-wide behavior and tuning knobs (Phase 2).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineConfig {
    /// Controls whether per-stage timing/events are emitted (stderr) in addition to the summary.
    #[serde(default)]
    pub verbosity: PipelineVerbosity,
    /// Wake word LISTENING window duration after trigger (seconds).
    #[serde(default = "default_listen_window_secs")]
    pub listen_window_secs: u64,
}

fn default_listen_window_secs() -> u64 {
    5
}

impl Default for PipelineConfig {
    fn default() -> Self {
        PipelineConfig { verbosity: PipelineVerbosity::Summary, listen_window_secs: default_listen_window_secs() }
    }
}

/// Voice activity detection parameters (Silero VAD) (Phase 2).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VadConfig {
    #[serde(default = "default_vad_start_threshold")]
    pub start_threshold: f32,
    #[serde(default = "default_vad_stop_threshold")]
    pub stop_threshold: f32,
    #[serde(default = "default_min_speech_ms")]
    pub min_speech_ms: u64,
    #[serde(default = "default_end_silence_ms")]
    pub end_silence_ms: u64,
    #[serde(default = "default_preroll_ms")]
    pub preroll_ms: u64,
    #[serde(default = "default_tail_ms")]
    pub tail_ms: u64,
    #[serde(default = "default_max_utterance_secs")]
    pub max_utterance_secs: u64,
}

fn default_vad_start_threshold() -> f32 {
    0.60
}
fn default_vad_stop_threshold() -> f32 {
    0.45
}
fn default_min_speech_ms() -> u64 {
    100
}
fn default_end_silence_ms() -> u64 {
    400
}
fn default_preroll_ms() -> u64 {
    150
}
fn default_tail_ms() -> u64 {
    150
}
fn default_max_utterance_secs() -> u64 {
    10
}

impl Default for VadConfig {
    fn default() -> Self {
        VadConfig {
            start_threshold: default_vad_start_threshold(),
            stop_threshold: default_vad_stop_threshold(),
            min_speech_ms: default_min_speech_ms(),
            end_silence_ms: default_end_silence_ms(),
            preroll_ms: default_preroll_ms(),
            tail_ms: default_tail_ms(),
            max_utterance_secs: default_max_utterance_secs(),
        }
    }
}

/// Speech-to-text configuration (whisper.cpp via `whisper-rs`) (Phase 2).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SttConfig {
    #[serde(default)]
    pub enabled: bool,
    pub model_path: Option<PathBuf>,
    #[serde(default = "default_stt_confidence_threshold")]
    pub confidence_threshold: f32,
}

fn default_stt_confidence_threshold() -> f32 {
    0.8
}

impl Default for SttConfig {
    fn default() -> Self {
        SttConfig { enabled: false, model_path: None, confidence_threshold: default_stt_confidence_threshold() }
    }
}

/// Wake word keyword spotter configuration (`sherpa-onnx`) (Phase 2).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WakeConfig {
    #[serde(default)]
    pub enabled: bool,
    pub encoder: Option<PathBuf>,
    pub decoder: Option<PathBuf>,
    pub joiner: Option<PathBuf>,
    pub tokens: Option<PathBuf>,
    pub keywords: Option<PathBuf>,
}

impl Default for WakeConfig {
    fn default() -> Self {
        WakeConfig {
            enabled: false,
            encoder: None,
            decoder: None,
            joiner: None,
            tokens: None,
            keywords: None,
        }
    }
}

/// A named macro consisting of an ordered key sequence (MCRO-01, MCRO-02).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MacroConfig {
    pub name: String,
    pub phrase: Option<String>,
    pub if_flag: Option<String>,
    pub set_flag: Option<String>,
    pub sound: Option<PathBuf>,
    pub keys: Vec<KeyAction>,
}

/// A single key event within a macro.
/// dwell_ms/gap_ms override the global TimingConfig defaults when set (D-06).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct KeyAction {
    /// evdev key name string, e.g. "KEY_UP". Parsed to evdev::KeyCode at runtime.
    pub key: String,
    /// Override global dwell_ms for this key only.
    pub dwell_ms: Option<u64>,
    /// Override global gap_ms after this key only.
    pub gap_ms: Option<u64>,
}

/// Return the XDG config file path (D-14).
/// $XDG_CONFIG_HOME/hd-linux-voice/config.yaml, or ~/.config/hd-linux-voice/config.yaml.
/// Return the XDG config file path (D-14), creating the config directory if needed.
/// $XDG_CONFIG_HOME/hd-linux-voice/config.yaml, or ~/.config/hd-linux-voice/config.yaml.
pub fn default_config_path() -> Result<PathBuf> {
    let xdg = xdg::BaseDirectories::with_prefix("hd-linux-voice");
    xdg.place_config_file("config.yaml")
        .context("Failed to create XDG config directory for hd-linux-voice")
}

/// Load and deserialize config from the given path (or XDG default).
/// Returns Err with an actionable message on any parse or I/O failure.
pub fn load(path_override: Option<&std::path::Path>) -> Result<Config> {
    let path = match path_override {
        Some(p) => p.to_path_buf(),
        None => default_config_path()?,
    };

    let file = std::fs::File::open(&path).with_context(|| {
        format!(
            "Config file not found: {}\n\
             Create it at that path. See config.example.yaml for the format.",
            path.display()
        )
    })?;

    let config: Config = serde_yaml_ng::from_reader(file).with_context(|| {
        format!("Failed to parse config file: {}", path.display())
    })?;

    Ok(config)
}
