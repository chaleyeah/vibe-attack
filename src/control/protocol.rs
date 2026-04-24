use serde::{Deserialize, Serialize};

/// Requests sent from the CLI to the daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", content = "args", rename_all = "snake_case")]
pub enum ControlRequest {
    /// Check if the daemon is alive.
    Ping,
    /// Switch the active macro pack/profile.
    SwitchProfile { name: String },
    /// Execute a specific macro by name immediately (for testing).
    TestMacro { name: String },
    /// Gracefully shut down the daemon.
    Shutdown,
}

/// Responses sent from the daemon to the CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", content = "data", rename_all = "snake_case")]
pub enum ControlResponse {
    /// Command was received and is being processed.
    Ok,
    /// Response to a Ping.
    Pong,
    /// An error occurred during processing.
    Error { message: String },
}
