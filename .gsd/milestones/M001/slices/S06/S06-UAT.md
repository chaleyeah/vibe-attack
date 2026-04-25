# S06: Documentation — Usage docs, troubleshooting, and contributor guides — UAT

**Milestone:** M001
**Written:** 2026-04-25T20:00:08.011Z

# S06: Documentation — Usage docs, troubleshooting, and contributor guides — UAT

**Milestone:** M001
**Written:** 2026-04-25

## UAT Type

- UAT mode: artifact-driven
- Why this mode is sufficient: S06 is a documentation slice — all deliverables are files with required content. The 11 structural tests in `tests/documentation.rs` serve as the machine-checkable UAT; manual review confirms human readability.

## Preconditions

- Repository checked out to working tree
- Rust toolchain installed (`rustup show`)
- `tests/documentation.rs`, `README.md`, `CONTRIBUTING.md`, `docs/troubleshooting.md`, `docs/configuration.md` all present

## Smoke Test

```
cargo test --test documentation
```
All 11 tests must pass with no failures.

## Test Cases

### 1. Structural test suite passes

1. Run: `cargo test --test documentation --nocapture`
2. **Expected:** 11 tests pass, 0 failures. Output lists: readme_exists, readme_has_installation_section, readme_has_usage_section, readme_has_correct_project_name, readme_does_not_reference_portaudio, troubleshooting_doc_exists, troubleshooting_has_uinput_section, contributing_exists, contributing_has_build_section, configuration_doc_exists, configuration_has_ptt_section

### 2. README correctness

1. Open `README.md`
2. Verify header is `# hd-linux-voice` (not vibe-attack)
3. Verify `## Installation` section lists `libasound2-dev` (Debian) or `alsa-lib` (Arch) — no portaudio
4. Verify `## Usage` section documents daemon invocation, stdout=JSONL/stderr=logs split
5. Verify CLI subcommands listed: ping, switch, test, import, export, edit
6. **Expected:** All present; no portaudio anywhere in file

### 3. Portaudio regression guard

1. Run: `grep -i portaudio README.md`
2. **Expected:** No output (exit code 1 = no match)

### 4. CONTRIBUTING.md build instructions

1. Open `CONTRIBUTING.md`
2. Verify `## Building` section exists with `cargo build` variants (default, --features gui, --features stt)
3. Verify coding conventions section mentions no allocs in audio callback
4. **Expected:** All sections present and accurate

### 5. Troubleshooting guide — uinput section

1. Open `docs/troubleshooting.md`
2. Find the uinput section
3. Verify symptom (permission denied), likely cause (udev rules/group membership), and fix (link to docs/uinput-setup.md) are present
4. **Expected:** Section exists with actionable fix, cross-link to uinput-setup.md present

### 6. Configuration reference — ptt section

1. Open `docs/configuration.md`
2. Find the `ptt` section
3. Verify it explains the evdev key name, how to discover the key with evtest
4. **Expected:** ptt config documented with discovery instructions

### 7. ONNX conflict documented

1. Open `docs/troubleshooting.md` → Models section
2. Open `docs/configuration.md` → wake section
3. **Expected:** Both mention the dual ORT instance conflict between sherpa-onnx and the ort crate, directing users to disable wake word until resolved

## Edge Cases

### Vibe-attack / portaudio absent everywhere

1. Run: `grep -rni 'vibe-attack\|portaudio' README.md CONTRIBUTING.md docs/`
2. **Expected:** No matches in any file

### Test count stays ≥ 11

1. Run: `grep -c '#\[test\]' tests/documentation.rs`
2. **Expected:** Returns 11 or higher

## Failure Signals

- `cargo test --test documentation` shows any FAILED test
- `grep -i portaudio README.md` returns a match
- Any of the four doc files is missing
- `grep -c '#[test]' tests/documentation.rs` returns < 11

## Not Proven By This UAT

- Compiled cargo test execution (blocked in auto-mode; static verification substituted)
- Accuracy of uinput-setup.md content (pre-existing file, not authored in S06)
- Full rendering in a browser or markdown viewer
- Docs coverage of future CLI flags not yet implemented

## Notes for Tester

The most important manual check is reading the README Installation section to confirm a new user would not be sent to install portaudio. The structural tests assert presence but not prose quality. `docs/uinput-setup.md` is cross-linked from troubleshooting but was created in an earlier slice — verify the link target exists.
