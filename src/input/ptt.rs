//! evdev PTT key detection (ACT-01, D-09, D-10, D-11, D-12).
//! RED phase: tests defined first, implementation pending.

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
        // parse_key_code not yet defined → compile error (RED)
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
        match check_input_readable() {
            Ok(()) => {}
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
        let event = InputEvent::new(EventType::KEY.0, KeyCode::KEY_SPACE.0, 1);
        process_event(event, KeyCode::KEY_F13, &ptt);
        assert!(!ptt.load(Ordering::Relaxed), "Different key must not change ptt_active");
    }
}
