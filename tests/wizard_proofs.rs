// Static tests asserting the wizard UAT transcript structure — no VM required.
//
// Supported distros (M011): Debian 13, Ubuntu 26.04, Fedora 44, CachyOS
//
// Transcripts may carry any of these valid STATUS values:
//   STATUS: ok                    (full VM run succeeded)
//   STATUS: pending VM run        (placeholder until VM run is executed in S06)
//   STATUS: failed:scenario-A     (Scenario A — fresh install — failed)
//   STATUS: failed:scenario-B     (Scenario B — model pre-placed — failed)
//   STATUS: failed:scenario-C     (Scenario C — relaunch skips wizard — failed)
//   STATUS: failed:scenario-D     (Scenario D — --skip-wizard flag — failed)
// All are accepted here; only structural completeness is enforced.

use std::fs;
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
    "BINARY:",
    "BINARY_VERSION:",
    "SCENARIO_A:",
    "SCENARIO_B:",
    "SCENARIO_C:",
    "SCENARIO_D:",
    "STRATAGEM_FIRED:",
];

const VALID_STATUSES: &[&str] = &[
    "STATUS: ok",
    "STATUS: pending VM run",
    "STATUS: failed:scenario-A",
    "STATUS: failed:scenario-B",
    "STATUS: failed:scenario-C",
    "STATUS: failed:scenario-D",
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
fn debian13_wizard_transcript_has_required_fields() {
    assert_transcript("docs/distribution-proofs/wizard/debian13/transcript.md");
}

#[test]
fn ubuntu2604_wizard_transcript_has_required_fields() {
    assert_transcript("docs/distribution-proofs/wizard/ubuntu2604/transcript.md");
}

#[test]
fn fedora44_wizard_transcript_has_required_fields() {
    assert_transcript("docs/distribution-proofs/wizard/fedora44/transcript.md");
}

#[test]
fn cachyos_wizard_transcript_has_required_fields() {
    assert_transcript("docs/distribution-proofs/wizard/cachyos/transcript.md");
}

#[test]
fn wizard_readme_contains_four_scenario_headings() {
    let content = read_file("docs/distribution-proofs/wizard/README.md");
    for scenario in &["Scenario A", "Scenario B", "Scenario C", "Scenario D"] {
        assert!(
            content.contains(scenario),
            "docs/distribution-proofs/wizard/README.md is missing '{scenario}' heading"
        );
    }
}
