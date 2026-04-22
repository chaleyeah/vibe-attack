//! Typed error variants — implemented in Phase 01 Plan 04.

#[derive(Debug, thiserror::Error)]
pub enum DaemonError {
    #[error("uinput permission denied — see docs for fix")]
    UinputPermissionDenied,

    #[error("no PTT device found: {0}")]
    NoPttDevice(String),

    #[error("config error: {0}")]
    Config(String),
}
