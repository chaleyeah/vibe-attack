use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use ksni::{
    menu::{CheckmarkItem, StandardItem, SubMenu},
    MenuItem, Tray, TrayMethods,
};
use tokio::sync::Mutex;

use crate::{
    control::{
        client::{query_status, send_command},
        protocol::{ControlRequest, DaemonState},
    },
    ui::config_app::load_profiles,
};

/// Shared state polled from the daemon; updated by the background poll task.
#[derive(Clone, Default)]
struct TrayState {
    /// None = daemon not running.
    daemon_state: Option<DaemonState>,
    /// Active profile name reported by the daemon (None = none loaded or daemon stopped).
    active_profile: Option<String>,
    /// Profile names discovered from the XDG config directory.
    profiles: Vec<String>,
}

pub struct TrayHandle {
    /// Set to true by the tray "Open Config" action; cleared by the eframe loop.
    pub open_window: Arc<AtomicBool>,
    _thread: std::thread::JoinHandle<()>,
}

impl TrayHandle {
    /// Spawn the tray on a dedicated tokio thread. Returns immediately.
    /// Returns `None` if the D-Bus session bus is not available (e.g. headless CI).
    pub fn spawn() -> Option<Self> {
        let open_window = Arc::new(AtomicBool::new(false));
        let open_window_clone = Arc::clone(&open_window);

        let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();

        let thread = std::thread::Builder::new()
            .name("tray-tokio".into())
            .spawn(move || {
                let rt = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(rt) => rt,
                    Err(e) => {
                        let _ = tx.send(Err(format!("tokio runtime build failed: {e}")));
                        return;
                    }
                };

                rt.block_on(async move {
                    let tray = VibeTray {
                        open_window: Arc::clone(&open_window_clone),
                        state: Arc::new(Mutex::new(TrayState::default())),
                    };
                    let state_ref = Arc::clone(&tray.state);

                    match tray.spawn().await {
                        Ok(handle) => {
                            let _ = tx.send(Ok(()));

                            // Spawn the poll loop inside the same tokio runtime.
                            let poll_handle = handle.clone();
                            tokio::spawn(async move {
                                loop {
                                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                                    let status = query_status();
                                    let new_daemon_state = status.as_ref().map(|s| s.state.clone());
                                    let new_active_profile = status.and_then(|s| s.active_profile);
                                    // Read profile list from disk on each tick — cheap stat, no daemon needed.
                                    let new_profiles = load_profiles();

                                    // Only push a D-Bus update when the visible state actually changed.
                                    let changed = {
                                        let s = state_ref.lock().await;
                                        s.daemon_state != new_daemon_state
                                            || s.active_profile != new_active_profile
                                            || s.profiles != new_profiles
                                    };

                                    if changed {
                                        poll_handle
                                            .update(|tray: &mut VibeTray| {
                                                if let Ok(mut s) = tray.state.try_lock() {
                                                    s.daemon_state = new_daemon_state.clone();
                                                    s.active_profile = new_active_profile.clone();
                                                    s.profiles = new_profiles.clone();
                                                }
                                            })
                                            .await;
                                    }
                                }
                            });

                            std::future::pending::<()>().await;
                        }
                        Err(e) => {
                            let _ = tx.send(Err(format!("tray spawn failed: {e}")));
                        }
                    }
                });
            })
            .ok()?;

        match rx.recv() {
            Ok(Ok(())) => Some(TrayHandle {
                open_window,
                _thread: thread,
            }),
            Ok(Err(e)) => {
                tracing::warn!(reason = %e, "System tray unavailable");
                None
            }
            Err(_) => None,
        }
    }

    /// Returns true (and resets the flag) if the tray requested the window to open.
    pub fn take_open_request(&self) -> bool {
        self.open_window.swap(false, Ordering::AcqRel)
    }
}

// ── Tray impl ────────────────────────────────────────────────────────────────

struct VibeTray {
    open_window: Arc<AtomicBool>,
    state: Arc<Mutex<TrayState>>,
}

impl VibeTray {
    fn current_state(&self) -> TrayState {
        self.state
            .try_lock()
            .map(|s| s.clone())
            .unwrap_or_default()
    }
}

impl Tray for VibeTray {
    fn id(&self) -> String {
        "vibe-attack".into()
    }

    fn icon_name(&self) -> String {
        match self.current_state().daemon_state {
            None => "audio-input-microphone-muted".into(),
            Some(DaemonState::Muted) => "audio-input-microphone-muted".into(),
            Some(_) => "audio-input-microphone".into(),
        }
    }

    fn title(&self) -> String {
        "Vibe Attack".into()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        let description = match self.current_state().daemon_state {
            None => "Not running".into(),
            Some(DaemonState::Idle) => "Idle — listening for wake word".into(),
            Some(DaemonState::Muted) => "Muted".into(),
            Some(DaemonState::Listening) => "Listening…".into(),
            Some(DaemonState::Recording) => "Recording…".into(),
        };
        ksni::ToolTip {
            title: "Vibe Attack".into(),
            description,
            ..Default::default()
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let state = self.current_state();
        let is_muted = matches!(state.daemon_state, Some(DaemonState::Muted));
        let daemon_running = state.daemon_state.is_some();

        let open_flag = Arc::clone(&self.open_window);

        let mut items: Vec<MenuItem<Self>> = Vec::new();

        items.push(
            StandardItem {
                label: "Open Config".into(),
                icon_name: "preferences-system".into(),
                activate: Box::new(move |_this: &mut Self| {
                    open_flag.store(true, Ordering::Release);
                }),
                ..Default::default()
            }
            .into(),
        );

        items.push(MenuItem::Separator);

        if daemon_running {
            let mute_label = if is_muted { "Unmute" } else { "Mute" };
            let mute_icon = if is_muted {
                "audio-input-microphone"
            } else {
                "audio-input-microphone-muted"
            };
            items.push(
                StandardItem {
                    label: mute_label.into(),
                    icon_name: mute_icon.into(),
                    activate: Box::new(move |_this: &mut Self| {
                        let cmd = if is_muted {
                            ControlRequest::Unmute
                        } else {
                            ControlRequest::Mute
                        };
                        // Fire-and-forget on a fresh thread — ksni callbacks must not block.
                        std::thread::spawn(move || {
                            let _ = send_command(cmd);
                        });
                    }),
                    ..Default::default()
                }
                .into(),
            );
            items.push(MenuItem::Separator);
        }

        // Profile switcher submenu — always shown, disabled when daemon is not running.
        let profile_submenu: Vec<MenuItem<Self>> = state
            .profiles
            .iter()
            .map(|name| {
                let is_active = state.active_profile.as_deref() == Some(name.as_str());
                let name_clone = name.clone();
                CheckmarkItem {
                    label: name.clone(),
                    checked: is_active,
                    enabled: daemon_running,
                    activate: Box::new(move |_this: &mut Self| {
                        let req = ControlRequest::SwitchProfile {
                            name: name_clone.clone(),
                        };
                        std::thread::spawn(move || {
                            let _ = send_command(req);
                        });
                    }),
                    ..Default::default()
                }
                .into()
            })
            .collect();

        items.push(
            SubMenu {
                label: "Profiles".into(),
                icon_name: "folder".into(),
                enabled: !state.profiles.is_empty(),
                submenu: profile_submenu,
                ..Default::default()
            }
            .into(),
        );

        items.push(MenuItem::Separator);

        items.push(
            StandardItem {
                label: "Quit".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|_| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
        );

        items
    }
}
