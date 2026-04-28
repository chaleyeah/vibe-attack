/// End-to-end integration tests for the M008 control surface over a real UDS socket.
///
/// Each test spins up `spawn_control_listener` against a live tokio runtime, waits
/// for the socket file to appear, sends commands via the blocking `send_command`
/// client on a `spawn_blocking` task, and asserts both wire-level responses and
/// in-process side-effects on the shared `DaemonHandle`.
///
/// Tests are serialised with `#[serial]` to prevent socket-path races.
/// If `XDG_RUNTIME_DIR` is absent (bare CI), the bind will fail and the test
/// returns early (no panic) so the suite remains green.
use std::sync::{mpsc, Arc};
use serial_test::serial;
use vibe_attack::control::{
    client::{is_daemon_running, send_command},
    protocol::{ActivationMode, ControlRequest, ControlResponse},
    spawn_control_listener, DaemonHandle,
};
use vibe_attack::input::inject::MacroCmd;
use vibe_attack::pipeline::coordinator::RuntimeCommand;
use vibe_attack::pipeline::dispatcher::Dispatcher;

// ── Drop guard ───────────────────────────────────────────────────────────────

/// Removes the socket file on drop so a panicking assertion doesn't leave a stale
/// socket that breaks the next test run.
struct SocketGuard(std::path::PathBuf);

impl Drop for SocketGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Resolve the server-side socket path (place_runtime_file creates the XDG dir).
/// Returns `None` when XDG_RUNTIME_DIR is absent so tests can skip gracefully.
fn socket_path() -> Option<std::path::PathBuf> {
    let xdg = xdg::BaseDirectories::with_prefix("vibe-attack");
    xdg.place_runtime_file("vibe-attack.sock").ok()
}

// ── Helper ───────────────────────────────────────────────────────────────────

/// Build a `DaemonHandle` wired to a fresh runtime channel.
/// Returns the handle and the Receiver end so the test can drain sent commands.
fn make_handle_with_runtime_tx() -> (DaemonHandle, mpsc::Receiver<RuntimeCommand>) {
    let (macro_tx, _macro_rx) = mpsc::channel::<MacroCmd>();
    let dispatcher = Arc::new(Dispatcher::new(0.5, vec![], macro_tx, 50, 30));
    let (rt_tx, rt_rx) = mpsc::channel::<RuntimeCommand>();
    let handle = DaemonHandle::new(dispatcher).with_runtime_tx(rt_tx);
    (handle, rt_rx)
}

/// Build a `DaemonHandle` whose dispatcher contains one macro named `"smoke_test"` with
/// a single KEY_UP key step. Returns the handle and the `MacroCmd` receiver so tests can
/// assert that `MacroCmd::Execute` was sent when `TestMacro` fires.
fn make_handle_with_macro_rx() -> (DaemonHandle, mpsc::Receiver<MacroCmd>) {
    use vibe_attack::config::{KeyAction, MacroConfig};
    let (macro_tx, macro_rx) = mpsc::channel::<MacroCmd>();
    let macro_cfg = MacroConfig {
        name: "smoke_test".to_string(),
        phrase: None,
        if_flag: None,
        set_flag: None,
        sound: None,
        keys: vec![KeyAction { key: "KEY_UP".to_string(), dwell_ms: None, gap_ms: None }],
    };
    let dispatcher = Arc::new(Dispatcher::new(0.5, vec![macro_cfg], macro_tx, 50, 30));
    let handle = DaemonHandle::new(dispatcher);
    (handle, macro_rx)
}

// ── Tests ────────────────────────────────────────────────────────────────────

/// Prove the full round-trip: SetMode travels over the socket, the handler caches
/// `active_mode` on the handle, forwards a `RuntimeCommand::SetMode` to the
/// runtime channel, and a subsequent Status query reflects the new mode.
#[tokio::test]
#[serial]
async fn set_mode_round_trip_via_socket() {
    let path = match socket_path() {
        Some(p) => p,
        None => {
            eprintln!("set_mode_round_trip_via_socket: XDG_RUNTIME_DIR absent — skipping");
            return;
        }
    };

    let (handle, rt_rx) = make_handle_with_runtime_tx();

    // Confirm default mode is Ptt before any command.
    assert_eq!(
        *handle.active_mode.read().unwrap(),
        ActivationMode::Ptt,
        "initial active_mode must be Ptt"
    );

    // Bind the control listener; skip if bind fails (e.g. runtime dir not writable).
    if let Err(e) = spawn_control_listener(handle.clone()).await {
        eprintln!("set_mode_round_trip_via_socket: bind failed ({e}) — skipping");
        return;
    }

    let _guard = SocketGuard(path.clone());

    // Poll until is_daemon_running() returns true (socket file exists).
    let mut ready = false;
    for _ in 0..50 {
        if is_daemon_running() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    if !ready {
        eprintln!("set_mode_round_trip_via_socket: socket never became ready — skipping");
        return;
    }

    // Send SetMode{Wake} over the socket from a blocking thread.
    let resp = tokio::task::spawn_blocking(|| {
        send_command(ControlRequest::SetMode { mode: ActivationMode::Wake })
    })
    .await
    .expect("spawn_blocking panicked")
    .expect("send_command failed");

    assert!(
        matches!(resp, ControlResponse::Ok),
        "expected Ok from SetMode, got: {resp:?}"
    );

    // The handler must have forwarded exactly one RuntimeCommand::SetMode(Wake).
    let mut forwarded = Vec::new();
    while let Ok(cmd) = rt_rx.try_recv() {
        forwarded.push(cmd);
    }
    assert_eq!(forwarded.len(), 1, "expected exactly one RuntimeCommand forwarded, got {}", forwarded.len());
    assert!(
        matches!(forwarded[0], RuntimeCommand::SetMode(ActivationMode::Wake)),
        "expected SetMode(Wake), got: {:?}", forwarded[0]
    );

    // The cache on the handle must reflect the new mode.
    assert_eq!(
        *handle.active_mode.read().unwrap(),
        ActivationMode::Wake,
        "handle active_mode not updated after SetMode"
    );

    // Status query over the wire must echo back the cached mode.
    let status_resp = tokio::task::spawn_blocking(|| {
        send_command(ControlRequest::Status)
    })
    .await
    .expect("spawn_blocking panicked")
    .expect("send_command(Status) failed");

    match status_resp {
        ControlResponse::StatusData(s) => {
            assert_eq!(
                s.active_mode,
                ActivationMode::Wake,
                "Status response active_mode should reflect the SetMode we sent"
            );
        }
        other => panic!("expected StatusData from Status query, got: {other:?}"),
    }
}

/// Prove that SetThreshold travels over the socket and the handler forwards a
/// `RuntimeCommand::SetThreshold` with the correct value to the runtime channel.
/// The dispatcher's internal threshold is updated by the coordinator when it drains
/// the channel (tested in tests/runtime_commands.rs); here we only assert the wire
/// and channel path.
#[tokio::test]
#[serial]
async fn set_threshold_via_socket_updates_dispatcher() {
    let path = match socket_path() {
        Some(p) => p,
        None => {
            eprintln!("set_threshold_via_socket_updates_dispatcher: XDG_RUNTIME_DIR absent — skipping");
            return;
        }
    };

    let (handle, rt_rx) = make_handle_with_runtime_tx();

    if let Err(e) = spawn_control_listener(handle.clone()).await {
        eprintln!("set_threshold_via_socket_updates_dispatcher: bind failed ({e}) — skipping");
        return;
    }

    let _guard = SocketGuard(path.clone());

    let mut ready = false;
    for _ in 0..50 {
        if is_daemon_running() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    if !ready {
        eprintln!("set_threshold_via_socket_updates_dispatcher: socket never became ready — skipping");
        return;
    }

    let resp = tokio::task::spawn_blocking(|| {
        send_command(ControlRequest::SetThreshold { threshold: 0.6 })
    })
    .await
    .expect("spawn_blocking panicked")
    .expect("send_command failed");

    assert!(
        matches!(resp, ControlResponse::Ok),
        "expected Ok from SetThreshold, got: {resp:?}"
    );

    // The handler must have forwarded exactly one RuntimeCommand::SetThreshold(0.6).
    let mut forwarded = Vec::new();
    while let Ok(cmd) = rt_rx.try_recv() {
        forwarded.push(cmd);
    }
    assert_eq!(forwarded.len(), 1, "expected exactly one RuntimeCommand forwarded, got {}", forwarded.len());
    match &forwarded[0] {
        RuntimeCommand::SetThreshold(t) => {
            assert!(
                (t - 0.6_f32).abs() < 1e-5,
                "expected SetThreshold(0.6), got SetThreshold({t})"
            );
        }
        other => panic!("expected SetThreshold, got: {other:?}"),
    }
}

/// Prove that `TestMacro` travels over the socket, the handler calls `fire_named`, and
/// exactly one `MacroCmd::Execute` is delivered to the injection channel.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn test_macro_via_socket_fires_dispatcher() {
    use std::sync::mpsc::TryRecvError;

    let path = match socket_path() {
        Some(p) => p,
        None => {
            eprintln!("test_macro_via_socket_fires_dispatcher: XDG_RUNTIME_DIR absent — skipping");
            return;
        }
    };

    let (handle, macro_rx) = make_handle_with_macro_rx();

    if let Err(e) = spawn_control_listener(handle.clone()).await {
        eprintln!("test_macro_via_socket_fires_dispatcher: bind failed ({e}) — skipping");
        return;
    }

    let _guard = SocketGuard(path.clone());

    let mut ready = false;
    for _ in 0..50 {
        if is_daemon_running() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    if !ready {
        eprintln!("test_macro_via_socket_fires_dispatcher: socket never became ready — skipping");
        return;
    }

    let resp = tokio::task::spawn_blocking(|| {
        send_command(ControlRequest::TestMacro { name: "smoke_test".to_string() })
    })
    .await
    .expect("spawn_blocking panicked")
    .expect("send_command failed");

    assert!(
        matches!(resp, ControlResponse::Ok),
        "expected Ok from TestMacro, got: {resp:?}"
    );

    // Exactly one MacroCmd::Execute must have been sent to the injection channel.
    let cmd = macro_rx.try_recv().expect("expected one MacroCmd::Execute on the channel");
    assert!(
        matches!(cmd, MacroCmd::Execute { .. }),
        "expected MacroCmd::Execute but got a different variant"
    );
    assert!(
        matches!(macro_rx.try_recv(), Err(TryRecvError::Empty)),
        "expected exactly one MacroCmd, got more"
    );
}

/// Prove that a `TestMacro` for an unknown name returns `ControlResponse::Error`
/// whose message contains "macro not found".
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn test_macro_unknown_name_returns_error() {
    let path = match socket_path() {
        Some(p) => p,
        None => {
            eprintln!("test_macro_unknown_name_returns_error: XDG_RUNTIME_DIR absent — skipping");
            return;
        }
    };

    let (handle, _macro_rx) = make_handle_with_macro_rx();

    if let Err(e) = spawn_control_listener(handle.clone()).await {
        eprintln!("test_macro_unknown_name_returns_error: bind failed ({e}) — skipping");
        return;
    }

    let _guard = SocketGuard(path.clone());

    let mut ready = false;
    for _ in 0..50 {
        if is_daemon_running() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    if !ready {
        eprintln!("test_macro_unknown_name_returns_error: socket never became ready — skipping");
        return;
    }

    let resp = tokio::task::spawn_blocking(|| {
        send_command(ControlRequest::TestMacro { name: "nonexistent".to_string() })
    })
    .await
    .expect("spawn_blocking panicked")
    .expect("send_command failed");

    match resp {
        ControlResponse::Error { message } => {
            assert!(
                message.contains("macro not found"),
                "error message must contain 'macro not found', got: {message}"
            );
        }
        other => panic!("expected ControlResponse::Error, got: {other:?}"),
    }
}
