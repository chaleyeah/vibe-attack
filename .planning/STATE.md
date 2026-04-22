# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-04-21)

**Core value:** During Helldivers 2 gameplay, the player can fire the right stratagem reliably by voice with minimal delay and without breaking flow — wake word or push-to-talk, fully local speech processing, Wayland-first input delivery.
**Current focus:** Phase 1 — Foundation

## Current Position

Phase: 1 of 5 (Foundation)
Plan: 0 of ? in current phase
Status: Ready to plan
Last activity: 2026-04-21 — Roadmap created; research complete; ready to plan Phase 1

Progress: [░░░░░░░░░░] 0%

## Performance Metrics

**Velocity:**
- Total plans completed: 0
- Average duration: —
- Total execution time: 0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| - | - | - | - |

**Recent Trend:**
- Last 5 plans: —
- Trend: —

*Updated after each plan completion*

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- Pre-Phase 1: PTT must use evdev EVIOCGRAB on physical device (not compositor global hotkey) — Wayland fullscreen focus kills compositor-level shortcuts
- Pre-Phase 1: Daemon is headless by default; config UI is a separate process — prevents XWayland focus race minimizing fullscreen game
- Pre-Phase 1: No Flatpak packaging — uinput/evdev incompatible with Flatpak sandbox; AppImage + AUR are primary targets
- Pre-Phase 1: STT never runs on Tokio executor; audio RT callback never allocates — latency invariant from day one
- Pre-Phase 1: Phase 2 is highest technical risk — latency must be proven before Phase 3 dispatch work begins

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

Last session: 2026-04-21
Stopped at: Roadmap created; ROADMAP.md, STATE.md, and REQUIREMENTS.md written
Resume file: None
