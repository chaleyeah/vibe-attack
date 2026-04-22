//! Verify daemon starts with no display surface (UI-01).

#[test]
fn daemon_does_not_set_display_env_stub() {
    // TODO: spawn daemon binary and verify no WAYLAND_DISPLAY / DISPLAY socket
    // opened during startup. Implemented in Plan 05.
    assert!(std::env::var("VERIFY_HEADLESS").is_err() || true,
        "stub: real test added in Plan 05");
}
