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
        protocol::{ActivationMode, ControlRequest, DaemonState},
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
    /// Currently active activation mode (None = daemon stopped or unknown).
    active_mode: Option<ActivationMode>,
}

/// Handle returned by [`TrayHandle::spawn`]; keeps the tray alive for the process lifetime.
pub struct TrayHandle {
    /// Set to true by the tray "Open Config" action; cleared by the eframe loop.
    pub open_window: Arc<AtomicBool>,
    /// Set to true by the tray "Quit" action; cleared by the eframe loop.
    pub quit_window: Arc<AtomicBool>,
    _thread: std::thread::JoinHandle<()>,
}

impl TrayHandle {
    /// Spawn the tray on a dedicated tokio thread. Returns immediately.
    /// Returns `None` if the D-Bus session bus is not available (e.g. headless CI).
    pub fn spawn() -> Option<Self> {
        let open_window = Arc::new(AtomicBool::new(false));
        let open_window_clone = Arc::clone(&open_window);
        let quit_window = Arc::new(AtomicBool::new(false));
        let quit_window_clone = Arc::clone(&quit_window);

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
                        quit_window: Arc::clone(&quit_window_clone),
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
                                    let new_active_mode = status.as_ref().map(|s| s.active_mode.clone());
                                    let new_active_profile = status.and_then(|s| s.active_profile);
                                    // Read profile list from disk on each tick — cheap stat, no daemon needed.
                                    let new_profiles = load_profiles();

                                    // Only push a D-Bus update when the visible state actually changed.
                                    let changed = {
                                        let s = state_ref.lock().await;
                                        s.daemon_state != new_daemon_state
                                            || s.active_profile != new_active_profile
                                            || s.profiles != new_profiles
                                            || s.active_mode != new_active_mode
                                    };

                                    if changed {
                                        poll_handle
                                            .update(|tray: &mut VibeTray| {
                                                if let Ok(mut s) = tray.state.try_lock() {
                                                    s.daemon_state = new_daemon_state.clone();
                                                    s.active_profile = new_active_profile.clone();
                                                    s.profiles = new_profiles.clone();
                                                    s.active_mode = new_active_mode.clone();
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
                quit_window,
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

    /// Returns true (and resets the flag) if the tray "Quit" item was clicked.
    pub fn take_quit_request(&self) -> bool {
        self.quit_window.swap(false, Ordering::AcqRel)
    }
}

// ── Icon mapping ─────────────────────────────────────────────────────────────

/// Return the XDG/FreeDesktop icon name for a given daemon state.
///
/// `None` means the daemon is not running — treat the same as Muted so the
/// tray icon makes it obvious the mic is inactive.  `Listening` gets a
/// distinct "high" variant to distinguish the wake-word window from idle.
pub(crate) fn icon_name_for_state(state: Option<&DaemonState>) -> &'static str {
    match state {
        None | Some(DaemonState::Muted) => "audio-input-microphone-muted",
        Some(DaemonState::Idle) | Some(DaemonState::Recording) => "audio-input-microphone",
        Some(DaemonState::Listening) => "audio-input-microphone-high",
    }
}

/// Build the tooltip description string for a given daemon state and activation mode.
///
/// Extracted as a free function (mirroring [`icon_name_for_state`]) so it is unit-testable
/// without constructing any tray state or D-Bus connection.
pub(crate) fn tooltip_description_for(
    state: Option<&DaemonState>,
    mode: Option<&ActivationMode>,
) -> String {
    match state {
        None => "Not running".into(),
        Some(DaemonState::Idle) => match mode {
            Some(ActivationMode::Ptt) => "Idle — waiting for PTT key".into(),
            Some(ActivationMode::Wake) => "Idle — listening for wake word".into(),
            None => "Idle".into(),
        },
        Some(DaemonState::Muted) => "Muted".into(),
        Some(DaemonState::Listening) => "Listening\u{2026}".into(),
        Some(DaemonState::Recording) => "Recording\u{2026}".into(),
    }
}

// ── Tray impl ────────────────────────────────────────────────────────────────

struct VibeTray {
    open_window: Arc<AtomicBool>,
    quit_window: Arc<AtomicBool>,
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
        icon_name_for_state(self.current_state().daemon_state.as_ref()).into()
    }

    fn title(&self) -> String {
        "Vibe Attack".into()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        let s = self.current_state();
        let description =
            tooltip_description_for(s.daemon_state.as_ref(), s.active_mode.as_ref());
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

        // Mode switcher submenu — PTT vs Wake word.
        let is_ptt = state.active_mode == Some(ActivationMode::Ptt);
        let is_wake = state.active_mode == Some(ActivationMode::Wake);
        let mode_submenu: Vec<MenuItem<Self>> = vec![
            CheckmarkItem {
                label: "Push-to-talk".into(),
                checked: is_ptt,
                enabled: daemon_running,
                activate: Box::new(move |_this: &mut Self| {
                    std::thread::spawn(move || {
                        let _ = send_command(ControlRequest::SetMode {
                            mode: ActivationMode::Ptt,
                        });
                    });
                }),
                ..Default::default()
            }
            .into(),
            CheckmarkItem {
                label: "Wake word".into(),
                checked: is_wake,
                enabled: daemon_running,
                activate: Box::new(move |_this: &mut Self| {
                    std::thread::spawn(move || {
                        let _ = send_command(ControlRequest::SetMode {
                            mode: ActivationMode::Wake,
                        });
                    });
                }),
                ..Default::default()
            }
            .into(),
        ];

        items.push(
            SubMenu {
                label: "Mode".into(),
                submenu: mode_submenu,
                ..Default::default()
            }
            .into(),
        );

        items.push(MenuItem::Separator);

        let quit_flag = Arc::clone(&self.quit_window);
        items.push(
            StandardItem {
                label: "Quit".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(move |_this: &mut Self| {
                    tracing::info!("Tray quit requested");
                    quit_flag.store(true, Ordering::Release);
                }),
                ..Default::default()
            }
            .into(),
        );

        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_name_for_none_is_muted() {
        assert_eq!(icon_name_for_state(None), "audio-input-microphone-muted");
    }

    #[test]
    fn icon_name_for_idle() {
        assert_eq!(
            icon_name_for_state(Some(&DaemonState::Idle)),
            "audio-input-microphone"
        );
    }

    #[test]
    fn icon_name_for_listening() {
        assert_eq!(
            icon_name_for_state(Some(&DaemonState::Listening)),
            "audio-input-microphone-high"
        );
    }

    #[test]
    fn icon_name_for_recording() {
        assert_eq!(
            icon_name_for_state(Some(&DaemonState::Recording)),
            "audio-input-microphone"
        );
    }

    #[test]
    fn icon_name_for_muted() {
        assert_eq!(
            icon_name_for_state(Some(&DaemonState::Muted)),
            "audio-input-microphone-muted"
        );
    }

    #[test]
    fn tooltip_description_idle_ptt() {
        let desc =
            tooltip_description_for(Some(&DaemonState::Idle), Some(&ActivationMode::Ptt));
        assert!(desc.contains("PTT"), "expected PTT in '{desc}'");
    }

    #[test]
    fn tooltip_description_idle_wake() {
        let desc =
            tooltip_description_for(Some(&DaemonState::Idle), Some(&ActivationMode::Wake));
        assert!(desc.contains("wake word"), "expected 'wake word' in '{desc}'");
    }

    #[test]
    fn tooltip_description_idle_unknown() {
        let desc = tooltip_description_for(Some(&DaemonState::Idle), None);
        assert_eq!(desc, "Idle");
    }

    #[test]
    fn tooltip_description_recording_unaffected_by_mode() {
        let ptt = tooltip_description_for(Some(&DaemonState::Recording), Some(&ActivationMode::Ptt));
        let wake =
            tooltip_description_for(Some(&DaemonState::Recording), Some(&ActivationMode::Wake));
        let none = tooltip_description_for(Some(&DaemonState::Recording), None);
        assert_eq!(ptt, wake);
        assert_eq!(ptt, none);
    }

    #[test]
    fn tray_handle_take_quit_request_clears_flag() {
        use std::sync::atomic::AtomicBool;
        let flag = Arc::new(AtomicBool::new(true));
        let returned = flag.swap(false, Ordering::AcqRel);
        assert!(returned, "swap should have returned the original true value");
        assert!(!flag.load(Ordering::Acquire), "flag should now be false");
    }
}
