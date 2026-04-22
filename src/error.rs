//! Typed error variants for hd-linux-voice daemon.

/// All catastrophic daemon errors. These result in process exit with an actionable message.
#[derive(Debug, thiserror::Error)]
pub enum DaemonError {
    /// /dev/uinput is not accessible — fail-hard per D-15.
    /// The Display message is printed directly to stderr; it must be copy-pasteable.
    #[error(
        "Error: cannot open /dev/uinput — permission denied.\n\
         Fix: sudo modprobe uinput && sudo usermod -aG input $USER\n\
         Then log out and back in, or run: newgrp input\n\
         Note: On systemd v258+, use the 'input' group (not 'uinput').\n\
         Docs: https://github.com/yourusername/hd-linux-voice/blob/main/docs/uinput-setup.md"
    )]
    UinputPermissionDenied,

    /// A /dev/input/event* node could not be read — likely not in 'input' group.
    #[error(
        "Error: cannot read /dev/input — permission denied.\n\
         Fix: sudo usermod -aG input $USER\n\
         Then log out and back in, or run: newgrp input"
    )]
    InputGroupMissing,

    /// PTT key device not found after scanning all /dev/input/event* nodes.
    #[error("No input device found that reports key '{0}'.\nRun `evtest` to list devices.")]
    NoPttDevice(String),

    /// Config file not found or could not be parsed.
    #[error("Config error: {0}")]
    Config(String),
}
