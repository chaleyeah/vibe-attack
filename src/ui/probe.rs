use std::fs;
use std::io::Read as _;
use std::path::PathBuf;

use tracing::{info, warn};
use xdg::BaseDirectories;

use crate::ui::first_run::FirstRunState;


fn xdg_config_path() -> Option<PathBuf> {
    BaseDirectories::with_prefix("vibe-attack")
        .get_config_home()
        .map(|p| p.join("config.yaml"))
}

fn xdg_data_model_path() -> Option<PathBuf> {
    BaseDirectories::with_prefix("vibe-attack")
        .get_data_home()
        .map(|p| p.join("models/whisper/ggml-tiny.en.bin"))
}

/// Resolved config file path as a display string (for wizard UI and logging).
pub fn config_path_for_display() -> String {
    xdg_config_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "~/.config/vibe-attack/config.yaml".to_string())
}

/// Resolved model file path as a display string (for wizard UI).
pub fn model_path_for_display() -> String {
    xdg_data_model_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "~/.local/share/vibe-attack/models/whisper/ggml-tiny.en.bin".to_string())
}

fn check_config() -> bool {
    let Some(path) = xdg_config_path() else {
        warn!(check = "config", reason = "could not resolve XDG config path");
        return false;
    };
    if path.is_file() {
        true
    } else {
        warn!(
            check = "config",
            path = %path.display(),
            reason = "config file not found"
        );
        false
    }
}

fn check_model() -> bool {
    let Some(path) = xdg_data_model_path() else {
        warn!(check = "model", reason = "could not resolve XDG data path");
        return false;
    };
    match path.metadata() {
        Ok(m) if m.len() > 0 => true,
        Ok(_) => {
            warn!(
                check = "model",
                path = %path.display(),
                reason = "model file exists but is empty"
            );
            false
        }
        Err(_) => {
            warn!(
                check = "model",
                path = %path.display(),
                reason = "model file not found"
            );
            false
        }
    }
}

fn check_uinput() -> bool {
    match fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/uinput")
    {
        Ok(_) => true,
        Err(e) => {
            warn!(
                check = "uinput",
                reason = %e,
                "cannot open /dev/uinput for read+write"
            );
            false
        }
    }
}

fn check_ptt() -> bool {
    let Some(path) = xdg_config_path() else {
        return false;
    };
    let Ok(mut f) = fs::File::open(&path) else {
        warn!(
            check = "ptt",
            path = %path.display(),
            reason = "config file not readable"
        );
        return false;
    };
    let mut contents = String::new();
    if f.read_to_string(&mut contents).is_err() {
        warn!(check = "ptt", reason = "could not read config file");
        return false;
    }
    // Matches lines like:  key: KEY_LEFTCTRL
    if contents.lines().any(|l| {
        let t = l.trim();
        t.starts_with("key:") && t.contains("KEY_")
    }) {
        true
    } else {
        warn!(
            check = "ptt",
            path = %path.display(),
            reason = "no 'key: KEY_*' entry found in config — set ptt.key"
        );
        false
    }
}

/// Probe the environment and return a populated [`FirstRunState`].
///
/// Each failed check emits a `tracing::warn` with the check name and reason.
pub fn run() -> FirstRunState {
    let config_path = xdg_config_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    info!(config_path, "probing first-run environment");

    FirstRunState::from_checks(
        check_config(),
        check_model(),
        check_uinput(),
        check_ptt(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    /// Set XDG_CONFIG_HOME and XDG_DATA_HOME to temp dirs for hermetic isolation.
    /// Must be used with #[serial] to avoid env-var races across parallel tests.
    fn with_xdg(f: impl FnOnce(&TempDir, &TempDir)) {
        let cfg_dir = tempfile::tempdir().unwrap();
        let data_dir = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", cfg_dir.path());
            std::env::set_var("XDG_DATA_HOME", data_dir.path());
        }
        f(&cfg_dir, &data_dir);
        unsafe {
            std::env::remove_var("XDG_CONFIG_HOME");
            std::env::remove_var("XDG_DATA_HOME");
        }
    }

    #[test]
    #[serial]
    fn config_missing_returns_false() {
        with_xdg(|_cfg, _data| {
            assert!(!check_config());
        });
    }

    #[test]
    #[serial]
    fn config_present_returns_true() {
        with_xdg(|cfg, _data| {
            let dir = cfg.path().join("vibe-attack");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("config.yaml"), b"ptt:\n  key: KEY_LEFTCTRL\n").unwrap();
            assert!(check_config());
        });
    }

    #[test]
    #[serial]
    fn model_missing_returns_false() {
        with_xdg(|_cfg, _data| {
            assert!(!check_model());
        });
    }

    #[test]
    #[serial]
    fn model_empty_returns_false() {
        with_xdg(|_cfg, data| {
            let dir = data.path().join("vibe-attack/models/whisper");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("ggml-tiny.en.bin"), b"").unwrap();
            assert!(!check_model());
        });
    }

    #[test]
    #[serial]
    fn model_present_returns_true() {
        with_xdg(|_cfg, data| {
            let dir = data.path().join("vibe-attack/models/whisper");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("ggml-tiny.en.bin"), b"not-empty").unwrap();
            assert!(check_model());
        });
    }

    #[test]
    #[serial]
    fn ptt_missing_config_returns_false() {
        with_xdg(|_cfg, _data| {
            assert!(!check_ptt());
        });
    }

    #[test]
    #[serial]
    fn ptt_no_key_returns_false() {
        with_xdg(|cfg, _data| {
            let dir = cfg.path().join("vibe-attack");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("config.yaml"), b"ptt:\n  # key: KEY_LEFTCTRL\n").unwrap();
            assert!(!check_ptt());
        });
    }

    #[test]
    #[serial]
    fn ptt_key_present_returns_true() {
        with_xdg(|cfg, _data| {
            let dir = cfg.path().join("vibe-attack");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("config.yaml"), b"ptt:\n  key: KEY_LEFTCTRL\n").unwrap();
            assert!(check_ptt());
        });
    }
}
