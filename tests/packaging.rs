// Static packaging artifact tests — no build tools required, no FUSE mount.
//
// Verifies that build.sh and PKGBUILD contain the structural elements that
// make the AppImage work at runtime: ORT library bundling, AppRun LD_LIBRARY_PATH,
// and the onnxruntime runtime dependency declaration.

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

#[test]
fn pkgbuild_has_clang_in_makedepends() {
    let src = read_file("packaging/PKGBUILD");
    // clang is required at build time by bindgen/clang-sys (transitive dep of sherpa-onnx-sys)
    let has_makedepends = src.lines().any(|l| l.contains("makedepends="));
    let has_clang = src.contains("'clang'");
    assert!(
        has_makedepends && has_clang,
        "PKGBUILD must list 'clang' inside makedepends=; got:\n{src}"
    );
}

#[test]
fn pkgbuild_includes_sherpa_onnx_offline_source() {
    let src = read_file("packaging/PKGBUILD");
    assert!(
        src.contains("sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2"),
        "PKGBUILD must include the sherpa-onnx 1.12.39 prebuilt archive as a source entry; got:\n{src}"
    );
}

#[test]
fn pkgbuild_sets_sherpa_onnx_archive_dir() {
    let src = read_file("packaging/PKGBUILD");
    assert!(
        src.contains("SHERPA_ONNX_ARCHIVE_DIR"),
        "PKGBUILD must export SHERPA_ONNX_ARCHIVE_DIR so the build script picks up the local archive; got:\n{src}"
    );
}

#[test]
fn release_yml_uploads_source_tarball() {
    let src = read_file(".github/workflows/release.yml");
    assert!(
        src.contains("git archive") && src.contains("vibe-attack-"),
        "release.yml must invoke git archive and reference a vibe-attack-*.tar.gz glob; got:\n{src}"
    );
}

#[test]
fn release_yml_uploads_hd2_hdpack() {
    let src = read_file(".github/workflows/release.yml");
    assert!(
        src.contains("profiles/hd2/pack.yaml") && src.contains(".hdpack"),
        "release.yml must zip profiles/hd2/pack.yaml and reference a hd2-*.hdpack glob; got:\n{src}"
    );
}

#[test]
fn release_yml_has_build_deb_job() {
    let src = read_file(".github/workflows/release.yml");
    assert!(
        src.lines().any(|l| l == "  build-deb:"),
        "release.yml must declare a 'build-deb:' job at column 2; got:\n{src}"
    );
}

#[test]
fn release_yml_has_build_rpm_job() {
    let src = read_file(".github/workflows/release.yml");
    assert!(
        src.lines().any(|l| l == "  build-rpm:"),
        "release.yml must declare a 'build-rpm:' job at column 2; got:\n{src}"
    );
}

#[test]
fn release_yml_uploads_deb_artifact() {
    let src = read_file(".github/workflows/release.yml");
    assert!(
        src.contains("vibe-attack_*.deb"),
        "release.yml must reference a vibe-attack_*.deb glob in the upload block; got:\n{src}"
    );
}

#[test]
fn release_yml_uploads_rpm_artifact() {
    let src = read_file(".github/workflows/release.yml");
    assert!(
        src.contains("vibe-attack-*.x86_64.rpm"),
        "release.yml must reference a vibe-attack-*.x86_64.rpm glob in the upload block; got:\n{src}"
    );
}

#[test]
fn release_yml_caches_sherpa_onnx_in_all_release_jobs() {
    let src = read_file(".github/workflows/release.yml");
    let count = src.matches("sherpa-onnx-1.12.39-linux-x64").count();
    assert!(
        count >= 3,
        "release.yml must reference sherpa-onnx-1.12.39-linux-x64 cache key at least 3 times \
         (once per build job: appimage, deb, rpm); found {count} occurrence(s)"
    );
}
