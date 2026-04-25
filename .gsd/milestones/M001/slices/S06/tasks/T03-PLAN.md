---
estimated_steps: 19
estimated_files: 2
skills_used: []
---

# T03: Create docs/troubleshooting.md and docs/configuration.md, verify all tests pass

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

## Inputs

- ``tests/documentation.rs` — test contracts this task must satisfy`
- ``config.example.yaml` — authoritative config spec to document`
- ``docs/uinput-setup.md` — existing guide to cross-link (not duplicate)`

## Expected Output

- ``docs/troubleshooting.md` — consolidated troubleshooting guide with uinput, audio, model, and build sections`
- ``docs/configuration.md` — prose configuration reference covering all config.example.yaml sections`

## Verification

test -f docs/troubleshooting.md && grep -qi 'uinput' docs/troubleshooting.md && test -f docs/configuration.md && grep -qi 'ptt' docs/configuration.md && echo PASS
