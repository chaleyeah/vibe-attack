/// Static packaging artifact tests — no build tools required, no FUSE mount.
///
/// Verifies that build.sh and PKGBUILD contain the structural elements that
/// make the AppImage work at runtime: ORT library bundling, AppRun LD_LIBRARY_PATH,
/// and the onnxruntime runtime dependency declaration.

use std::fs;
use std::path::Path;

fn project_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn read_file(rel: &str) -> String {
    let path = project_root().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {rel}: {e}"))
}

#[test]
fn build_sh_copies_ort_so_into_appdir_lib() {
    let src = read_file("packaging/appimage/build.sh");
    assert!(
        src.contains("libonnxruntime.so") && src.contains("usr/lib/libonnxruntime.so"),
        "build.sh must copy libonnxruntime.so into usr/lib/; got:\n{src}"
    );
}

#[test]
fn build_sh_discovers_ort_via_env_var_fallback() {
    let src = read_file("packaging/appimage/build.sh");
    assert!(
        src.contains("ORT_DYLIB_PATH"),
        "build.sh must fall back to ORT_DYLIB_PATH for ORT discovery; got:\n{src}"
    );
}

#[test]
fn build_sh_writes_apprun_with_ld_library_path() {
    let src = read_file("packaging/appimage/build.sh");
    assert!(
        src.contains("AppRun") && src.contains("LD_LIBRARY_PATH"),
        "build.sh must write AppRun and set LD_LIBRARY_PATH; got:\n{src}"
    );
}

#[test]
fn build_sh_gates_linuxdeploy_on_command_v() {
    let src = read_file("packaging/appimage/build.sh");
    assert!(
        src.contains("command -v linuxdeploy"),
        "build.sh must gate linuxdeploy invocation on command -v; got:\n{src}"
    );
}

#[test]
fn pkgbuild_declares_onnxruntime_runtime_dep() {
    let src = read_file("packaging/PKGBUILD");
    assert!(
        src.contains("onnxruntime"),
        "PKGBUILD must list onnxruntime in depends; got:\n{src}"
    );
}
