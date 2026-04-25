---
id: S06
parent: M001
milestone: M001
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - ["tests/documentation.rs", "README.md", "CONTRIBUTING.md", "docs/troubleshooting.md", "docs/configuration.md"]
key_decisions:
  - ["Used case-insensitive search (to_lowercase().contains()) for uinput and ptt checks — avoids brittle exact-case matching while still asserting the concepts are covered", "Removed portaudio entirely from README — project uses CPAL/ALSA backend; regression test guards against re-introduction", "Sourced CLI subcommands directly from src/main.rs Commands enum to stay accurate with the actual implementation", "Cross-linked to docs/uinput-setup.md from troubleshooting rather than duplicating udev/group setup content", "ONNX Runtime dual-instance conflict noted in both troubleshooting (Models) and configuration (wake) for discoverability"]
patterns_established:
  - ["Structural doc tests follow env!(CARGO_MANIFEST_DIR) + std::fs::read_to_string + to_lowercase().contains() — same pattern as tests/ui_distribution.rs from S05", "Documentation TDD: write test contracts first (T01), create docs to satisfy them (T02/T03), verify statically when cargo test is gated"]
observability_surfaces:
  - none
drill_down_paths:
  - [".gsd/milestones/M001/slices/S06/tasks/T01-SUMMARY.md", ".gsd/milestones/M001/slices/S06/tasks/T02-SUMMARY.md", ".gsd/milestones/M001/slices/S06/tasks/T03-SUMMARY.md"]
duration: ""
verification_result: passed
completed_at: 2026-04-25T20:00:08.011Z
blocker_discovered: false
---

# S06: Documentation — Usage docs, troubleshooting, and contributor guides

**Produced complete project documentation (README rewrite, CONTRIBUTING.md, troubleshooting guide, configuration reference) backed by 11 structural tests that serve as a living regression guard.**

## What Happened

S06 delivered the full documentation set for hd-linux-voice using a TDD-first approach across three tasks.

**T01** established the test contract by writing `tests/documentation.rs` with 11 `#[test]` functions covering all four target files. Tests follow the `env!("CARGO_MANIFEST_DIR")` portable path pattern from `tests/ui_distribution.rs` (S05), using `std::fs::read_to_string` for content checks and `to_lowercase().contains()` for case-insensitive heading assertions. All tests were intentionally failing at completion — the docs didn't exist yet.

**T02** rewrote `README.md` from scratch (replacing the stale "vibe-attack" header with "hd-linux-voice", removing all portaudio references, adding correct ALSA/CPAL deps, documenting all CLI subcommands sourced directly from `src/main.rs`) and created `CONTRIBUTING.md` with build variants, test instructions, pipeline architecture overview, and coding conventions (no allocs in audio callback, STT always spawn_blocking, stdout reserved for JSONL).

**T03** created `docs/troubleshooting.md` (six sections: uinput, Audio/CPAL, Models, STT Accuracy, Daemon, Build — each following symptom→cause→fix) and `docs/configuration.md` (prose companion to config.example.yaml covering all config sections with the ONNX Runtime conflict noted in the wake section). With both files present, all 11 test contracts were satisfied statically verified.

The ONNX Runtime dual-instance conflict (sherpa-onnx static ORT vs ort crate dynamic ORT, tracked since Phase 2) is now documented in two places for discoverability: troubleshooting Models section and configuration wake section. Cargo test could not run in auto-mode (permission gate); static verification confirmed all 11 test contracts pass.

## Verification

All slice must-haves verified:
- `grep -c '#[test]' tests/documentation.rs` → 11 (≥ 11 required)
- `grep -q 'hd-linux-voice' README.md` → exit 0
- `! grep -qi 'portaudio' README.md` → exit 0 (portaudio absent)
- `grep -q '## Installation' README.md` → exit 0
- `grep -q '## Usage' README.md` → exit 0
- `test -f CONTRIBUTING.md && grep -q 'cargo build' CONTRIBUTING.md` → exit 0
- `test -f docs/troubleshooting.md && grep -qi 'uinput' docs/troubleshooting.md` → exit 0
- `test -f docs/configuration.md && grep -qi 'ptt' docs/configuration.md` → exit 0
- Combined one-liner: ALL_PASS

Cargo test blocked by auto-mode permission gate (MEM004); static verification substituted per task plan guidance. Compiled test execution should be confirmed in a manual session.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

Cargo test suite could not be run in auto-mode due to permission gate (MEM004). Static verification was substituted per the task plan's own guidance — all 11 test contracts confirmed via grep against the actual file contents.

## Known Limitations

Compiled cargo test execution has not been confirmed in this session — only static verification. A manual `cargo test --test documentation` run should be performed before shipping. docs/uinput-setup.md (cross-linked from troubleshooting) was created in a prior slice and was not re-verified in S06.

## Follow-ups

Run `cargo test --test documentation` in a non-gated session to confirm compiled test execution. S07 (Wake Word / dual ORT conflict resolution) depends on S06 — the troubleshooting and configuration docs already surface the conflict for end users.

## Files Created/Modified

- `tests/documentation.rs` — 11 structural tests asserting file existence and required section headings for all four doc files
- `README.md` — Full rewrite: correct project name, ALSA deps, CLI reference, config path, no portaudio
- `CONTRIBUTING.md` — New file: dev prerequisites, build variants, test instructions, architecture overview, coding conventions
- `docs/troubleshooting.md` — New file: six sections (uinput, Audio/CPAL, Models, STT Accuracy, Daemon, Build) with symptom→cause→fix
- `docs/configuration.md` — New file: section-by-section prose reference for all config keys including ptt, audio, timing, pipeline, vad, stt, wake, macros
