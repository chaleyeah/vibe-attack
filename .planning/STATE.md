---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: verifying
stopped_at: Completed 02-pipeline-core-04-PLAN.md
last_updated: "2026-04-22T12:38:01.464Z"
last_activity: 2026-04-22
progress:
  total_phases: 6
  completed_phases: 2
  total_plans: 9
  completed_plans: 9
  percent: 100
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-04-21)

**Core value:** During Helldivers 2 gameplay, the player can fire the right stratagem reliably by voice with minimal delay and without breaking flow — wake word or push-to-talk, fully local speech processing, Wayland-first input delivery.
**Current focus:** Phase 2 — Pipeline Core

## Current Position

Phase: 2 (Pipeline Core) — EXECUTING
Plan: 4 of 4
Status: Phase complete — ready for verification
Last activity: 2026-04-22

Progress: [██████████] 100%

## Performance Metrics

**Velocity:**

- Total plans completed: 5
- Average duration: —
- Total execution time: 0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 1 | 5 | - | - |

**Recent Trend:**

- Last 5 plans: —
- Trend: —

*Updated after each plan completion*
| Phase 01-foundation P02 | 153s | 2 tasks | 5 files |
| Phase 01-foundation P03 | 12m | 2 tasks | 2 files |
| Phase 01-foundation P04 | 8m | 2 tasks | 4 files |
| Phase 01-foundation P05 | 15m | 2 tasks | 6 files |
| Phase 02-pipeline-core P01 | 161s | 3 tasks | 8 files |
| Phase 02-pipeline-core P02 | 200s | 2 tasks | 7 files |
| Phase 02 P03 | 420s | 2 tasks | 11 files |
| Phase 02-pipeline-core P03 | 420s | 2 tasks | 11 files |
| Phase 02-pipeline-core P04 | 160s | 3 tasks | 3 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- Pre-Phase 1: PTT must use evdev EVIOCGRAB on physical device (not compositor global hotkey) — Wayland fullscreen focus kills compositor-level shortcuts
- Pre-Phase 1: Daemon is headless by default; config UI is a separate process — prevents XWayland focus race minimizing fullscreen game
- Pre-Phase 1: No Flatpak packaging — uinput/evdev incompatible with Flatpak sandbox; AppImage + AUR are primary targets
- Pre-Phase 1: STT never runs on Tokio executor; audio RT callback never allocates — latency invariant from day one
- Pre-Phase 1: Phase 2 is highest technical risk — latency must be proven before Phase 3 dispatch work begins
- Phase 01 Plan 01: tokio-util has no named "sync" feature — CancellationToken in tokio_util::sync is always available without feature flags
- Phase 01 Plan 01: about.hbs excludes root crate by name (not publish=false) to preserve crates.io publishability
- Phase 01 Plan 01: serde_yaml_ng enforced over serde_yaml — deprecated March 2024 with unresolved libyaml CVE
- Config struct hierarchy with deny_unknown_fields on all 4 structs; serde_yaml_ng enforced
- src/lib.rs created for integration test access; macros field is serde(default) making it optional
- xdg::place_config_file used (not get_config_file) — returns Result<PathBuf> unconditionally
- Manual /dev/input/event{0..63} loop avoids glob crate dependency
- ringbuf 0.4 traits (Producer/Split/Consumer) must be explicitly imported
- CPAL 0.17 SampleRate is u32 type alias; device.description().name() replaces deprecated name()
- VirtualDevice::builder() used (VirtualDeviceBuilder::new() deprecated in evdev 0.13)
- 'input' group in DaemonError::UinputPermissionDenied message (not 'uinput') — systemd v258+ Pitfall 3
- No SYN_REPORT in emit_key_action — VirtualDevice::emit() auto-appends it (Pitfall 6)
- Preflight-before-threads pattern: all fail-hard checks happen before any std::thread::spawn (D-11, D-15)
- about.toml AGPL-3.0-only in accepted list; about.hbs template guards exclude self from LICENSES.md output (D-16)
- PTT thread join is best-effort 500ms timeout — fetch_events blocks; AudioHandle drop stops CPAL stream
- Pin ort to 2.0.0-rc.10 for silero-vad-rust compile stability
- Gate whisper-rs behind opt-in stt feature to keep default cargo test green without cmake
- Moved PTT gating out of CPAL callback so wake word can run without PTT (callback remains allocation-free)
- Dedicated output thread is the only stdout writer (stdout JSONL stays clean; compute threads avoid IO stalls)
- Phase 2 latency proof uses end-of-speech → transcript JSONL emit as the STT-04 proxy; end-of-speech → first key event is validated in Phase 3 dispatch.

### Pending Todos

None yet.

### Blockers/Concerns

- **Phase 2 risk**: whisper-rs Codeberg build reliability on CachyOS needs empirical verification during Phase 2 planning
- **Phase 2 risk**: Silero-VAD ONNX Runtime dylib bundling for AppImage needs testing (ort crate + FUSE interaction)
- **Phase 3 validation**: nProtect GameGuard anti-cheat behavior against uinput virtual device requires live HD2 testing — not resolvable by research alone
- **Phase 4 content**: HD2 stratagem key sequences must be validated in-game before pack ships; wiki/forum sources are MEDIUM confidence
- **Wake-word model**: Specific model not yet selected — sherpa-onnx keyword spotter recommended (ONNX Runtime already in dep tree via Silero-VAD)

## Deferred Items

| Category | Item | Status | Deferred At |
|----------|------|--------|-------------|
| *(none)* | | | |

## Session Continuity

Last session: 2026-04-22T12:38:01.463Z
Stopped at: Completed 02-pipeline-core-04-PLAN.md
Resume file: None

**Planned Phase:** 2 (Pipeline Core) — 4 plans — 2026-04-22T12:03:27.007Z
