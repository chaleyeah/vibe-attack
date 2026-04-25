---
estimated_steps: 22
estimated_files: 2
skills_used: []
---

# T02: Rewrite README.md and create CONTRIBUTING.md

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

## Inputs

- ``tests/documentation.rs` — test contracts this task must satisfy`
- ``src/main.rs` — CLI struct and Commands enum for accurate documentation`
- ``config.example.yaml` — config file structure reference`

## Expected Output

- ``README.md` — complete rewrite with correct project name, deps, CLI docs`
- ``CONTRIBUTING.md` — contributor guide with build instructions and conventions`

## Verification

grep -q 'hd-linux-voice' README.md && ! grep -qi 'portaudio' README.md && grep -q '## Installation' README.md && grep -q '## Usage' README.md && test -f CONTRIBUTING.md && grep -q 'cargo build' CONTRIBUTING.md && echo PASS
