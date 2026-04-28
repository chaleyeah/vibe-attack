/// Hermetic tests for the control protocol — no daemon process required.
///
/// Tests cover:
///   - ControlRequest JSON serialization (all new variants)
///   - ControlResponse deserialization (StatusData)
///   - DaemonHandle state machine (Idle, Muted, Listening, Recording priority)
///   - DaemonHandle status snapshot (active_profile, macro_count)
use std::sync::Arc;
use vibe_attack::control::protocol::{
    ActivationMode, ControlRequest, ControlResponse, DaemonState, DaemonStatus,
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
        active_mode: ActivationMode::Wake,
    };
    let resp = ControlResponse::StatusData(status);
    let json = serde_json::to_string(&resp).unwrap();
    let back: ControlResponse = serde_json::from_str(&json).unwrap();
    match back {
        ControlResponse::StatusData(s) => {
            assert_eq!(s.state, DaemonState::Idle);
            assert_eq!(s.active_profile.as_deref(), Some("hd2"));
            assert_eq!(s.macro_count, 42);
            assert_eq!(s.active_mode, ActivationMode::Wake);
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

// ── New control variants: round-trip serde ───────────────────────────────────

#[test]
fn set_mode_roundtrip() {
    let req = ControlRequest::SetMode { mode: ActivationMode::Wake };
    let json = serde_json::to_string(&req).unwrap();
    // adjacently tagged: {"cmd":"set_mode","args":{"mode":"wake"}}
    assert!(json.contains("\"set_mode\""), "cmd tag missing: {json}");
    assert!(json.contains("\"wake\""), "mode value missing: {json}");
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::SetMode { mode: ActivationMode::Wake }));

    // also round-trip Ptt variant
    let req2 = ControlRequest::SetMode { mode: ActivationMode::Ptt };
    let json2 = serde_json::to_string(&req2).unwrap();
    let back2: ControlRequest = serde_json::from_str(&json2).unwrap();
    assert!(matches!(back2, ControlRequest::SetMode { mode: ActivationMode::Ptt }));
}

#[test]
fn set_threshold_roundtrip() {
    let req = ControlRequest::SetThreshold { threshold: 0.75 };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("\"set_threshold\""), "cmd tag missing: {json}");
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    match back {
        ControlRequest::SetThreshold { threshold } => {
            assert!((threshold - 0.75_f32).abs() < 1e-6, "threshold mismatch: {threshold}");
        }
        other => panic!("expected SetThreshold, got {other:?}"),
    }
}

#[test]
fn set_input_device_roundtrip() {
    // Some(device)
    let req = ControlRequest::SetInputDevice { device: Some("hw:1,0".to_string()) };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("\"set_input_device\""), "cmd tag missing: {json}");
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::SetInputDevice { device: Some(ref d) } if d == "hw:1,0"));

    // None variant
    let req_none = ControlRequest::SetInputDevice { device: None };
    let json_none = serde_json::to_string(&req_none).unwrap();
    let back_none: ControlRequest = serde_json::from_str(&json_none).unwrap();
    assert!(matches!(back_none, ControlRequest::SetInputDevice { device: None }));
}

#[test]
fn set_ptt_binding_roundtrip() {
    let req = ControlRequest::SetPttBinding { key: "ctrl+shift+v".to_string() };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("\"set_ptt_binding\""), "cmd tag missing: {json}");
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::SetPttBinding { ref key } if key == "ctrl+shift+v"));
}

#[test]
fn reload_config_roundtrip() {
    let req = ControlRequest::ReloadConfig;
    let json = serde_json::to_string(&req).unwrap();
    // Unit variant with adjacently tagged enum: {"cmd":"reload_config"} — no "args" key
    assert_eq!(json, r#"{"cmd":"reload_config"}"#, "unexpected JSON: {json}");
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::ReloadConfig));
}

// ── Negative tests ───────────────────────────────────────────────────────────

#[test]
fn set_mode_bogus_activation_mode_is_error() {
    let result = serde_json::from_str::<ControlRequest>(
        r#"{"cmd":"set_mode","args":{"mode":"bogus"}}"#,
    );
    assert!(result.is_err(), "expected Err for unknown ActivationMode, got: {result:?}");
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

#[test]
fn daemon_handle_active_mode_defaults_to_ptt() {
    let h = make_handle();
    assert_eq!(*h.active_mode.read().unwrap(), ActivationMode::Ptt);
    assert_eq!(h.status().active_mode, ActivationMode::Ptt);
}

#[test]
fn daemon_handle_active_mode_updates_on_write() {
    let h = make_handle();
    *h.active_mode.write().unwrap() = ActivationMode::Wake;
    assert_eq!(*h.active_mode.read().unwrap(), ActivationMode::Wake);
    assert_eq!(h.status().active_mode, ActivationMode::Wake);
}

#[test]
fn status_active_mode_serializes_snake_case() {
    // active_mode must round-trip through JSON (tray parses Status responses).
    let status = DaemonStatus {
        state: DaemonState::Idle,
        active_profile: None,
        macro_count: 0,
        active_mode: ActivationMode::Ptt,
    };
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("\"ptt\""), "expected ptt in JSON: {json}");
    let back: DaemonStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(back.active_mode, ActivationMode::Ptt);
}

#[test]
fn daemon_status_backward_compat_no_active_mode_field() {
    // Old JSON without active_mode should deserialize to Ptt via #[serde(default)].
    let old_json = r#"{"state":"idle","active_profile":null,"macro_count":0}"#;
    let status: DaemonStatus = serde_json::from_str(old_json).unwrap();
    assert_eq!(status.active_mode, ActivationMode::Ptt);
}
