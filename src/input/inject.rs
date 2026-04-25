//! uinput virtual keyboard injection (MCRO-01, MCRO-02, MCRO-05, D-05, D-07).
//!
//! Architecture:
//! - VirtualDeviceBuilder creates a keyboard-only device via /dev/uinput (D-05)
//! - Injection runs on a dedicated std::thread::spawn OS thread (D-07)
//! - Key timing: dwell_ms (key hold) + gap_ms (inter-key gap); per-key override via D-06
//!
//! CRITICAL — Pitfall 6 from RESEARCH.md:
//! VirtualDevice::emit() automatically appends SYN_REPORT after the provided slice.
//! Do NOT manually append SYN_REPORT — doing so sends a double-SYN which garbles events.

use anyhow::Result;
use evdev::{uinput::VirtualDevice, AttributeSet, InputEvent, KeyCode};
use std::sync::mpsc;
use std::time::Duration;

/// Keys declared on the virtual keyboard.
/// A broad but conservative set covering arrow keys, WASD, and HD2-relevant keys.
/// VirtualDeviceBuilder requires declaring all keys the device will ever emit upfront.
const VIRTUAL_KEYBOARD_KEYS: &[KeyCode] = &[
    // Arrow keys (HD2 stratagem directional inputs)
    KeyCode::KEY_UP,
    KeyCode::KEY_DOWN,
    KeyCode::KEY_LEFT,
    KeyCode::KEY_RIGHT,
    // WASD
    KeyCode::KEY_W,
    KeyCode::KEY_A,
    KeyCode::KEY_S,
    KeyCode::KEY_D,
    // Common game keys
    KeyCode::KEY_SPACE,
    KeyCode::KEY_ENTER,
    KeyCode::KEY_ESC,
    KeyCode::KEY_LEFTCTRL,
    KeyCode::KEY_LEFTSHIFT,
    KeyCode::KEY_LEFTALT,
    // Function keys (PTT candidates)
    KeyCode::KEY_F1,
    KeyCode::KEY_F2,
    KeyCode::KEY_F3,
    KeyCode::KEY_F4,
    KeyCode::KEY_F5,
    KeyCode::KEY_F6,
    KeyCode::KEY_F7,
    KeyCode::KEY_F8,
    KeyCode::KEY_F9,
    KeyCode::KEY_F10,
    KeyCode::KEY_F11,
    KeyCode::KEY_F12,
    KeyCode::KEY_F13,
    KeyCode::KEY_F14,
    KeyCode::KEY_F15,
    // Number row + grave (common PTT keys)
    KeyCode::KEY_GRAVE,
    KeyCode::KEY_1,
    KeyCode::KEY_2,
    KeyCode::KEY_3,
    KeyCode::KEY_4,
    KeyCode::KEY_5,
    KeyCode::KEY_6,
    KeyCode::KEY_7,
    KeyCode::KEY_8,
    KeyCode::KEY_9,
    KeyCode::KEY_0,
    // Tab, backspace (sometimes used in macro sequences)
    KeyCode::KEY_TAB,
    KeyCode::KEY_BACKSPACE,
];

/// Commands sent to the injection thread via mpsc channel.
pub enum MacroCmd {
    /// Execute a key sequence. Keys are evdev key names (e.g., "KEY_UP").
    Execute {
        /// Ordered list of keys to press, with optional per-key timing overrides.
        keys: Vec<KeyStep>,
        /// Global timing defaults (from config) to use when per-key override is absent.
        default_dwell_ms: u64,
        default_gap_ms: u64,
    },
    /// Signal the injection thread to exit cleanly.
    Shutdown,
}

/// One step in a macro sequence.
pub struct KeyStep {
    pub key_name: String,
    /// Per-key dwell override (milliseconds). None = use global default.
    pub dwell_ms: Option<u64>,
    /// Per-key gap override (milliseconds). None = use global default.
    pub gap_ms: Option<u64>,
}

impl KeyStep {
    pub fn from_config(action: &crate::config::KeyAction) -> Self {
        KeyStep {
            key_name: action.key.clone(),
            dwell_ms: action.dwell_ms,
            gap_ms: action.gap_ms,
        }
    }
}

/// Open a keyboard-only uinput VirtualDevice named "vibe-attack" (D-05, MCRO-05).
///
/// Returns DaemonError::UinputPermissionDenied on EACCES — the Display message
/// is the exact actionable error the user sees (D-15).
///
/// Pitfall 3 mitigation: error message specifies 'input' group, not 'uinput' group,
/// because systemd v258+ ignores non-system groups in udev rules.
pub fn open_uinput_device() -> Result<evdev::uinput::VirtualDevice> {
    let mut keys = AttributeSet::<KeyCode>::new();
    for &k in VIRTUAL_KEYBOARD_KEYS {
        keys.insert(k);
    }

    VirtualDevice::builder()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                anyhow::anyhow!("{}", crate::error::DaemonError::UinputPermissionDenied)
            } else {
                anyhow::anyhow!("Failed to open /dev/uinput: {e}")
            }
        })?
        .name("vibe-attack")
        .with_keys(&keys)
        .map_err(|e| anyhow::anyhow!("Failed to configure virtual keys: {e}"))?
        .build()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                anyhow::anyhow!("{}", crate::error::DaemonError::UinputPermissionDenied)
            } else {
                anyhow::anyhow!("Failed to build VirtualDevice: {e}")
            }
        })
}

/// Emit a single key press-hold-release sequence (MCRO-01, MCRO-02).
///
/// Sequence: KEY_DOWN → sleep(dwell_ms) → KEY_UP → sleep(gap_ms)
///
/// CRITICAL — Pitfall 6 from RESEARCH.md:
/// `VirtualDevice::emit()` automatically appends SYN_REPORT. Do NOT pass SYN_REPORT
/// manually — that sends a double-SYN and garbles the event stream.
fn emit_key_action(
    device: &mut evdev::uinput::VirtualDevice,
    key: KeyCode,
    dwell_ms: u64,
    gap_ms: u64,
) -> Result<()> {
    use evdev::EventType;

    // KEY_DOWN (value = 1)
    device
        .emit(&[InputEvent::new(EventType::KEY.0, key.0, 1)])
        .map_err(|e| anyhow::anyhow!("Failed to emit KEY_DOWN for {key:?}: {e}"))?;

    std::thread::sleep(Duration::from_millis(dwell_ms));

    // KEY_UP (value = 0)
    device
        .emit(&[InputEvent::new(EventType::KEY.0, key.0, 0)])
        .map_err(|e| anyhow::anyhow!("Failed to emit KEY_UP for {key:?}: {e}"))?;

    std::thread::sleep(Duration::from_millis(gap_ms));

    tracing::trace!(key = ?key, dwell_ms, gap_ms, "Key action emitted");
    Ok(())
}

/// Spawn the injection thread on a dedicated OS thread (D-07).
///
/// The thread owns the VirtualDevice and executes MacroCmd::Execute messages
/// received on the channel. Exits on MacroCmd::Shutdown or channel disconnect.
///
/// Use `std::thread::spawn` (NOT `tokio::task::spawn_blocking`): this thread is
/// long-lived and uses std::thread::sleep for precise timing (D-07).
pub fn spawn_injection_thread(
    mut device: evdev::uinput::VirtualDevice,
    rx: mpsc::Receiver<MacroCmd>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        tracing::info!("Injection thread started");
        loop {
            match rx.recv() {
                Ok(MacroCmd::Execute {
                    keys,
                    default_dwell_ms,
                    default_gap_ms,
                }) => {
                    tracing::debug!("Executing macro with {} key(s)", keys.len());
                    for step in &keys {
                        let key = match crate::input::ptt::parse_key_code(&step.key_name) {
                            Ok(k) => k,
                            Err(e) => {
                                tracing::error!("Invalid key '{}': {e}", step.key_name);
                                continue;
                            }
                        };
                        let dwell = step.dwell_ms.unwrap_or(default_dwell_ms);
                        let gap = step.gap_ms.unwrap_or(default_gap_ms);
                        if let Err(e) = emit_key_action(&mut device, key, dwell, gap) {
                            tracing::error!("Inject error: {e}");
                        }
                    }
                }
                Ok(MacroCmd::Shutdown) => {
                    tracing::info!("Injection thread: shutdown received, exiting");
                    break;
                }
                Err(_) => {
                    // Channel disconnected — daemon is shutting down
                    tracing::debug!("Injection thread: channel closed, exiting");
                    break;
                }
            }
        }
        tracing::info!("Injection thread stopped");
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uinput_permission_denied_error_contains_required_strings() {
        // Test the error message format (D-15) without opening /dev/uinput
        let err_msg = format!("{}", crate::error::DaemonError::UinputPermissionDenied);
        assert!(
            err_msg.contains("cannot open /dev/uinput"),
            "Error must say 'cannot open /dev/uinput', got: {err_msg}"
        );
        assert!(
            err_msg.contains("modprobe uinput"),
            "Error must contain 'modprobe uinput', got: {err_msg}"
        );
        assert!(
            err_msg.contains("usermod -aG input"),
            "Error must contain 'usermod -aG input' (not uinput group), got: {err_msg}"
        );
        assert!(
            err_msg.contains("newgrp input"),
            "Error must contain 'newgrp input' for immediate use, got: {err_msg}"
        );
    }

    #[test]
    fn key_step_from_config_copies_overrides() {
        let action = crate::config::KeyAction {
            key: "KEY_UP".to_string(),
            dwell_ms: Some(100),
            gap_ms: None,
        };
        let step = KeyStep::from_config(&action);
        assert_eq!(step.key_name, "KEY_UP");
        assert_eq!(step.dwell_ms, Some(100));
        assert_eq!(step.gap_ms, None);
    }

    #[test]
    fn macro_cmd_shutdown_stops_thread() {
        // Test that the injection thread exits when it receives Shutdown.
        // Uses a fake device created in-memory (no /dev/uinput required).
        // We can't create a real VirtualDevice without privileges, so we test
        // the channel mechanic by checking the thread joins after Shutdown.
        // NOTE: This test only validates thread lifecycle, not actual key emission.
        // Key emission is tested in tests/macro_inject.rs (privileged integration test).
        // For this unit test, we skip if /dev/uinput is inaccessible.
        match open_uinput_device() {
            Err(_) => {
                // /dev/uinput not accessible in this environment — skip
                eprintln!("Skipping macro_cmd_shutdown_stops_thread: /dev/uinput inaccessible");
            }
            Ok(device) => {
                let (tx, rx) = std::sync::mpsc::channel();
                let handle = spawn_injection_thread(device, rx);
                tx.send(MacroCmd::Shutdown).unwrap();
                handle.join().expect("Injection thread must exit cleanly after Shutdown");
            }
        }
    }

    #[test]
    #[ignore = "requires /dev/uinput — set RUN_PRIVILEGED_TESTS=1"]
    fn open_uinput_device_succeeds_with_privileges() {
        let mut device = open_uinput_device().expect("Must open with input group access");
        // Verify at least one dev node exists
        let devnodes = device
            .enumerate_dev_nodes_blocking()
            .expect("Must enumerate dev nodes");
        let nodes: Vec<_> = devnodes.collect();
        assert!(!nodes.is_empty(), "VirtualDevice must have at least one dev node");
    }
}
