use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Top-level daemon configuration.
/// Loaded from $XDG_CONFIG_HOME/vibe-attack/config.yaml (D-13, D-14).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Push-to-talk key binding (required).
    pub ptt: PttConfig,
    /// Global key dwell/gap timing defaults used by macros.
    pub timing: TimingConfig,
    /// Audio capture device selection (optional; defaults to system default).
    #[serde(default)]
    pub audio: AudioConfig,
    /// Pipeline-wide behavior and verbosity knobs.
    #[serde(default)]
    pub pipeline: PipelineConfig,
    /// Silero VAD thresholds and windowing parameters.
    #[serde(default)]
    pub vad: VadConfig,
    /// Whisper speech-to-text model and decode settings.
    #[serde(default)]
    pub stt: SttConfig,
    /// Sherpa-ONNX wake word spotter model paths and toggle.
    #[serde(default)]
    pub wake: WakeConfig,
    /// Ordered list of named macros the daemon can dispatch.
    #[serde(default)]
    pub macros: Vec<MacroConfig>,
}

impl Config {
    /// Validate that every model file path referenced in the config actually exists on disk.
    ///
    /// Called at daemon startup so the user gets an actionable error immediately rather than
    /// a cryptic panic when the pipeline tries to load a missing file.
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

/// Controls how much per-stage event detail the pipeline emits on stderr.
///
/// `Summary` (default) emits one line per utterance. `Stages` adds timing
/// events for each intermediate stage (VAD, wake, STT, match, dispatch).
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PipelineVerbosity {
    /// Emit one summary line per completed utterance (default).
    #[default]
    Summary,
    /// Also emit timing events for each intermediate stage (VAD, wake, STT, match, dispatch).
    Stages,
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
    /// Probability threshold above which a frame is considered speech onset. Default: 0.60.
    #[serde(default = "default_vad_start_threshold")]
    pub start_threshold: f32,
    /// Probability threshold below which a frame is considered silence (ends utterance). Default: 0.45.
    #[serde(default = "default_vad_stop_threshold")]
    pub stop_threshold: f32,
    /// Minimum consecutive speech duration (ms) before an utterance is accepted. Default: 100.
    #[serde(default = "default_min_speech_ms")]
    pub min_speech_ms: u64,
    /// Silence duration (ms) required after speech before the utterance is considered complete. Default: 200.
    #[serde(default = "default_end_silence_ms")]
    pub end_silence_ms: u64,
    /// Audio pre-rolled (ms) before the speech onset frame to avoid clipping leading phonemes. Default: 150.
    #[serde(default = "default_preroll_ms")]
    pub preroll_ms: u64,
    /// Audio appended (ms) after the end-silence threshold to avoid clipping trailing phonemes. Default: 150.
    #[serde(default = "default_tail_ms")]
    pub tail_ms: u64,
    /// Hard cap on utterance length (seconds); audio beyond this is truncated and flushed. Default: 10.
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
    200
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
    /// Whether speech-to-text decoding is active. When false the pipeline skips whisper entirely.
    #[serde(default)]
    pub enabled: bool,
    /// Path to a whisper.cpp GGML model file. Required when `enabled` is true.
    pub model_path: Option<PathBuf>,
    /// Minimum whisper segment confidence (0–1) required to accept a transcription. Default: 0.80.
    #[serde(default = "default_stt_confidence_threshold")]
    pub confidence_threshold: f32,
    /// Optional prompt injected before each decode to bias whisper toward known vocabulary.
    /// Comma-separated phrases work well (e.g. "reinforce, resupply, eagle airstrike").
    #[serde(default)]
    pub initial_prompt: Option<String>,
}

fn default_stt_confidence_threshold() -> f32 {
    0.8
}

impl Default for SttConfig {
    fn default() -> Self {
        SttConfig {
            enabled: false,
            model_path: None,
            confidence_threshold: default_stt_confidence_threshold(),
            initial_prompt: None,
        }
    }
}

/// Wake word keyword spotter configuration (`sherpa-onnx`) (Phase 2).
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct WakeConfig {
    /// Whether the sherpa-onnx keyword spotter runs before STT. When false the pipeline listens
    /// on PTT only and skips wake word detection entirely.
    #[serde(default)]
    pub enabled: bool,
    /// Path to the sherpa-onnx encoder ONNX model file. Required when `enabled` is true.
    pub encoder: Option<PathBuf>,
    /// Path to the sherpa-onnx decoder ONNX model file. Required when `enabled` is true.
    pub decoder: Option<PathBuf>,
    /// Path to the sherpa-onnx joiner ONNX model file. Required when `enabled` is true.
    pub joiner: Option<PathBuf>,
    /// Path to the BPE tokens file used by the spotter. Required when `enabled` is true.
    pub tokens: Option<PathBuf>,
    /// Path to the keywords text file (one keyword per line). Required when `enabled` is true.
    pub keywords: Option<PathBuf>,
}

/// A named macro consisting of an ordered key sequence (MCRO-01, MCRO-02).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MacroConfig {
    /// Unique identifier for this macro; used in logs and the `set_flag`/`if_flag` namespace.
    pub name: String,
    /// Transcript phrase that triggers this macro (exact match after normalization).
    /// When absent the macro can only be triggered programmatically.
    pub phrase: Option<String>,
    /// Gate condition: macro fires only when the named flag is currently set.
    pub if_flag: Option<String>,
    /// Side effect: sets the named flag after the macro's key sequence completes.
    pub set_flag: Option<String>,
    /// Optional audio file to play (via rodio) when this macro fires.
    pub sound: Option<PathBuf>,
    /// Ordered sequence of key events that form the macro body.
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

/// Return the XDG config file path (D-14), creating the config directory if needed.
/// Resolves to $XDG_CONFIG_HOME/vibe-attack/config.yaml (default: ~/.config/vibe-attack/config.yaml).
pub fn default_config_path() -> Result<PathBuf> {
    let xdg = xdg::BaseDirectories::with_prefix("vibe-attack");
    xdg.place_config_file("config.yaml")
        .context("Failed to create XDG config directory for vibe-attack")
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
