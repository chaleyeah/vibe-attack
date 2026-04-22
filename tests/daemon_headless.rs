//! Verify daemon binary is headless (UI-01) — no display surface creation.

use std::process::Command;

/// Locate the daemon binary in the target/debug/ directory.
fn daemon_bin() -> std::path::PathBuf {
    let mut p = std::env::current_dir().unwrap();
    for _ in 0..3 {
        let bin = p.join("target/debug/hd-linux-voice");
        if bin.exists() {
            return bin;
        }
        let parent = p.join("../target/debug/hd-linux-voice");
        if parent.exists() {
            return parent;
        }
        if let Some(up) = p.parent() {
            p = up.to_path_buf();
        } else {
            break;
        }
    }
    std::path::PathBuf::from("target/debug/hd-linux-voice")
}

#[test]
fn daemon_binary_does_not_link_gui_libraries() {
    let bin = daemon_bin();
    if !bin.exists() {
        eprintln!("Daemon binary not found at {:?}, skipping link check", bin);
        return;
    }

    let output = Command::new("ldd").arg(&bin).output();

    match output {
        Ok(out) => {
            let libs = String::from_utf8_lossy(&out.stdout);
            let forbidden = [
                "libwayland-client",
                "libX11",
                "libxcb",
                "libgtk",
                "libgdk",
            ];
            for lib in &forbidden {
                assert!(
                    !libs.contains(lib),
                    "Daemon must not link against {} (UI-01: headless only). ldd output:\n{libs}",
                    lib
                );
            }
        }
        Err(_) => {
            eprintln!("ldd not available, skipping shared library check");
        }
    }
}

#[test]
fn daemon_exits_with_error_on_missing_config() {
    let bin = daemon_bin();
    if !bin.exists() {
        eprintln!("Daemon binary not found, skipping");
        return;
    }

    let output = Command::new(&bin)
        .arg("--config")
        .arg("/tmp/hd_linux_voice_nonexistent_config_xyz.yaml")
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("DISPLAY")
        .output()
        .expect("Failed to spawn daemon");

    assert!(
        !output.status.success(),
        "Daemon must exit non-zero when config is missing"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found")
            || stderr.contains("No such file")
            || stderr.contains("Config")
            || stderr.contains("os error"),
        "Stderr must mention config error, got: {stderr}"
    );

    assert!(
        !stderr.contains("wayland") && !stderr.contains("DISPLAY"),
        "Daemon must not attempt display server connection on startup, got: {stderr}"
    );
}

#[test]
fn uinput_permission_denied_message_links_to_docs() {
    // Test that DaemonError::UinputPermissionDenied mentions the setup doc (D-15).
    let err_msg = format!("{}", hd_linux_voice::error::DaemonError::UinputPermissionDenied);
    assert!(
        err_msg.contains("uinput-setup.md"),
        "D-15 error must link to docs/uinput-setup.md, got: {err_msg}"
    );
}
