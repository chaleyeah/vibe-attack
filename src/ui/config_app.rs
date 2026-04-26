use tracing::info;
use xdg::BaseDirectories;

/// Maximum number of log lines retained in the config UI.
pub const MAX_LOG_LINES: usize = 100;

/// Pure-logic state for the configuration application window.
pub struct ConfigApp {
    pub profiles: Vec<String>,
    pub active_profile: Option<String>,
    pub log_lines: Vec<String>,
    pub mic_level: f32,
    pub mic_no_device: bool,
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
}

impl Default for ConfigApp {
    fn default() -> Self {
        Self::new()
    }
}

/// Load profile names from the XDG config profiles directory.
///
/// Returns the file stems (without `.yaml`) of all `.yaml` files found.
/// Logs the count via tracing::info.
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
            if path.extension().and_then(|e| e.to_str()) == Some("yaml") {
                path.file_stem()
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
