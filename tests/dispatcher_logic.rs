use hd_linux_voice::config::{MacroConfig, KeyAction};
use hd_linux_voice::pipeline::dispatcher::Dispatcher;
use hd_linux_voice::input::inject::MacroCmd;
use std::sync::mpsc::channel;
use evdev::KeyCode;

#[test]
fn test_dispatcher_conditional_reuse() {
    let (tx, rx) = channel();
    
    let macros = vec![
        // Macro to set mode_b
        MacroConfig {
            name: "set_mode_b".to_string(),
            phrase: Some("enable mode b".to_string()),
            if_flag: None,
            set_flag: Some("mode_b".to_string()),
            sound: None,
            keys: vec![],
        },
        // Macro A: only if mode_a is true
        MacroConfig {
            name: "macro_a".to_string(),
            phrase: Some("deploy".to_string()),
            if_flag: Some("mode_a".to_string()),
            set_flag: None,
            sound: None,
            keys: vec![KeyAction { key: "KEY_A".to_string(), dwell_ms: None, gap_ms: None }],
        },
        // Macro B: only if mode_b is true
        MacroConfig {
            name: "macro_b".to_string(),
            phrase: Some("deploy".to_string()),
            if_flag: Some("mode_b".to_string()),
            set_flag: None,
            sound: None,
            keys: vec![KeyAction { key: "KEY_B".to_string(), dwell_ms: None, gap_ms: None }],
        },
    ];

    let dispatcher = Dispatcher::new(0.8, macros, tx, 50, 30);
    
    // 1. Initially both mode_a and mode_b are false. "deploy" should match nothing.
    dispatcher.process("deploy");
    assert!(rx.try_recv().is_err(), "Should not have fired any macro");

    // 2. Set mode_b = true
    dispatcher.process("enable mode b");
    let cmd = rx.recv().expect("Should have fired set_mode_b");
    if let MacroCmd::Execute { keys, .. } = cmd {
        assert!(keys.is_empty());
    }

    // 3. Now "deploy" should fire macro_b
    dispatcher.process("deploy");
    
    let cmd = rx.try_recv().expect("Should have fired macro_b");
    if let MacroCmd::Execute { keys, .. } = cmd {
        assert_eq!(keys.len(), 1);
        let expected_key = "KEY_B".parse::<KeyCode>().unwrap();
        let actual_key = keys[0].key_name.parse::<KeyCode>().unwrap();
        assert_eq!(actual_key, expected_key);
    }
}

#[test]
fn test_dispatcher_negated_condition() {
    let (tx, rx) = channel();
    
    let macros = vec![
        MacroConfig {
            name: "macro_not_active".to_string(),
            phrase: Some("status".to_string()),
            if_flag: Some("!active".to_string()),
            set_flag: None,
            sound: None,
            keys: vec![KeyAction { key: "KEY_N".to_string(), dwell_ms: None, gap_ms: None }],
        },
        MacroConfig {
            name: "macro_active".to_string(),
            phrase: Some("status".to_string()),
            if_flag: Some("active".to_string()),
            set_flag: None,
            sound: None,
            keys: vec![KeyAction { key: "KEY_A".to_string(), dwell_ms: None, gap_ms: None }],
        },
        MacroConfig {
            name: "set_active".to_string(),
            phrase: Some("activate".to_string()),
            if_flag: None,
            set_flag: Some("active".to_string()),
            sound: None,
            keys: vec![],
        },
        MacroConfig {
            name: "unset_active".to_string(),
            phrase: Some("deactivate".to_string()),
            if_flag: None,
            set_flag: Some("!active".to_string()),
            sound: None,
            keys: vec![],
        },
    ];

    let dispatcher = Dispatcher::new(0.8, macros, tx, 50, 30);
    
    // 1. Initially active is false. "status" should fire macro_not_active
    dispatcher.process("status");
    let cmd = rx.recv().expect("Should have fired macro_not_active");
    if let MacroCmd::Execute { keys, .. } = cmd {
        assert_eq!(keys[0].key_name, "KEY_N");
    }

    // 2. Activate
    dispatcher.process("activate");
    let _ = rx.recv();

    // 3. "status" should fire macro_active
    dispatcher.process("status");
    let cmd = rx.recv().expect("Should have fired macro_active");
    if let MacroCmd::Execute { keys, .. } = cmd {
        assert_eq!(keys[0].key_name, "KEY_A");
    }

    // 4. Deactivate
    dispatcher.process("deactivate");
    let _ = rx.recv();

    // 5. "status" should fire macro_not_active again
    dispatcher.process("status");
    let cmd = rx.recv().expect("Should have fired macro_not_active");
    if let MacroCmd::Execute { keys, .. } = cmd {
        assert_eq!(keys[0].key_name, "KEY_N");
    }
}
