pub mod protocol;
pub mod client;

use std::path::PathBuf;
use anyhow::{Context, Result};
use tokio::net::UnixListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use crate::control::protocol::{ControlRequest, ControlResponse};
use crate::pipeline::dispatcher::Dispatcher;
use std::sync::Arc;

/// Spawn the control channel listener on a Tokio task.
///
/// Listens on a Unix Domain Socket for commands from the CLI.
pub async fn spawn_control_listener(dispatcher: Arc<Dispatcher>) -> Result<()> {
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
                    let dispatcher_clone = Arc::clone(&dispatcher);
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
                                        ControlRequest::SwitchProfile { name } => {
                                            match tokio::task::block_in_place(|| {
                                                handle_switch_profile(&name, &dispatcher_clone)
                                            }) {
                                                Ok(_) => ControlResponse::Ok,
                                                Err(e) => ControlResponse::Error { message: e.to_string() },
                                            }
                                        }
                                        ControlRequest::Shutdown => {
                                            // TODO: Trigger global shutdown
                                            ControlResponse::Ok
                                        }
                                        _ => ControlResponse::Error { message: "Not yet implemented".into() },
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

fn handle_switch_profile(name: &str, dispatcher: &Dispatcher) -> Result<()> {
    use crate::pack::Pack;
    use crate::pack::manager::ProfileManager;

    let dir = crate::pack::get_profiles_dir()?.join(name);
    let pack = Pack::load_from_dir(&dir)?;
    dispatcher.update_macros(pack.flatten());

    // Update persistence so it survives restart
    let mut manager = ProfileManager::load().unwrap_or(ProfileManager { active_profile: None });
    manager.active_profile = Some(name.to_string());
    manager.save()?;

    Ok(())
}

fn get_socket_path() -> Result<PathBuf> {
    let xdg = xdg::BaseDirectories::with_prefix("hd-linux-voice");

    xdg.place_runtime_file("hd-linux-voice.sock")
        .context("Failed to place UDS socket in runtime directory")
}
