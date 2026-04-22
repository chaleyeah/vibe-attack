//! Integration tests for key sequence injection (MCRO-01, MCRO-02).
//! Requires /dev/uinput: run with RUN_PRIVILEGED_TESTS=1.

#[test]
#[ignore = "requires /dev/uinput — set RUN_PRIVILEGED_TESTS=1"]
fn key_sequence_fires_with_gap_stub() {
    // TODO: implemented when injection thread exists (Plan 04)
}

#[test]
#[ignore = "requires /dev/uinput — set RUN_PRIVILEGED_TESTS=1"]
fn key_hold_emits_press_sleep_release_stub() {
    // TODO: implemented when injection thread exists (Plan 04)
}
