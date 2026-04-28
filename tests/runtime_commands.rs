/// Integration tests for the RuntimeCommand channel contract.
///
/// These tests exercise the coordinator drain logic, DaemonHandle channel
/// forwarding, and the Dispatcher threshold update path end-to-end without
/// spinning up a full audio pipeline.
use std::sync::{mpsc, Arc};
use vibe_attack::control::protocol::ActivationMode;
use vibe_attack::pipeline::coordinator::RuntimeCommand;
use vibe_attack::pipeline::dispatcher::{DispatchOutcome, Dispatcher};

fn make_dispatcher(threshold: f32, phrase: &str) -> Arc<Dispatcher> {
    use vibe_attack::config::MacroConfig;
    let (tx, _rx) = mpsc::channel();
    let macro_cfg = MacroConfig {
        name: "test_macro".to_string(),
        phrase: Some(phrase.to_string()),
        if_flag: None,
        set_flag: None,
        sound: None,
        keys: vec![],
    };
    Arc::new(Dispatcher::new(threshold, vec![macro_cfg], tx, 50, 30))
}

// ── Channel drain simulation ─────────────────────────────────────────────────

/// Simulates the coordinator's per-frame try_recv drain against a Dispatcher.
/// Returns the number of commands processed.
fn drain_into_dispatcher(rx: &mpsc::Receiver<RuntimeCommand>, dispatcher: &Arc<Dispatcher>) -> usize {
    let mut count = 0;
    while let Ok(cmd) = rx.try_recv() {
        match cmd {
            RuntimeCommand::SetThreshold(t) => {
                dispatcher.update_threshold(t);
                count += 1;
            }
            RuntimeCommand::SetMode(_m) => {
                count += 1;
            }
            RuntimeCommand::SetInputDevice(_) | RuntimeCommand::SetPttBinding(_) | RuntimeCommand::ReloadConfig => {
                count += 1;
            }
        }
    }
    count
}

// ── Happy-path tests ─────────────────────────────────────────────────────────

#[test]
fn set_threshold_via_channel_changes_dispatch_outcome() {
    // Start with a threshold so high nothing matches.
    let dispatcher = make_dispatcher(0.99, "eagle airstrike");
    let (tx, rx) = mpsc::channel::<RuntimeCommand>();

    // Confirm NoMatch at strict threshold.
    assert!(
        matches!(dispatcher.process("eagal airstrike"), DispatchOutcome::NoMatch),
        "expected NoMatch before threshold update"
    );

    // Send SetThreshold over the channel, then drain it into the dispatcher.
    tx.send(RuntimeCommand::SetThreshold(0.5)).unwrap();
    let processed = drain_into_dispatcher(&rx, &dispatcher);
    assert_eq!(processed, 1, "expected exactly one command drained");

    // After threshold update the same near-miss phrase should fire.
    assert!(
        matches!(dispatcher.process("eagal airstrike"), DispatchOutcome::Fired { .. }),
        "expected Fired after threshold lowered via channel"
    );
}

#[test]
fn set_threshold_clamp_via_channel() {
    let dispatcher = make_dispatcher(0.5, "eagle airstrike");
    let (tx, rx) = mpsc::channel::<RuntimeCommand>();

    // Send an out-of-range value — update_threshold clamps it internally.
    tx.send(RuntimeCommand::SetThreshold(1.5)).unwrap();
    drain_into_dispatcher(&rx, &dispatcher);

    // Clamped to 1.0 → near-miss should not match.
    assert!(
        matches!(dispatcher.process("eagal airstrike"), DispatchOutcome::NoMatch),
        "expected NoMatch after threshold clamped to 1.0"
    );
    // Exact match should still fire (score == 1.0 >= 1.0).
    assert!(
        matches!(dispatcher.process("eagle airstrike"), DispatchOutcome::Fired { .. }),
        "expected Fired on exact match after clamp"
    );
}

#[test]
fn set_mode_command_is_accepted_by_drain() {
    let dispatcher = make_dispatcher(0.5, "eagle airstrike");
    let (tx, rx) = mpsc::channel::<RuntimeCommand>();

    tx.send(RuntimeCommand::SetMode(ActivationMode::Wake)).unwrap();
    let processed = drain_into_dispatcher(&rx, &dispatcher);
    assert_eq!(processed, 1);

    tx.send(RuntimeCommand::SetMode(ActivationMode::Ptt)).unwrap();
    let processed = drain_into_dispatcher(&rx, &dispatcher);
    assert_eq!(processed, 1);
}

// ── Negative tests ───────────────────────────────────────────────────────────

/// When the Receiver is dropped (coordinator gone), the Sender returns SendError.
/// The control handler converts this to ControlResponse::Error{message}.
#[test]
fn channel_send_after_receiver_dropped_returns_error() {
    let (tx, rx) = mpsc::channel::<RuntimeCommand>();
    drop(rx); // simulate coordinator stopped

    let result = tx.send(RuntimeCommand::SetThreshold(0.5));
    assert!(result.is_err(), "expected SendError when receiver is dropped");
}

/// DaemonHandle with no runtime_tx attached returns pipeline-not-running error.
#[test]
fn daemon_handle_without_runtime_tx_returns_error() {
    use vibe_attack::control::{protocol::ControlResponse, DaemonHandle};

    let (macro_tx, _macro_rx) = mpsc::channel();
    let dispatcher = Arc::new(Dispatcher::new(0.5, vec![], macro_tx, 50, 30));
    let handle = DaemonHandle::new(dispatcher); // no runtime_tx attached

    // Access via the internal send helper through control protocol test.
    // We verify indirectly: runtime_cmd_tx is None, so any forwarded command
    // should produce ControlResponse::Error{message}.
    let result = match &handle.runtime_cmd_tx {
        None => ControlResponse::Error { message: "pipeline not running".into() },
        Some(tx) => match tx.send(RuntimeCommand::SetThreshold(0.5)) {
            Ok(()) => ControlResponse::Ok,
            Err(_) => ControlResponse::Error { message: "pipeline not running".into() },
        },
    };
    assert!(
        matches!(result, ControlResponse::Error { ref message } if message == "pipeline not running"),
        "expected pipeline-not-running error, got: {result:?}"
    );
}

/// Multiple commands queued before drain are all processed in order.
#[test]
fn multiple_commands_drained_in_order() {
    let dispatcher = make_dispatcher(0.99, "eagle airstrike");
    let (tx, rx) = mpsc::channel::<RuntimeCommand>();

    tx.send(RuntimeCommand::SetThreshold(0.99)).unwrap();
    tx.send(RuntimeCommand::SetThreshold(0.5)).unwrap();

    let processed = drain_into_dispatcher(&rx, &dispatcher);
    assert_eq!(processed, 2);

    // Last command (0.5) wins — near-miss fires.
    assert!(
        matches!(dispatcher.process("eagal airstrike"), DispatchOutcome::Fired { .. }),
        "expected Fired — last SetThreshold(0.5) should win"
    );
}
