/// Maximum number of log lines retained in the config UI.
pub const MAX_LOG_LINES: usize = 100;

/// Pure-logic state for the configuration application window.
pub struct ConfigApp {
    pub profiles: Vec<String>,
    pub active_profile: Option<String>,
    pub log_lines: Vec<String>,
    pub mic_level: f32,
}

impl ConfigApp {
    /// Construct a fresh ConfigApp with no profiles and no logs.
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
            active_profile: None,
            log_lines: Vec::new(),
            mic_level: 0.0,
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
