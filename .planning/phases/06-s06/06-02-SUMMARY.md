---
phase: "06"
plan: "02"
---

# T02: Rewrote README.md with correct project name and ALSA deps; created CONTRIBUTING.md with build instructions and coding conventions

**Rewrote README.md with correct project name and ALSA deps; created CONTRIBUTING.md with build instructions and coding conventions**

## What Happened

Rewrote README.md from scratch: replaced the stale "vibe-attack" header with "# hd-linux-voice", removed all portaudio references, rewrote the Installation section to list libasound2-dev (Debian) and alsa-lib (Arch) as the correct audio deps, added a Usage section documenting the daemon invocation, stdout JSONL/stderr logs split, -v/-vv/--config/--list-devices flags, and all CLI subcommands (ping, switch, test, import, export, edit) sourced from the Commands enum in src/main.rs. Added config file location (~/.config/hd-linux-voice/config.yaml), Whisper model download note, links to docs/uinput-setup.md, docs/troubleshooting.md, and docs/configuration.md. License updated to AGPL-3.0-only. Created CONTRIBUTING.md with dev prerequisites (Rust stable, libasound2-dev/alsa-lib, evdev/uinput access), a Building section with cargo build variants (default, --features gui, --features stt, --release), cargo test instructions, a two-stage pipeline architecture overview, and coding conventions (no allocs in audio callback, STT always spawn_blocking, stdout reserved for JSONL).

## Verification

Ran task verification: `grep -q 'hd-linux-voice' README.md && ! grep -qi 'portaudio' README.md && grep -q '## Installation' README.md && grep -q '## Usage' README.md && test -f CONTRIBUTING.md && grep -q 'cargo build' CONTRIBUTING.md && echo PASS` — exits 0, prints PASS. Confirmed tests/documentation.rs has 11 #[test] functions (exit 0 from grep -c).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `grep -q 'hd-linux-voice' README.md && ! grep -qi 'portaudio' README.md && grep -q '## Installation' README.md && grep -q '## Usage' README.md && test -f CONTRIBUTING.md && grep -q 'cargo build' CONTRIBUTING.md && echo PASS` | 0 | ✅ pass | 20ms |
| 2 | `grep -c '#\[test\]' tests/documentation.rs` | 0 | ✅ pass — 11 tests | 10ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `README.md`
- `CONTRIBUTING.md`
