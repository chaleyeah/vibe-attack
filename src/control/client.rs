use anyhow::{Context, Result};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use crate::control::protocol::{ControlRequest, ControlResponse, DaemonStatus};
use std::path::PathBuf;

/// Send a command to the running daemon via the UDS socket.
pub fn send_command(req: ControlRequest) -> Result<ControlResponse> {
    let socket_path = get_socket_path()?;

    let mut stream = UnixStream::connect(&socket_path)
        .with_context(|| format!("Failed to connect to daemon at {}. Is it running?", socket_path.display()))?;

    let req_json = serde_json::to_string(&req)?;
    stream.write_all(req_json.as_bytes())?;
    stream.write_all(b"\n")?;

    let mut reader = BufReader::new(stream);
    let mut response_line = String::new();
    reader.read_line(&mut response_line)?;

    let response: ControlResponse = serde_json::from_str(&response_line)
        .with_context(|| format!("Failed to parse daemon response: {response_line}"))?;

    Ok(response)
}

/// Query live daemon status. Returns `None` if the daemon is not running.
pub fn query_status() -> Option<DaemonStatus> {
    match send_command(ControlRequest::Status) {
        Ok(ControlResponse::StatusData(status)) => Some(status),
        _ => None,
    }
}

/// Returns true if the daemon socket exists (daemon likely running).
pub fn is_daemon_running() -> bool {
    get_socket_path().map(|p| p.exists()).unwrap_or(false)
}

// Client-side path resolution: uses find_runtime_file, which is read-only and returns an
// error if the socket doesn't exist (i.e. the daemon isn't running). The directory-creating
// counterpart lives in control/mod.rs.
fn get_socket_path() -> Result<PathBuf> {
    let xdg = xdg::BaseDirectories::with_prefix("vibe-attack");
    xdg.find_runtime_file("vibe-attack.sock")
        .context("Could not find daemon socket. Is the daemon running?")
}
