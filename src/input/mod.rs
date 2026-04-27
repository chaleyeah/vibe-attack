//! Input subsystem (PTT detection + uinput injection).

/// uinput virtual keyboard injection — `MacroCmd` channel, `KeyStep`, and the injection thread.
pub mod inject;
/// PTT (push-to-talk) evdev key detection and `KeyCode` parsing.
pub mod ptt;
