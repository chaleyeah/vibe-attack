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
    pub macros: Vec<MacroConfig>,
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

/// A named macro consisting of an ordered key sequence (MCRO-01, MCRO-02).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MacroConfig {
    pub name: String,
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
