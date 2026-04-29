// Static tests asserting the distribution-proof transcript structure and
// verify-appimage.sh integrity — no FUSE mount, no VM required.
//
// Transcripts may carry any of three valid STATUS values:
//   STATUS: ok                    (full VM run succeeded)
//   STATUS: skipped:tools-missing (build host lacks linuxdeploy/appimagetool)
//   STATUS: pending VM run        (placeholder until VM run is executed in S02)
// All three are accepted here; only structural completeness is enforced.
//
// Supported distros (M011): Debian 13, Ubuntu 26.04, Fedora 44, CachyOS

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

fn project_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn read_file(rel: &str) -> String {
    let path = project_root().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {rel}: {e}"))
}

const REQUIRED_FIELDS: &[&str] = &[
    "STATUS:",
    "DISTRO:",
    "KERNEL:",
    "SIZE_BYTES:",
    "SHA256:",
    "EXIT_CODE:",
    "VERSION_OUTPUT:",
];

const VALID_STATUSES: &[&str] = &[
    "STATUS: ok",
    "STATUS: skipped:tools-missing",
    "STATUS: pending VM run",
];

fn assert_transcript(rel: &str) {
    let content = read_file(rel);
    for field in REQUIRED_FIELDS {
        assert!(
            content.contains(field),
            "transcript {rel} is missing required field '{field}'"
        );
    }
    let has_valid_status = VALID_STATUSES.iter().any(|s| content.contains(s));
    assert!(
        has_valid_status,
        "transcript {rel} has no recognized STATUS line; accepted values: {:?}",
        VALID_STATUSES
    );
}

#[test]
fn debian13_transcript_has_required_fields() {
    assert_transcript("docs/distribution-proofs/appimage/debian13/transcript.md");
}

#[test]
fn ubuntu2604_transcript_has_required_fields() {
    assert_transcript("docs/distribution-proofs/appimage/ubuntu2604/transcript.md");
}

#[test]
fn fedora44_transcript_has_required_fields() {
    assert_transcript("docs/distribution-proofs/appimage/fedora44/transcript.md");
}

#[test]
fn cachyos_transcript_has_required_fields() {
    assert_transcript("docs/distribution-proofs/appimage/cachyos/transcript.md");
}

#[test]
fn verify_appimage_sh_exists_and_is_executable() {
    let path = project_root().join("scripts/verify-appimage.sh");
    assert!(
        path.exists(),
        "scripts/verify-appimage.sh must exist"
    );
    let meta = fs::metadata(&path).expect("cannot stat scripts/verify-appimage.sh");
    let mode = meta.permissions().mode();
    assert!(
        mode & 0o111 != 0,
        "scripts/verify-appimage.sh must be executable (mode={mode:#o})"
    );
}

#[test]
fn verify_appimage_sh_contains_status_emitter() {
    let src = read_file("scripts/verify-appimage.sh");
    assert!(
        src.contains("STATUS:"),
        "scripts/verify-appimage.sh must contain a STATUS: line emitter"
    );
    // The script writes the structured transcript unconditionally — even on failure.
    assert!(
        src.contains("echo \"STATUS:"),
        "scripts/verify-appimage.sh must emit STATUS via echo"
    );
}

#[test]
fn build_sh_bundles_both_ort_libraries() {
    let src = read_file("packaging/appimage/build.sh");
    assert!(
        src.contains("libonnxruntime.so"),
        "build.sh must reference libonnxruntime.so for dual-ORT bundling"
    );
    assert!(
        src.contains("libsherpa-onnx-c-api.so"),
        "build.sh must reference libsherpa-onnx-c-api.so for dual-ORT bundling"
    );
}

// ── Final UAT proof helpers ──────────────────────────────────────────────────

const FINAL_REQUIRED_FIELDS: &[&str] = &[
    "STATUS:",
    "DISTRO:",
    "KERNEL:",
    "APPIMAGE_VERSION:",
    "APPIMAGE_SIZE_BYTES:",
    "WIZARD_COMPLETED:",
    "STRATAGEM_FIRED:",
    "INSTALL_METHOD:",
];

fn assert_final_transcript(rel: &str) {
    let content = read_file(rel);
    for field in FINAL_REQUIRED_FIELDS {
        assert!(
            content.contains(field),
            "transcript {rel} is missing required field '{field}'"
        );
    }
    // Accept: "STATUS: ok", "STATUS: pending VM run", or "STATUS: failed:<reason>"
    let has_valid_status = content.contains("STATUS: ok")
        || content.contains("STATUS: pending VM run")
        || content.lines().any(|l| l.starts_with("STATUS: failed:"));
    assert!(
        has_valid_status,
        "transcript {rel} has no recognized STATUS line; accepted: 'ok', 'pending VM run', or 'failed:<reason>'"
    );
}

#[test]
fn debian13_final_transcript_has_required_fields() {
    assert_final_transcript("docs/distribution-proofs/final/debian13/transcript.md");
}

#[test]
fn ubuntu2604_final_transcript_has_required_fields() {
    assert_final_transcript("docs/distribution-proofs/final/ubuntu2604/transcript.md");
}

#[test]
fn fedora44_final_transcript_has_required_fields() {
    assert_final_transcript("docs/distribution-proofs/final/fedora44/transcript.md");
}

#[test]
fn cachyos_final_transcript_has_required_fields() {
    assert_final_transcript("docs/distribution-proofs/final/cachyos/transcript.md");
}
