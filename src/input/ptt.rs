//! evdev PTT key detection (ACT-01, D-09, D-10, D-11, D-12).
//!
//! Architecture:
//! - Scans ALL /dev/input/event* nodes (D-09) — no explicit device path in config
//! - Non-exclusive read (D-10): EVIOCGRAB is NOT called; game receives PTT events too
//! - Fail-hard (D-11): if no readable device found or device open fails
//! - TRACE-level logging only (D-12): no INFO/WARN on normal PTT press/release

use anyhow::{anyhow, Result};
use evdev::{Device, EventSummary, KeyCode};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio_util::sync::CancellationToken;

/// Parse an evdev key code name string (e.g., "KEY_F13") to a `KeyCode`.
///
/// evdev::KeyCode implements FromStr unconditionally — maps names to constants.
/// Returns Err if the name is unknown.
pub fn parse_key_code(name: &str) -> Result<KeyCode> {
    name.parse::<KeyCode>().map_err(|_| {
        anyhow!(
            "Unknown key code: '{}'. Use evdev key names like KEY_F13, KEY_GRAVE, KEY_UP",
            name
        )
    })
}

/// Preflight check: verify at least one /dev/input/event* node is readable.
///
/// Pitfall 2 mitigation: evdev::enumerate() silently skips devices it can't open.
/// We check readability first so the user gets an actionable error instead of
/// a mysterious "no PTT device found" after enumeration returns empty.
pub fn check_input_readable() -> Result<()> {
    // Scan /dev/input/event0..63 without requiring the glob crate
    let event_nodes: Vec<std::path::PathBuf> = (0..64)
        .map(|i| std::path::PathBuf::from(format!("/dev/input/event{i}")))
        .filter(|p| p.exists())
        .collect();

    if event_nodes.is_empty() {
        return Err(anyhow!(
            "No /dev/input/event* devices found.\n\
             Is the input subsystem loaded? Try: sudo modprobe evdev"
        ));
    }

    // Try opening at least one device to detect permission issues
    let any_readable = event_nodes.iter().any(|path| std::fs::File::open(path).is_ok());

    if !any_readable {
        return Err(anyhow!(
            "Cannot read /dev/input/event* — permission denied.\n\
             Fix: sudo usermod -aG input $USER\n\
             Then log out and back in, or run: newgrp input"
        ));
    }

    Ok(())
}

/// Find the first evdev device that has the given key in its supported key set.
///
/// Returns Err (D-11) if no device is found — never falls back silently.
pub fn find_ptt_device(target_key: KeyCode) -> Result<Device> {
    for (_path, device) in evdev::enumerate() {
        if device
            .supported_keys()
            .map_or(false, |keys| keys.contains(target_key))
        {
            tracing::debug!(
                "PTT device found: {} (key={:?})",
                device.name().unwrap_or("unknown"),
                target_key
            );
            return Ok(device);
        }
    }
    Err(anyhow!(
        "No input device found that reports key {:?}.\n\
         Connect the device and ensure you are in the 'input' group.\n\
         Run `evtest` to list available devices and their key codes.",
        target_key
    ))
}

/// Process a single evdev event: update ptt_active if the event matches the PTT key.
///
/// Extracted as a pure function for unit testing without a real device.
pub fn process_event(event: evdev::InputEvent, target_key: KeyCode, ptt_active: &AtomicBool) {
    // value 1 = key pressed, 0 = key released, 2 = key repeat (autorepeat)
    if let EventSummary::Key(_, k, value) = event.destructure() {
        if k == target_key {
            // Treat repeat as "still pressed" so a held PTT doesn't flicker.
            let pressed = value != 0;
            ptt_active.store(pressed, Ordering::Relaxed);
            // Make this visible at -v (DEBUG) so users can diagnose "PTT not working".
            tracing::debug!(key = ?k, value, pressed, "PTT state changed");
        }
    }
}

/// Spawn the PTT detection thread on a dedicated OS thread (not the Tokio executor).
///
/// The thread blocks on evdev::fetch_events() and updates ptt_active accordingly.
/// It exits when the shutdown flag is cancelled (checked between event batches).
///
/// Pitfall (from RESEARCH.md): Use std::thread::spawn, NOT tokio::task::spawn_blocking.
/// spawn_blocking is for short-duration tasks; the PTT loop is long-lived.
pub fn spawn_ptt_thread(
    mut device: Device,
    target_key: KeyCode,
    ptt_active: Arc<AtomicBool>,
    shutdown: CancellationToken,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        tracing::info!("PTT thread started (key={:?})", target_key);
        loop {
            if shutdown.is_cancelled() {
                tracing::debug!("PTT thread: shutdown signal received, exiting");
                break;
            }
            match device.fetch_events() {
                Ok(events) => {
                    for event in events {
                        process_event(event, target_key, &ptt_active);
                    }
                }
                Err(e) => {
                    tracing::error!("PTT device read error: {e} — PTT thread exiting");
                    break;
                }
            }
        }
        // Ensure PTT is not stuck as "active" after thread exits
        ptt_active.store(false, Ordering::Relaxed);
        tracing::info!("PTT thread stopped");
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use evdev::KeyCode;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    #[test]
    fn parse_valid_key_code() {
        let key = parse_key_code("KEY_UP").expect("KEY_UP is a valid key code");
        assert_eq!(key, KeyCode::KEY_UP);
    }

    #[test]
    fn parse_key_f13() {
        let key = parse_key_code("KEY_F13").expect("KEY_F13 is a valid key code");
        assert_eq!(key, KeyCode::KEY_F13);
    }

    #[test]
    fn parse_invalid_key_returns_err() {
        let result = parse_key_code("NOT_A_REAL_KEY");
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(
            msg.contains("NOT_A_REAL_KEY"),
            "Error message should include the bad key name, got: {msg}"
        );
    }

    #[test]
    fn check_input_readable_actionable_error_contains_group() {
        // This test is meaningful when /dev/input/event* is inaccessible.
        // When running with input group access, check_input_readable() returns Ok — that's fine.
        match check_input_readable() {
            Ok(()) => {
                // Developer has input group access — test passes trivially
            }
            Err(e) => {
                let msg = format!("{e}");
                assert!(
                    msg.contains("input") && msg.contains("usermod"),
                    "Error must mention 'input' group and 'usermod' command, got: {msg}"
                );
            }
        }
    }

    #[test]
    fn process_event_press_sets_ptt_active() {
        use evdev::{EventType, InputEvent};

        let ptt = Arc::new(AtomicBool::new(false));
        let event = InputEvent::new(EventType::KEY.0, KeyCode::KEY_F13.0, 1);
        process_event(event, KeyCode::KEY_F13, &ptt);
        assert!(ptt.load(Ordering::Relaxed), "Key press must set ptt_active=true");
    }

    #[test]
    fn process_event_release_clears_ptt_active() {
        use evdev::{EventType, InputEvent};

        let ptt = Arc::new(AtomicBool::new(true));
        let event = InputEvent::new(EventType::KEY.0, KeyCode::KEY_F13.0, 0);
        process_event(event, KeyCode::KEY_F13, &ptt);
        assert!(!ptt.load(Ordering::Relaxed), "Key release must set ptt_active=false");
    }

    #[test]
    fn process_event_different_key_does_not_change_ptt() {
        use evdev::{EventType, InputEvent};

        let ptt = Arc::new(AtomicBool::new(false));
        // KEY_SPACE pressed, but PTT key is KEY_F13
        let event = InputEvent::new(EventType::KEY.0, KeyCode::KEY_SPACE.0, 1);
        process_event(event, KeyCode::KEY_F13, &ptt);
        assert!(!ptt.load(Ordering::Relaxed), "Different key must not change ptt_active");
    }
}
