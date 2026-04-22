//! VirtualDevice creation smoke test (MCRO-05).
//! Requires /dev/uinput: run with RUN_PRIVILEGED_TESTS=1.

#[test]
#[ignore = "requires /dev/uinput — set RUN_PRIVILEGED_TESTS=1"]
fn virtual_keyboard_opens_stub() {
    // TODO: implemented when VirtualDeviceBuilder logic exists (Plan 04)
}
