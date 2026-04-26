/// Hermetic tests for the control protocol — no daemon process required.
///
/// Tests cover:
///   - ControlRequest JSON serialization (all new variants)
///   - ControlResponse deserialization (StatusData)
///   - DaemonHandle state machine (Idle, Muted, Listening, Recording priority)
///   - DaemonHandle status snapshot (active_profile, macro_count)
use std::sync::Arc;
use vibe_attack::control::protocol::{
    ControlRequest, ControlResponse, DaemonState, DaemonStatus,
};

// ── Serialization round-trips ────────────────────────────────────────────────

#[test]
fn status_request_roundtrip() {
    let req = ControlRequest::Status;
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::Status));
}

#[test]
fn mute_request_roundtrip() {
    let req = ControlRequest::Mute;
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::Mute));
}

#[test]
fn unmute_request_roundtrip() {
    let req = ControlRequest::Unmute;
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::Unmute));
}

#[test]
fn status_data_response_roundtrip() {
    let status = DaemonStatus {
        state: DaemonState::Idle,
        active_profile: Some("hd2".to_string()),
        macro_count: 42,
    };
    let resp = ControlResponse::StatusData(status);
    let json = serde_json::to_string(&resp).unwrap();
    let back: ControlResponse = serde_json::from_str(&json).unwrap();
    match back {
        ControlResponse::StatusData(s) => {
            assert_eq!(s.state, DaemonState::Idle);
            assert_eq!(s.active_profile.as_deref(), Some("hd2"));
            assert_eq!(s.macro_count, 42);
        }
        other => panic!("expected StatusData, got {other:?}"),
    }
}

#[test]
fn daemon_state_serde_snake_case() {
    // State variants must serialize as snake_case for the tray to parse them.
    let muted = serde_json::to_string(&DaemonState::Muted).unwrap();
    assert_eq!(muted, r#""muted""#);
    let listening = serde_json::to_string(&DaemonState::Listening).unwrap();
    assert_eq!(listening, r#""listening""#);
    let recording = serde_json::to_string(&DaemonState::Recording).unwrap();
    assert_eq!(recording, r#""recording""#);
    let idle = serde_json::to_string(&DaemonState::Idle).unwrap();
    assert_eq!(idle, r#""idle""#);
}

// ── DaemonHandle state machine ───────────────────────────────────────────────

fn make_handle() -> vibe_attack::control::DaemonHandle {
    use vibe_attack::pipeline::dispatcher::Dispatcher;
    use std::sync::mpsc;

    let (tx, _rx) = mpsc::channel();
    let dispatcher = Arc::new(Dispatcher::new(0.5, vec![], tx, 50, 30));
    vibe_attack::control::DaemonHandle::new(dispatcher)
}

#[test]
fn state_idle_by_default() {
    let h = make_handle();
    assert_eq!(h.state(), DaemonState::Idle);
}

#[test]
fn state_muted_when_muted_flag_set() {
    let h = make_handle();
    h.muted.store(true, std::sync::atomic::Ordering::Relaxed);
    assert_eq!(h.state(), DaemonState::Muted);
}

#[test]
fn muted_takes_priority_over_listening() {
    let h = make_handle();
    h.muted.store(true, std::sync::atomic::Ordering::Relaxed);
    h.listening.store(true, std::sync::atomic::Ordering::Relaxed);
    assert_eq!(h.state(), DaemonState::Muted);
}

#[test]
fn state_recording_when_recording_flag_set() {
    let h = make_handle();
    h.recording.store(true, std::sync::atomic::Ordering::Relaxed);
    assert_eq!(h.state(), DaemonState::Recording);
}

#[test]
fn state_listening_when_listening_flag_set() {
    let h = make_handle();
    h.listening.store(true, std::sync::atomic::Ordering::Relaxed);
    assert_eq!(h.state(), DaemonState::Listening);
}

#[test]
fn status_reflects_active_profile_and_macro_count() {
    use vibe_attack::config::MacroConfig;
    use vibe_attack::pipeline::dispatcher::Dispatcher;
    use std::sync::mpsc;

    let (tx, _rx) = mpsc::channel();
    let macros: Vec<MacroConfig> = (0..5)
        .map(|i| MacroConfig {
            name: format!("macro_{i}"),
            phrase: Some(format!("phrase {i}")),
            keys: vec![],
            sound: None,
            if_flag: None,
            set_flag: None,
        })
        .collect();
    let dispatcher = Arc::new(Dispatcher::new(0.5, macros, tx, 50, 30));
    let h = vibe_attack::control::DaemonHandle::new(dispatcher);

    *h.active_profile.write().unwrap() = Some("hd2".to_string());

    let status = h.status();
    assert_eq!(status.macro_count, 5);
    assert_eq!(status.active_profile.as_deref(), Some("hd2"));
    assert_eq!(status.state, DaemonState::Idle);
}
