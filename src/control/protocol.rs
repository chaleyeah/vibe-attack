use serde::{Deserialize, Serialize};

/// Activation mode for the voice pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivationMode {
    /// Push-to-talk: audio captured only while key held.
    Ptt,
    /// Wake-word: audio captured after wake word detected.
    Wake,
}

impl Default for ActivationMode {
    /// Defaults to PTT; used by `#[serde(default)]` to keep existing JSON readable.
    fn default() -> Self {
        ActivationMode::Ptt
    }
}

/// Requests sent from the CLI to the daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", content = "args", rename_all = "snake_case")]
pub enum ControlRequest {
    /// Check if the daemon is alive.
    Ping,
    /// Query full daemon status (state, active profile, macro count).
    Status,
    /// Mute the daemon (stop processing audio).
    Mute,
    /// Unmute the daemon (resume processing audio).
    Unmute,
    /// Switch the active macro pack/profile.
    SwitchProfile { name: String },
    /// Execute a specific macro by name immediately (for testing).
    TestMacro { name: String },
    /// Gracefully shut down the daemon.
    Shutdown,
    /// Change the activation mode (ptt or wake) without restarting the pipeline.
    SetMode { mode: ActivationMode },
    /// Change the VAD/wake-word confidence threshold without restarting.
    SetThreshold { threshold: f32 },
    /// Request a change to the input audio device (requires pipeline restart; noted, not applied live).
    SetInputDevice { device: Option<String> },
    /// Request a change to the PTT key binding (requires restart; noted, not applied live).
    SetPttBinding { key: String },
    /// Reload configuration from disk without restarting the daemon.
    ReloadConfig,
}

/// Runtime state of the daemon pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DaemonState {
    /// Pipeline is running and accepting audio.
    Idle,
    /// Pipeline is muted; audio is not processed.
    Muted,
    /// Wake-word listen window is active.
    Listening,
    /// PTT is held; capturing audio for STT.
    Recording,
}

/// Full daemon status snapshot returned by the Status command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    /// Current runtime state of the pipeline.
    pub state: DaemonState,
    /// Name of the currently loaded profile, if any.
    pub active_profile: Option<String>,
    /// Number of macros currently loaded.
    pub macro_count: usize,
    /// Currently active activation mode (PTT or Wake); defaults to Ptt for old JSON.
    #[serde(default)]
    pub active_mode: ActivationMode,
}

/// Responses sent from the daemon to the CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", content = "data", rename_all = "snake_case")]
pub enum ControlResponse {
    /// Command was received and is being processed.
    Ok,
    /// Response to a Ping.
    Pong,
    /// Response to a Status query.
    StatusData(DaemonStatus),
    /// An error occurred during processing.
    Error { message: String },
}
