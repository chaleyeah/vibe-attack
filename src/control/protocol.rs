use serde::{Deserialize, Serialize};

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
    pub state: DaemonState,
    /// Name of the currently loaded profile, if any.
    pub active_profile: Option<String>,
    /// Number of macros currently loaded.
    pub macro_count: usize,
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
