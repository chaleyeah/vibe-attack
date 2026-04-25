# S06: Documentation — Usage docs, troubleshooting, and contributor guides

**Goal:** Produce usage documentation (README rewrite, troubleshooting guide, contributor guide, configuration reference) and a structural test file that proves all docs exist with required sections.
**Demo:** unit tests prove Documentation — Usage docs, troubleshooting, and contributor guides works

## Must-Haves

- `cargo test --test documentation` passes with 11+ tests\n- `README.md` references `hd-linux-voice` (not `vibe-attack`) and does not mention `portaudio`\n- `docs/troubleshooting.md`, `docs/configuration.md`, and `CONTRIBUTING.md` all exist with required sections\n- `grep -c '#[test]' tests/documentation.rs` returns >= 11

## Proof Level

- This slice proves: Not provided.

## Integration Closure

Not provided.

## Verification

- Not provided.

## Tasks

- [x] **T01: Write tests/documentation.rs with structural assertions for all planned docs** `est:20m`
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
  - Files: `tests/documentation.rs`
  - Verify: grep -c '#[test]' tests/documentation.rs returns >= 11

- [ ] **T02: Rewrite README.md and create CONTRIBUTING.md** `est:40m`
  Rewrite README.md and create CONTRIBUTING.md to satisfy tests 1-5, 8-9 from T01.

**README.md rewrite:**
- Change project name header to `# hd-linux-voice`
- Remove ALL references to `portaudio` — actual audio dep is ALSA/CPAL
- Installation section (`## Installation`): list system deps as `libasound2-dev` (Debian) or `alsa-lib` (Arch), plus Rust stable toolchain. No portaudio.
- Usage section (`## Usage`): document that running with no subcommand starts the daemon. stdout = JSONL transcripts, stderr = logs. Document `-v`/`-vv` verbosity, `--config FILE`, `--list-devices`.
- Document all CLI subcommands from src/main.rs Commands enum: `ping`, `switch <name>`, `test <name>`, `import <file>`, `export <name> [output]`, `edit`
- Add config file location: `~/.config/hd-linux-voice/config.yaml` (XDG_CONFIG_HOME)
- Add model download note: Whisper model must be downloaded manually (no auto-download)
- Link to `docs/uinput-setup.md` for uinput/evdev permissions
- Link to `docs/troubleshooting.md` and `docs/configuration.md`
- Keep the existing features list but update to reflect current state
- License: AGPL-3.0-only (matches Cargo.toml)

**CONTRIBUTING.md:**
- Dev prerequisites: Rust stable, libasound2-dev/alsa-lib, evdev/uinput access
- Building section with `## Building`: `cargo build` (default, no GUI), `cargo build --features gui` (with egui config window), `cargo build --features stt` (with whisper)
- Running tests: `cargo test` — all pass without hardware; hardware tests opt-in
- Brief architecture note: two-stage pipeline (VAD→STT→dispatch), stdout=JSONL contract. Link to detailed architecture docs if they exist.
- Coding conventions: no allocations in audio callback, STT always spawn_blocking, stdout reserved for JSONL
- PR process brief
- Link to `docs/pack-format.md` for pack authoring (future)

IMPORTANT: Do NOT reference `portaudio` anywhere. The project uses CPAL with ALSA backend.
  - Files: `README.md`, `CONTRIBUTING.md`
  - Verify: grep -q 'hd-linux-voice' README.md && ! grep -qi 'portaudio' README.md && grep -q '## Installation' README.md && grep -q '## Usage' README.md && test -f CONTRIBUTING.md && grep -q 'cargo build' CONTRIBUTING.md && echo PASS

- [ ] **T03: Create docs/troubleshooting.md and docs/configuration.md, verify all tests pass** `est:40m`
  Create the two remaining doc files to satisfy tests 6-7, 10-11 from T01, then run the full test suite.

**docs/troubleshooting.md:**
Consolidate known failure modes into sections:
- `## uinput / /dev/uinput` — permission denied, module not loaded, systemd v258+ group issue. Cross-link to `docs/uinput-setup.md` (already thorough, don't duplicate).
- `## Audio / CPAL` — no input devices found, wrong device selected. Mention `--list-devices` flag.
- `## Models` — Whisper model path not found, ONNX Runtime issues. Models must be manually downloaded.
- `## STT Accuracy` — low confidence, noisy environment, vocabulary mismatch.
- `## Daemon` — UDS socket missing, `ping` subcommand for health check.
- `## Build` — missing ALSA dev headers, cmake for whisper-rs, feature flags.

Each section: symptom → likely cause → fix command/action.

**docs/configuration.md:**
Prose companion to `config.example.yaml`:
- Header explaining XDG config path (`~/.config/hd-linux-voice/config.yaml`)
- `--config` CLI override
- Section-by-section reference: `ptt` (evdev key name, how to find with evtest), `audio` (CPAL device, --list-devices), `timing` (dwell_ms, gap_ms), `pipeline` (verbosity, listen_window), `vad` (thresholds, speech detection params), `stt` (model path, enabled flag), `wake` (wake word config), `macros` (phrase list, key sequences)
- Must contain the word `ptt` to satisfy test 11

**Verification:**
Run `cargo test --test documentation` to confirm all 11 tests pass. If cargo test is blocked in auto-mode (MEM004), perform static verification: confirm all files exist, grep for required section headings, confirm no `portaudio` or `vibe-attack` references in README.md.

Also run: `grep -c '#[test]' tests/documentation.rs` to confirm >= 11 tests exist.
  - Files: `docs/troubleshooting.md`, `docs/configuration.md`
  - Verify: test -f docs/troubleshooting.md && grep -qi 'uinput' docs/troubleshooting.md && test -f docs/configuration.md && grep -qi 'ptt' docs/configuration.md && echo PASS

## Files Likely Touched

- tests/documentation.rs
- README.md
- CONTRIBUTING.md
- docs/troubleshooting.md
- docs/configuration.md
