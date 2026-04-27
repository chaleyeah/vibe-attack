use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use ksni::{menu::StandardItem, MenuItem, Tray, TrayMethods};

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

        // Channel so we can detect spawn failure before returning.
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
                    };
                    match tray.spawn().await {
                        Ok(_handle) => {
                            let _ = tx.send(Ok(()));
                            // Run forever — tray lives until process exits.
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
}

impl Tray for VibeTray {
    fn id(&self) -> String {
        "vibe-attack".into()
    }

    fn icon_name(&self) -> String {
        "audio-input-microphone".into()
    }

    fn title(&self) -> String {
        "Vibe Attack".into()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            title: "Vibe Attack".into(),
            description: "Voice macro daemon".into(),
            ..Default::default()
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let open_flag = Arc::clone(&self.open_window);
        vec![
            StandardItem {
                label: "Open Config".into(),
                icon_name: "preferences-system".into(),
                activate: Box::new(move |_this: &mut Self| {
                    open_flag.store(true, Ordering::Release);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|_| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
        ]
    }
}
