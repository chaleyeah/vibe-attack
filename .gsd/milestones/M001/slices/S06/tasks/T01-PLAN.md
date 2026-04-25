---
estimated_steps: 23
estimated_files: 1
skills_used: []
---

# T01: Write tests/documentation.rs with structural assertions for all planned docs

Create the test-first contract for S06. Write `tests/documentation.rs` with 11 `#[test]` functions that assert file existence and required section headings for README.md, CONTRIBUTING.md, docs/troubleshooting.md, and docs/configuration.md. All tests use `env!("CARGO_MANIFEST_DIR")` for portable path resolution (same pattern as tests/ui_distribution.rs per MEM008). Tests will initially fail — that's expected; T02 and T03 create the docs that satisfy them.

Tests to write:
1. `readme_exists` — README.md exists
2. `readme_has_installation_section` — contains `## Installation`
3. `readme_has_usage_section` — contains `## Usage` or `## Running`
4. `readme_has_correct_project_name` — contains `hd-linux-voice`, does NOT contain `vibe-attack`
5. `readme_does_not_reference_portaudio` — `portaudio` must not appear (regression guard for stale dep)
6. `troubleshooting_doc_exists` — docs/troubleshooting.md exists
7. `troubleshooting_has_uinput_section` — contains `uinput` (case-insensitive search or exact)
8. `contributing_exists` — CONTRIBUTING.md exists
9. `contributing_has_build_section` — contains `cargo build` or `## Build`
10. `configuration_doc_exists` — docs/configuration.md exists
11. `configuration_has_ptt_section` — contains `ptt` (the most critical config section)

Pattern for each test:
```rust
#[test]
fn readme_exists() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file = root.join("README.md");
    assert!(file.exists(), "README.md must exist");
}
```

For content checks, read the file with `std::fs::read_to_string` and use `assert!(contents.contains(...))`. For negative checks (portaudio), use `assert!(!contents.contains("portaudio"))`.

## Inputs

- ``tests/ui_distribution.rs` — reference for the env!("CARGO_MANIFEST_DIR") structural test pattern`

## Expected Output

- ``tests/documentation.rs` — 11 structural tests defining documentation contracts`

## Verification

grep -c '#[test]' tests/documentation.rs returns >= 11
