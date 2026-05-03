/// Wire types for the Unix-socket control protocol: [`ControlRequest`], [`ControlResponse`], [`DaemonStatus`].
pub mod protocol;
/// Client-side helpers for sending commands to a running daemon from CLI subcommands.
pub mod client;

use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock,
};
use anyhow::{Context, Result};
use tokio::net::UnixListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio_util::sync::CancellationToken;
use crate::control::protocol::{ActivationMode, ControlRequest, ControlResponse, DaemonState, DaemonStatus};
use crate::pipeline::coordinator::RuntimeCommand;
use crate::pipeline::dispatcher::Dispatcher;

/// Shared handle passed to the control listener and polled by tray/UI.
///
/// All fields are cheap to clone (Arc-wrapped).
#[derive(Clone)]
pub struct DaemonHandle {
    /// Macro dispatcher shared with the pipeline; used to push profile updates.
    pub dispatcher: Arc<Dispatcher>,
    /// Set true to suppress audio processing. The pipeline thread checks this
    /// on each frame and skips wake/VAD/STT while muted.
    pub muted: Arc<AtomicBool>,
    /// Name of the currently active profile, updated on every SwitchProfile.
    pub active_profile: Arc<RwLock<Option<String>>>,
    /// Set true while the wake-word listen window is open (written by coordinator).
    pub listening: Arc<AtomicBool>,
    /// Set true while PTT is held (written by coordinator).
    pub recording: Arc<AtomicBool>,
    /// Channel to the running pipeline coordinator for live runtime changes.
    /// `None` when the pipeline is not running (e.g. during unit tests).
    pub runtime_cmd_tx: Option<Arc<std::sync::mpsc::Sender<RuntimeCommand>>>,
    /// Currently cached activation mode; updated by the SetMode handler before
    /// forwarding to the coordinator so Status responses stay coherent.
    pub active_mode: Arc<RwLock<ActivationMode>>,
    /// Cancellation token for graceful shutdown; cancelled by the Shutdown command.
    pub shutdown: Option<CancellationToken>,
}

impl DaemonHandle {
    /// Create a new handle wrapping the given dispatcher with all state flags initialised to false/None.
    pub fn new(dispatcher: Arc<Dispatcher>) -> Self {
        Self {
            dispatcher,
            muted: Arc::new(AtomicBool::new(false)),
            active_profile: Arc::new(RwLock::new(None)),
            listening: Arc::new(AtomicBool::new(false)),
            recording: Arc::new(AtomicBool::new(false)),
            runtime_cmd_tx: None,
            active_mode: Arc::new(RwLock::new(ActivationMode::Ptt)),
            shutdown: None,
        }
    }

    /// Attach the shutdown cancellation token so the Shutdown control command
    /// can trigger a graceful exit.
    pub fn with_shutdown(mut self, token: CancellationToken) -> Self {
        self.shutdown = Some(token);
        self
    }

    /// Attach the runtime command sender produced by `spawn_pipeline`.
    pub fn with_runtime_tx(mut self, tx: std::sync::mpsc::Sender<RuntimeCommand>) -> Self {
        self.runtime_cmd_tx = Some(Arc::new(tx));
        self
    }

    /// Send a [`RuntimeCommand`] to the pipeline coordinator.
    ///
    /// Returns `ControlResponse::Ok` on success or
    /// `ControlResponse::Error { message }` when the pipeline is not running.
    fn send_runtime_cmd(&self, cmd: RuntimeCommand) -> ControlResponse {
        match &self.runtime_cmd_tx {
            Some(tx) => match tx.send(cmd) {
                Ok(()) => ControlResponse::Ok,
                Err(_) => ControlResponse::Error { message: "pipeline not running".into() },
            },
            None => ControlResponse::Error { message: "pipeline not running".into() },
        }
    }

    /// Derive the current [`DaemonState`] from the atomic flags, in priority order: Muted > Recording > Listening > Idle.
    pub fn state(&self) -> DaemonState {
        if self.muted.load(Ordering::Relaxed) {
            DaemonState::Muted
        } else if self.recording.load(Ordering::Relaxed) {
            DaemonState::Recording
        } else if self.listening.load(Ordering::Relaxed) {
            DaemonState::Listening
        } else {
            DaemonState::Idle
        }
    }

    /// Build a full [`DaemonStatus`] snapshot (state + active profile + macro count + active mode) for Status queries.
    pub fn status(&self) -> DaemonStatus {
        let macro_count = self.dispatcher.macro_count();
        DaemonStatus {
            state: self.state(),
            active_profile: self.active_profile.read().unwrap().clone(),
            macro_count,
            active_mode: self.active_mode.read().unwrap().clone(),
        }
    }
}

/// Spawn the control channel listener on a Tokio task.
///
/// Listens on a Unix Domain Socket for commands from the CLI.
pub async fn spawn_control_listener(handle: DaemonHandle) -> Result<()> {
    let socket_path = get_socket_path()?;

    // Clean up existing socket file if it exists
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)
            .with_context(|| format!("Failed to remove existing socket: {}", socket_path.display()))?;
    }

    let listener = UnixListener::bind(&socket_path)
        .with_context(|| format!("Failed to bind to UDS socket: {}", socket_path.display()))?;

    // Set permissions to 0600 (user only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o600))?;
    }

    tracing::info!("Control channel listening on: {}", socket_path.display());

    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((mut stream, _)) => {
                    let h = handle.clone();
                    tokio::spawn(async move {
                        let (reader, mut writer) = stream.split();
                        let mut reader = BufReader::new(reader);
                        let mut line = String::new();

                        if let Ok(n) = reader.read_line(&mut line).await {
                            if n == 0 { return; }

                            let response = match serde_json::from_str::<ControlRequest>(&line) {
                                Ok(req) => {
                                    tracing::debug!("Control request: {:?}", req);
                                    match req {
                                        ControlRequest::Ping => ControlResponse::Pong,
                                        ControlRequest::Status => {
                                            ControlResponse::StatusData(h.status())
                                        }
                                        ControlRequest::Mute => {
                                            h.muted.store(true, Ordering::Relaxed);
                                            tracing::info!("Daemon muted via control socket");
                                            ControlResponse::Ok
                                        }
                                        ControlRequest::Unmute => {
                                            h.muted.store(false, Ordering::Relaxed);
                                            tracing::info!("Daemon unmuted via control socket");
                                            ControlResponse::Ok
                                        }
                                        ControlRequest::SwitchProfile { name } => {
                                            match tokio::task::block_in_place(|| {
                                                handle_switch_profile(&name, &h)
                                            }) {
                                                Ok(_) => ControlResponse::Ok,
                                                Err(e) => ControlResponse::Error { message: e.to_string() },
                                            }
                                        }
                                        ControlRequest::Shutdown => {
                                            tracing::info!("Shutdown requested via control socket");
                                            if let Some(token) = &h.shutdown {
                                                token.cancel();
                                            }
                                            ControlResponse::Ok
                                        }
                                        ControlRequest::SetMode { mode } => {
                                            *h.active_mode.write().unwrap() = mode.clone();
                                            tracing::debug!("SetMode: cached active_mode={mode:?}, forwarding to coordinator");
                                            h.send_runtime_cmd(RuntimeCommand::SetMode(mode))
                                        }
                                        ControlRequest::SetThreshold { threshold } => {
                                            h.send_runtime_cmd(RuntimeCommand::SetThreshold(threshold))
                                        }
                                        ControlRequest::SetInputDevice { device } => {
                                            tracing::info!("SetInputDevice received; requires daemon restart (S01 note)");
                                            h.send_runtime_cmd(RuntimeCommand::SetInputDevice(device))
                                        }
                                        ControlRequest::SetPttBinding { key } => {
                                            tracing::info!("SetPttBinding received; requires daemon restart (S01 note)");
                                            h.send_runtime_cmd(RuntimeCommand::SetPttBinding(key))
                                        }
                                        ControlRequest::ReloadConfig => {
                                            h.send_runtime_cmd(RuntimeCommand::ReloadConfig)
                                        }
                                        ControlRequest::TestMacro { name } => {
                                            tracing::info!(macro_name = %name, "TestMacro request received");
                                            match tokio::task::block_in_place(|| h.dispatcher.fire_named(&name)) {
                                                Ok(_) => ControlResponse::Ok,
                                                Err(msg) => ControlResponse::Error { message: msg },
                                            }
                                        }
                                    }
                                }
                                Err(e) => ControlResponse::Error { message: format!("Invalid JSON: {e}") },
                            };

                            if let Ok(resp_json) = serde_json::to_string(&response) {
                                let _ = writer.write_all(resp_json.as_bytes()).await;
                                let _ = writer.write_all(b"\n").await;
                            }
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("Control listener accept error: {e}");
                }
            }
        }
    });

    Ok(())
}

fn handle_switch_profile(name: &str, handle: &DaemonHandle) -> Result<()> {
    use crate::pack::Pack;
    use crate::pack::manager::ProfileManager;

    let dir = crate::pack::get_profiles_dir()?.join(name);
    let pack = Pack::load_from_dir(&dir)?;
    handle.dispatcher.update_macros(pack.flatten());

    // Track active profile for status queries
    *handle.active_profile.write().unwrap() = Some(name.to_string());

    // Update persistence so it survives restart
    let mut manager = ProfileManager::load().unwrap_or(ProfileManager { active_profile: None });
    manager.active_profile = Some(name.to_string());
    manager.save()?;

    Ok(())
}

// Server-side path resolution: uses place_runtime_file, which creates the XDG runtime
// directory if it doesn't exist yet. The read-only counterpart lives in control/client.rs.
fn get_socket_path() -> Result<PathBuf> {
    let xdg = xdg::BaseDirectories::with_prefix("vibe-attack");

    xdg.place_runtime_file("vibe-attack.sock")
        .context("Failed to place UDS socket in runtime directory")
}
