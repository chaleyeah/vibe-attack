//! Integration tests for key sequence injection (MCRO-01, MCRO-02, MCRO-05).
//!
//! These tests require /dev/uinput access (user in 'input' group).
//! Run with: RUN_PRIVILEGED_TESTS=1 cargo test --test macro_inject -- --include-ignored
//!
//! CI strategy: Standard CI runs cargo test --lib only. A separate privileged job
//! sets RUN_PRIVILEGED_TESTS=1 and runs the full suite.

use vibe_attack::input::inject::{open_uinput_device, spawn_injection_thread, KeyStep, MacroCmd};
use std::sync::mpsc;
use std::time::{Duration, Instant};

fn should_run_privileged() -> bool {
    std::env::var("RUN_PRIVILEGED_TESTS")
        .map(|v| v == "1")
        .unwrap_or(false)
}

#[test]
#[ignore = "requires /dev/uinput — set RUN_PRIVILEGED_TESTS=1"]
fn key_sequence_fires_with_configurable_gap() {
    if !should_run_privileged() {
        return;
    }

    let device = open_uinput_device().expect("Must open with input group access");
    let (tx, rx) = mpsc::channel();
    let handle = spawn_injection_thread(device, rx);

    // Send a 3-key sequence with default timing (50ms dwell, 30ms gap)
    tx.send(MacroCmd::Execute {
        keys: vec![
            KeyStep { key_name: "KEY_UP".to_string(), dwell_ms: None, gap_ms: None },
            KeyStep { key_name: "KEY_DOWN".to_string(), dwell_ms: None, gap_ms: None },
            KeyStep { key_name: "KEY_LEFT".to_string(), dwell_ms: None, gap_ms: None },
        ],
        default_dwell_ms: 50,
        default_gap_ms: 30,
    })
    .expect("Channel must be open");

    // Minimum expected time: 3 keys × (50ms dwell + 30ms gap) = 240ms
    let start = Instant::now();
    tx.send(MacroCmd::Shutdown).expect("Shutdown must send");
    handle.join().expect("Thread must exit cleanly");

    let elapsed = start.elapsed();
    assert!(
        elapsed >= Duration::from_millis(240),
        "3-key sequence must take at least 240ms (3 × 80ms), took {:?}",
        elapsed
    );
}

#[test]
#[ignore = "requires /dev/uinput — set RUN_PRIVILEGED_TESTS=1"]
fn per_key_dwell_override_is_applied() {
    if !should_run_privileged() {
        return;
    }

    let device = open_uinput_device().expect("Must open with input group access");
    let (tx, rx) = mpsc::channel();
    let handle = spawn_injection_thread(device, rx);

    // One key with 200ms dwell override vs 50ms global default
    tx.send(MacroCmd::Execute {
        keys: vec![KeyStep {
            key_name: "KEY_UP".to_string(),
            dwell_ms: Some(200), // override: 200ms instead of 50ms
            gap_ms: None,
        }],
        default_dwell_ms: 50,
        default_gap_ms: 30,
    })
    .expect("Channel must be open");

    let start = Instant::now();
    tx.send(MacroCmd::Shutdown).expect("Shutdown must send");
    handle.join().expect("Thread must exit cleanly");

    let elapsed = start.elapsed();
    // Must have held key for at least 200ms (override), not 50ms (default)
    assert!(
        elapsed >= Duration::from_millis(200),
        "Per-key dwell override (200ms) must be honored, took {:?}",
        elapsed
    );
}

#[test]
#[ignore = "requires /dev/uinput — set RUN_PRIVILEGED_TESTS=1"]
fn invalid_key_name_is_skipped_not_panicked() {
    if !should_run_privileged() {
        return;
    }

    let device = open_uinput_device().expect("Must open with input group access");
    let (tx, rx) = mpsc::channel();
    let handle = spawn_injection_thread(device, rx);

    // Mix of valid and invalid key names — invalid must be skipped, not panic
    tx.send(MacroCmd::Execute {
        keys: vec![
            KeyStep { key_name: "KEY_UP".to_string(), dwell_ms: None, gap_ms: None },
            KeyStep { key_name: "INVALID_KEY_XYZ".to_string(), dwell_ms: None, gap_ms: None },
            KeyStep { key_name: "KEY_DOWN".to_string(), dwell_ms: None, gap_ms: None },
        ],
        default_dwell_ms: 10,
        default_gap_ms: 5,
    })
    .expect("Channel must be open");

    tx.send(MacroCmd::Shutdown).expect("Shutdown must send");
    // Must join without panic — invalid key logged as error, not crash
    handle.join().expect("Thread must not panic on invalid key name");
}
