---
phase: "06"
plan: "03"
---

# T03: Created docs/troubleshooting.md and docs/configuration.md satisfying all 11 documentation structural tests

**Created docs/troubleshooting.md and docs/configuration.md satisfying all 11 documentation structural tests**

## What Happened

Created two missing documentation files to complete the S06 docs slice.

**docs/troubleshooting.md** covers six sections (uinput/dev/uinput, Audio/CPAL, Models, STT Accuracy, Daemon, Build), each following the symptom → cause → fix pattern specified in the task plan. The uinput section cross-links to docs/uinput-setup.md rather than duplicating the full udev/group setup there. The Models section notes the ONNX Runtime conflict risk (dual ORT instances) discovered in a prior session (MEM004). The file contains "uinput" (case-insensitive) 9+ times, satisfying test 7.

**docs/configuration.md** is a prose companion to config.example.yaml, organized section-by-section: ptt (with evtest discovery note), audio (CPAL device selection, --list-devices), timing (dwell_ms/gap_ms table), pipeline (verbosity/listen_window), vad (all seven threshold params in a reference table), stt (model path, initial_prompt, download instructions), wake (sherpa-onnx artifact paths, ORT conflict warning), and macros (phrase-to-key mapping structure). The XDG config path and --config CLI override are explained in the header. The word "ptt" appears multiple times satisfying test 11.

Cargo test was blocked in auto-mode (MEM004 / permission gate), so static verification was performed: all four doc files confirmed present, required strings confirmed via grep, portaudio/vibe-attack absence confirmed in README.md, CONTRIBUTING.md build section confirmed. All 11 test contracts verified statically.

## Verification

Static verification of all 11 test contracts from tests/documentation.rs:
- Tests 1-5 (README): file exists, ## Installation present, ## Usage present, hd-linux-voice present, no vibe-attack/portaudio — all pass
- Tests 6-7 (troubleshooting): file exists, uinput present case-insensitively — both pass
- Tests 8-9 (CONTRIBUTING): file exists, cargo build present — both pass
- Tests 10-11 (configuration): file exists, ptt present case-insensitively — both pass

Task plan shell check: `test -f docs/troubleshooting.md && grep -qi 'uinput' docs/troubleshooting.md && test -f docs/configuration.md && grep -qi 'ptt' docs/configuration.md && echo PASS` → PASS

grep -c '#[test]' tests/documentation.rs → 11 (meets >= 11 requirement)

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `test -f docs/troubleshooting.md && grep -qi uinput docs/troubleshooting.md && test -f docs/configuration.md && grep -qi ptt docs/configuration.md && echo PASS` | 0 | ✅ pass | 20ms |
| 2 | `grep -c '#[test]' tests/documentation.rs` | 0 | ✅ pass — 11 tests | 10ms |
| 3 | `grep -n '## Installation' README.md` | 0 | ✅ pass — line 13 | 10ms |
| 4 | `grep -n '## Usage' README.md` | 0 | ✅ pass — line 61 | 10ms |
| 5 | `grep -rn 'vibe-attack|portaudio' README.md (no matches)` | 1 | ✅ pass — absent | 10ms |
| 6 | `grep -n 'cargo build' CONTRIBUTING.md` | 0 | ✅ pass — multiple lines | 10ms |
| 7 | `cargo test --test documentation (blocked by permission gate)` | -1 | ⚠️ blocked — static verification substituted | 0ms |

## Deviations

None — files match the task plan specification exactly. Cargo test substituted with static verification per MEM004 guidance in the task plan itself.

## Known Issues

Cargo test suite could not be run in auto-mode (permission gate). The static verification covers all 11 test contracts, but actual compiled test execution should be confirmed in a manual session.

## Files Created/Modified

- `docs/troubleshooting.md`
- `docs/configuration.md`
