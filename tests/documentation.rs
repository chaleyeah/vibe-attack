// Structural tests for S06 documentation contracts.
// All tests use env!("CARGO_MANIFEST_DIR") for portable path resolution.
// T02 and T03 create the docs that make these tests pass.

#[test]
fn readme_exists() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file = root.join("README.md");
    assert!(file.exists(), "README.md must exist");
}

#[test]
fn readme_has_installation_section() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file = root.join("README.md");
    let contents = std::fs::read_to_string(&file).expect("failed to read README.md");
    assert!(
        contents.contains("## Installation"),
        "README.md must contain '## Installation'"
    );
}

#[test]
fn readme_has_usage_section() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file = root.join("README.md");
    let contents = std::fs::read_to_string(&file).expect("failed to read README.md");
    assert!(
        contents.contains("## Usage") || contents.contains("## Running"),
        "README.md must contain '## Usage' or '## Running'"
    );
}

#[test]
fn readme_has_correct_project_name() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file = root.join("README.md");
    let contents = std::fs::read_to_string(&file).expect("failed to read README.md");
    assert!(
        contents.contains("hd-linux-voice"),
        "README.md must contain 'hd-linux-voice'"
    );
    assert!(
        !contents.contains("vibe-attack"),
        "README.md must not reference 'vibe-attack' (stale project name)"
    );
}

#[test]
fn readme_does_not_reference_portaudio() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file = root.join("README.md");
    let contents = std::fs::read_to_string(&file).expect("failed to read README.md");
    assert!(
        !contents.contains("portaudio"),
        "README.md must not mention 'portaudio' (regression guard for removed dependency)"
    );
}

#[test]
fn troubleshooting_doc_exists() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file = root.join("docs/troubleshooting.md");
    assert!(file.exists(), "docs/troubleshooting.md must exist");
}

#[test]
fn troubleshooting_has_uinput_section() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file = root.join("docs/troubleshooting.md");
    let contents = std::fs::read_to_string(&file).expect("failed to read docs/troubleshooting.md");
    let lower = contents.to_lowercase();
    assert!(
        lower.contains("uinput"),
        "docs/troubleshooting.md must contain a 'uinput' section"
    );
}

#[test]
fn contributing_exists() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file = root.join("CONTRIBUTING.md");
    assert!(file.exists(), "CONTRIBUTING.md must exist");
}

#[test]
fn contributing_has_build_section() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file = root.join("CONTRIBUTING.md");
    let contents = std::fs::read_to_string(&file).expect("failed to read CONTRIBUTING.md");
    assert!(
        contents.contains("cargo build") || contents.contains("## Build"),
        "CONTRIBUTING.md must contain 'cargo build' or '## Build'"
    );
}

#[test]
fn configuration_doc_exists() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file = root.join("docs/configuration.md");
    assert!(file.exists(), "docs/configuration.md must exist");
}

#[test]
fn configuration_has_ptt_section() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file = root.join("docs/configuration.md");
    let contents = std::fs::read_to_string(&file).expect("failed to read docs/configuration.md");
    let lower = contents.to_lowercase();
    assert!(
        lower.contains("ptt"),
        "docs/configuration.md must contain a 'ptt' section (push-to-talk config)"
    );
}
