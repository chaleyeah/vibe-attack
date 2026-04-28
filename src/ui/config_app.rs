use tracing::info;
use xdg::BaseDirectories;

use crate::config::{default_config_path, Config};
use crate::control::protocol::ActivationMode;

/// Maximum number of log lines retained in the config UI.
pub const MAX_LOG_LINES: usize = 100;

/// Pure-logic state for the configuration application window.
pub struct ConfigApp {
    /// Available profile names loaded from the XDG profiles directory.
    pub profiles: Vec<String>,
    /// Name of the currently active profile, or `None` when no profile is loaded.
    pub active_profile: Option<String>,
    /// Recent log lines displayed in the UI; capped at [`MAX_LOG_LINES`].
    pub log_lines: Vec<String>,
    /// Current microphone input level (0.0–1.0), updated by the audio thread.
    pub mic_level: f32,
    /// True when no audio input device is available (shows a warning in the UI).
    pub mic_no_device: bool,
    /// Current activation mode (PTT or Wake).
    pub mode: ActivationMode,
    /// Confidence threshold as a 0–100 integer (converted from/to f32 at I/O boundaries).
    pub threshold_pct: u8,
    /// Selected audio input device name, or `None` to use the system default.
    pub input_device: Option<String>,
    /// PTT key binding string (e.g. "KEY_F13").
    pub ptt_binding: String,
    /// Status bar message surfaced to the UI after save or error.
    pub status_message: Option<String>,
    /// True when the daemon is reachable (polled each frame).
    pub daemon_running: bool,
}

impl ConfigApp {
    /// Construct a fresh ConfigApp with no profiles and no logs.
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
            active_profile: None,
            log_lines: Vec::new(),
            mic_level: 0.0,
            mic_no_device: false,
            mode: ActivationMode::Ptt,
            threshold_pct: 80,
            input_device: None,
            ptt_binding: String::new(),
            status_message: None,
            daemon_running: false,
        }
    }

    /// Number of available profiles.
    pub fn profile_count(&self) -> usize {
        self.profiles.len()
    }

    /// Append a log line, dropping the oldest entry when the cap is reached.
    pub fn add_log_line(&mut self, line: String) {
        if self.log_lines.len() >= MAX_LOG_LINES {
            self.log_lines.remove(0);
        }
        self.log_lines.push(line);
    }

    /// Copy config-derived fields from a loaded `Config` into this app state.
    ///
    /// `threshold_pct` is rounded (not truncated) and clamped to 0–100 to guard
    /// against out-of-range values in hand-edited config files.
    pub fn apply_from_config(&mut self, cfg: &Config) {
        self.threshold_pct = (cfg.stt.confidence_threshold * 100.0)
            .round()
            .clamp(0.0, 100.0) as u8;
        self.input_device = cfg.audio.device.clone();
        self.ptt_binding = cfg.ptt.key.clone();
    }

    /// Write a status message to the status bar (replaces any previous message).
    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
    }
}

impl Default for ConfigApp {
    fn default() -> Self {
        Self::new()
    }
}

/// Load config from disk into `app` state and return the full `Config` for the caller to cache.
///
/// Calls `crate::config::load`, applies the result via `apply_from_config`, and returns the
/// loaded `Config` so that T03 can retain it for subsequent saves without re-reading the file.
pub fn load_config_into_app(
    app: &mut ConfigApp,
    path_override: Option<&std::path::Path>,
) -> anyhow::Result<Config> {
    let cfg = crate::config::load(path_override)?;
    app.apply_from_config(&cfg);
    Ok(cfg)
}

/// Persist the current UI state to `~/.config/vibe-attack/config.yaml` atomically.
///
/// Clones `current`, patches the three user-editable fields (`stt.confidence_threshold`,
/// `audio.device`, `ptt.key`), serializes to YAML, writes to a sibling `.tmp` file, and
/// renames it into place so a crash during write never leaves a partial file.
///
/// # Note on `ActivationMode`
/// `mode` (PTT vs Wake) is a runtime-only flag in M008 and is NOT persisted to YAML here —
/// it has no field in `Config`. The caller (T03) sends `SetMode` over the control socket
/// to change the running daemon's mode. Adding a `mode` YAML field would change the schema
/// and is deferred to a later milestone.
pub fn save_app_to_config(
    app: &ConfigApp,
    current: &Config,
    path_override: Option<&std::path::Path>,
) -> anyhow::Result<Config> {
    let mut cfg = current.clone();
    cfg.stt.confidence_threshold = app.threshold_pct as f32 / 100.0;
    cfg.audio.device = app.input_device.clone();
    cfg.ptt.key = app.ptt_binding.clone();

    let yaml = serde_yaml_ng::to_string(&cfg)
        .map_err(|e| anyhow::anyhow!("Failed to serialize config: {e}"))?;

    let dest = match path_override {
        Some(p) => p.to_path_buf(),
        None => default_config_path()?,
    };

    // Atomic write: write to a sibling .tmp then rename so partial writes are never visible.
    let tmp = dest.with_extension("yaml.tmp");
    std::fs::write(&tmp, &yaml)
        .map_err(|e| anyhow::anyhow!("Failed to write temp config {}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, &dest)
        .map_err(|e| anyhow::anyhow!("Failed to rename temp config to {}: {e}", dest.display()))?;

    Ok(cfg)
}

/// Load profile names from the XDG config profiles directory.
///
/// Returns the directory names of all subdirectories that contain a `pack.yaml`
/// file, matching the format expected by `Pack::load_from_dir` and
/// `handle_switch_profile`. Flat `.yaml` files at the profiles root are ignored.
pub fn load_profiles() -> Vec<String> {
    let profiles_dir = BaseDirectories::with_prefix("vibe-attack")
        .get_config_home()
        .map(|p| p.join("profiles"));

    let Some(dir) = profiles_dir else {
        info!(count = 0, "Profiles dir not resolvable; no profiles loaded");
        return Vec::new();
    };

    if !dir.is_dir() {
        info!(path = %dir.display(), count = 0, "Profiles dir not found");
        return Vec::new();
    }

    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if entry.file_type().ok()?.is_dir() && path.join("pack.yaml").exists() {
                path.file_name()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();

    names.sort();
    info!(path = %dir.display(), count = names.len(), "Profiles loaded");
    names
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AudioConfig, PttConfig, SttConfig, TimingConfig};
    use serial_test::serial;

    fn minimal_config(confidence_threshold: f32) -> Config {
        Config {
            ptt: PttConfig { key: "KEY_F13".to_string() },
            timing: TimingConfig { dwell_ms: 50, gap_ms: 30 },
            audio: AudioConfig { device: None },
            pipeline: Default::default(),
            vad: Default::default(),
            stt: SttConfig {
                confidence_threshold,
                ..Default::default()
            },
            wake: Default::default(),
            macros: Vec::new(),
        }
    }

    #[test]
    fn apply_from_config_round_trips_08_to_80() {
        let mut app = ConfigApp::new();
        let cfg = minimal_config(0.8);
        app.apply_from_config(&cfg);
        assert_eq!(app.threshold_pct, 80);
    }

    #[test]
    fn apply_from_config_clamps_above_100() {
        let mut app = ConfigApp::new();
        let cfg = minimal_config(1.5);
        app.apply_from_config(&cfg);
        assert_eq!(app.threshold_pct, 100);
    }

    #[test]
    fn apply_from_config_clamps_below_0() {
        let mut app = ConfigApp::new();
        let cfg = minimal_config(-0.2);
        app.apply_from_config(&cfg);
        assert_eq!(app.threshold_pct, 0);
    }

    #[test]
    fn apply_from_config_rounds_not_truncates() {
        // 0.835 * 100 = 83.5, rounds to 84 (not 83)
        let mut app = ConfigApp::new();
        let cfg = minimal_config(0.835);
        app.apply_from_config(&cfg);
        assert_eq!(app.threshold_pct, 84);
    }

    #[test]
    fn activation_mode_round_trips_through_field() {
        let mut app = ConfigApp::new();
        assert_eq!(app.mode, ActivationMode::Ptt);
        app.mode = ActivationMode::Wake;
        assert_eq!(app.mode, ActivationMode::Wake);
    }

    #[test]
    fn set_status_writes_message() {
        let mut app = ConfigApp::new();
        assert!(app.status_message.is_none());
        app.set_status("Saved");
        assert_eq!(app.status_message.as_deref(), Some("Saved"));
    }

    #[test]
    fn apply_from_config_copies_device_and_ptt_key() {
        let mut app = ConfigApp::new();
        let mut cfg = minimal_config(0.8);
        cfg.audio.device = Some("plughw:CARD=Gaming,DEV=0".to_string());
        cfg.ptt.key = "KEY_GRAVE".to_string();
        app.apply_from_config(&cfg);
        assert_eq!(app.input_device.as_deref(), Some("plughw:CARD=Gaming,DEV=0"));
        assert_eq!(app.ptt_binding, "KEY_GRAVE");
    }

    #[test]
    #[serial]
    fn load_profiles_returns_empty_when_dir_absent() {
        let tmp = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("XDG_CONFIG_HOME", tmp.path()); }
        let profiles = load_profiles();
        unsafe { std::env::remove_var("XDG_CONFIG_HOME"); }
        assert!(profiles.is_empty());
    }

    #[test]
    #[serial]
    fn load_profiles_returns_sorted_subdirectory_names() {
        let tmp = tempfile::tempdir().unwrap();
        let profiles_dir = tmp.path().join("vibe-attack/profiles");
        for name in ["zulu", "alpha", "bravo"] {
            let subdir = profiles_dir.join(name);
            std::fs::create_dir_all(&subdir).unwrap();
            std::fs::write(subdir.join("pack.yaml"), b"").unwrap();
        }

        unsafe { std::env::set_var("XDG_CONFIG_HOME", tmp.path()); }
        let profiles = load_profiles();
        unsafe { std::env::remove_var("XDG_CONFIG_HOME"); }

        assert_eq!(profiles, vec!["alpha", "bravo", "zulu"]);
    }

    #[test]
    #[serial]
    fn load_profiles_ignores_flat_yaml_and_dirs_without_pack_yaml() {
        let tmp = tempfile::tempdir().unwrap();
        let profiles_dir = tmp.path().join("vibe-attack/profiles");
        std::fs::create_dir_all(&profiles_dir).unwrap();
        // Flat .yaml at root — must be ignored
        std::fs::write(profiles_dir.join("hd2.yaml"), b"").unwrap();
        // Dir without pack.yaml — must be ignored
        std::fs::create_dir_all(profiles_dir.join("empty-dir")).unwrap();
        // Valid subdirectory profile
        let good = profiles_dir.join("hd2");
        std::fs::create_dir_all(&good).unwrap();
        std::fs::write(good.join("pack.yaml"), b"").unwrap();

        unsafe { std::env::set_var("XDG_CONFIG_HOME", tmp.path()); }
        let profiles = load_profiles();
        unsafe { std::env::remove_var("XDG_CONFIG_HOME"); }

        assert_eq!(profiles, vec!["hd2"]);
    }

    // Minimal valid YAML fixture: only required fields; everything else uses #[serde(default)].
    fn minimal_config_yaml(confidence_threshold: f32, device: Option<&str>, ptt_key: &str) -> String {
        let device_line = match device {
            Some(d) => format!("  device: \"{d}\"\n"),
            None => String::new(),
        };
        format!(
            "ptt:\n  key: {ptt_key}\ntiming:\n  dwell_ms: 50\n  gap_ms: 30\naudio:\n{device_line}stt:\n  confidence_threshold: {confidence_threshold}\n"
        )
    }

    #[test]
    #[serial]
    fn load_config_into_app_populates_state() {
        let tmp = tempfile::tempdir().unwrap();
        let config_dir = tmp.path().join("vibe-attack");
        std::fs::create_dir_all(&config_dir).unwrap();
        let config_path = config_dir.join("config.yaml");

        let yaml = minimal_config_yaml(0.75, Some("plughw:CARD=Test"), "KEY_F14");
        std::fs::write(&config_path, yaml).unwrap();

        let mut app = ConfigApp::new();
        load_config_into_app(&mut app, Some(&config_path)).unwrap();

        assert_eq!(app.threshold_pct, 75);
        assert_eq!(app.input_device.as_deref(), Some("plughw:CARD=Test"));
        assert_eq!(app.ptt_binding, "KEY_F14");
    }

    #[test]
    #[serial]
    fn save_app_to_config_round_trips() {
        let tmp = tempfile::tempdir().unwrap();
        let config_dir = tmp.path().join("vibe-attack");
        std::fs::create_dir_all(&config_dir).unwrap();
        let config_path = config_dir.join("config.yaml");

        let yaml = minimal_config_yaml(0.80, None, "KEY_F13");
        std::fs::write(&config_path, yaml).unwrap();

        let mut app = ConfigApp::new();
        let cfg = load_config_into_app(&mut app, Some(&config_path)).unwrap();

        app.threshold_pct = 50;
        app.input_device = Some("plughw:CARD=Test".to_string());

        save_app_to_config(&app, &cfg, Some(&config_path)).unwrap();

        let reloaded = crate::config::load(Some(&config_path)).unwrap();
        assert!((reloaded.stt.confidence_threshold - 0.50).abs() < 1e-5);
        assert_eq!(reloaded.audio.device.as_deref(), Some("plughw:CARD=Test"));
    }

    #[test]
    #[serial]
    fn save_app_to_config_preserves_unknown_macros() {
        let tmp = tempfile::tempdir().unwrap();
        let config_dir = tmp.path().join("vibe-attack");
        std::fs::create_dir_all(&config_dir).unwrap();
        let config_path = config_dir.join("config.yaml");

        // Config with 2 macros to verify they survive a round-trip save.
        let yaml = concat!(
            "ptt:\n  key: KEY_F13\ntiming:\n  dwell_ms: 50\n  gap_ms: 30\n",
            "macros:\n",
            "  - name: alpha\n    keys:\n      - key: KEY_UP\n",
            "  - name: bravo\n    keys:\n      - key: KEY_DOWN\n",
        );
        std::fs::write(&config_path, yaml).unwrap();

        let mut app = ConfigApp::new();
        let cfg = load_config_into_app(&mut app, Some(&config_path)).unwrap();

        // Save without touching macros.
        save_app_to_config(&app, &cfg, Some(&config_path)).unwrap();

        let reloaded = crate::config::load(Some(&config_path)).unwrap();
        assert_eq!(reloaded.macros.len(), 2);
        assert_eq!(reloaded.macros[0].name, "alpha");
        assert_eq!(reloaded.macros[1].name, "bravo");
    }

    #[test]
    #[serial]
    fn save_app_to_config_atomic() {
        let tmp = tempfile::tempdir().unwrap();
        let config_dir = tmp.path().join("vibe-attack");
        std::fs::create_dir_all(&config_dir).unwrap();
        let config_path = config_dir.join("config.yaml");

        let yaml = minimal_config_yaml(0.80, None, "KEY_F13");
        std::fs::write(&config_path, yaml).unwrap();

        let mut app = ConfigApp::new();
        let cfg = load_config_into_app(&mut app, Some(&config_path)).unwrap();
        save_app_to_config(&app, &cfg, Some(&config_path)).unwrap();

        // No .tmp sibling should remain after a successful save.
        let tmp_file = config_path.with_extension("yaml.tmp");
        assert!(!tmp_file.exists(), "leftover .tmp file found after save");
    }

    #[test]
    fn add_log_line_respects_max_cap() {
        let mut app = ConfigApp::new();
        for i in 0..MAX_LOG_LINES + 5 {
            app.add_log_line(format!("line {i}"));
        }
        assert_eq!(app.log_lines.len(), MAX_LOG_LINES);
        // Oldest entries dropped; newest retained
        assert_eq!(app.log_lines.last().unwrap(), &format!("line {}", MAX_LOG_LINES + 4));
    }
}
