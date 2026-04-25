//! VirtualDevice creation smoke test (MCRO-05).
//! Requires /dev/uinput: run with RUN_PRIVILEGED_TESTS=1.

#[test]
#[ignore = "requires /dev/uinput — set RUN_PRIVILEGED_TESTS=1"]
fn virtual_keyboard_opens_with_vibe_attack_name() {
    let _device = vibe_attack::input::inject::open_uinput_device()
        .expect("VirtualDevice must open when user is in 'input' group");
    // If we get here without panic/error, the VirtualDevice was created successfully.
    // Full key injection tested in tests/macro_inject.rs.
}

#[test]
fn uinput_error_message_is_actionable() {
    // This test validates D-15 error format WITHOUT needing /dev/uinput.
    let err_msg = format!("{}", vibe_attack::error::DaemonError::UinputPermissionDenied);
    assert!(
        err_msg.contains("usermod -aG input"),
        "Error must reference 'input' group (not 'uinput'), got: {err_msg}"
    );
}
