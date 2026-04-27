//! Typed error variants for vibe-attack daemon.

/// Catastrophic daemon errors that cause process exit with an actionable stderr message.
///
/// Every variant carries a human-readable [`Display`] message (via [`thiserror`]) that
/// tells the user exactly what went wrong and how to fix it. Callers should propagate
/// these to `main` and print them; do not swallow them silently.
#[derive(Debug, thiserror::Error)]
pub enum DaemonError {
    /// `/dev/uinput` could not be opened due to a permission error.
    ///
    /// **Condition:** Produced by `src/input/inject.rs` (`open_uinput`) when the kernel
    /// returns `EACCES` on `/dev/uinput`. This happens when the running user is not a
    /// member of the `input` group (or `uinput` on older kernels) and `uinput` module
    /// is not loaded.
    ///
    /// **Recovery:** Load the module with `sudo modprobe uinput`, add the user to the
    /// `input` group (`sudo usermod -aG input $USER`), then log out and back in (or run
    /// `newgrp input` to avoid a full re-login). See `docs/uinput-setup.md` for details.
    #[error(
        "Error: cannot open /dev/uinput — permission denied.\n\
         Fix: sudo modprobe uinput && sudo usermod -aG input $USER\n\
         Then log out and back in, or run: newgrp input\n\
         Note: On systemd v258+, use the 'input' group (not 'uinput').\n\
         Docs: https://github.com/yourusername/vibe-attack/blob/main/docs/uinput-setup.md"
    )]
    UinputPermissionDenied,

    /// A `/dev/input/event*` node could not be read due to a permission error.
    ///
    /// **Condition:** Produced by `src/input/scan.rs` when opening an evdev event node
    /// returns `EACCES`. This is distinct from [`UinputPermissionDenied`]: it affects
    /// *reading* raw key events (PTT detection) rather than *injecting* synthetic events.
    ///
    /// **Recovery:** Add the user to the `input` group (`sudo usermod -aG input $USER`)
    /// and re-login. No module load is required — the group membership alone is sufficient.
    ///
    /// [`UinputPermissionDenied`]: DaemonError::UinputPermissionDenied
    #[error(
        "Error: cannot read /dev/input — permission denied.\n\
         Fix: sudo usermod -aG input $USER\n\
         Then log out and back in, or run: newgrp input"
    )]
    InputGroupMissing,

    /// No evdev device reports the configured PTT key.
    ///
    /// **Condition:** Produced by `src/input/scan.rs` after exhausting all
    /// `/dev/input/event*` nodes without finding one that advertises the key code named
    /// in the config (`ptt.key`). The inner `String` is the key name from config (e.g.
    /// `"KEY_F13"`).
    ///
    /// **Recovery:** Run `evtest` (package: `evtest`) to list connected devices and the
    /// key codes they report. Update `ptt.key` in the config to a code that actually
    /// appears on a connected device.
    #[error("No input device found that reports key '{0}'.\nRun `evtest` to list devices.")]
    NoPttDevice(String),

    /// The daemon configuration could not be loaded or validated.
    ///
    /// **Condition:** Reserved for config-layer failures — either the YAML file cannot be
    /// parsed (malformed YAML, unknown fields, wrong value types) or post-parse validation
    /// fails (e.g. a referenced model file does not exist on disk). The inner `String`
    /// carries the human-readable reason. Currently unused in production paths (config
    /// errors propagate as `anyhow::Error`); reserved for future tightening of the config
    /// error type.
    ///
    /// **Recovery:** Fix the error described in the message. Common causes: a typo in a
    /// field name, a path to an ONNX model file that does not exist, or a required field
    /// (`ptt`, `ptt.key`) left blank. See `docs/configuration.md` for the full schema.
    #[error("Config error: {0}")]
    Config(String),
}
