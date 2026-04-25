# hd-linux-voice

## What This Is

An **open source** Linux desktop application in the spirit of [VoiceAttack](https://voiceattack.com/) that binds spoken phrases to keyboard and mouse macros so players can trigger in-game actions by voice. The first concrete target is **Helldivers 2**: ship a maintained set of **stratagem** voice commands that map to the correct key sequences, usable during live gameplay. The project is licensed under **GNU AGPL v3.0** (see `LICENSE`); distribution is aimed at a **small release** (other users installing and running it), not a single-developer-only script.

## Core Value

**During Helldivers 2 gameplay, the player can fire the right stratagem reliably by voice with minimal delay and without breaking flow** — wake word or push-to-talk, fully **local** speech processing, **Wayland-first** input delivery.

## Current State (2026-04-25)

**M001 Migration — complete ✅**

| Slice | Status | Delivered |
|-------|--------|-----------|
| S01: Foundation | ✅ complete | Rust toolchain, compilable project skeleton, Cargo.toml, LICENSES.md |
| S02: Pipeline Core | ✅ complete | VAD/STT/wake-word config + test scaffolding, JSONL event schema, latency baseline |
| S03: Phrase Dispatch | ✅ complete | PhraseMatcher + Dispatcher, 31 tests (lib + dispatcher_logic + jsonl_schema) |
| S04: Pack System HD2 Bundle | ✅ complete | 22 hermetic integration tests: YAML round-trip, ZIP export/import, ProfileManager, full lifecycle |
| S05: UI + Distribution | ✅ complete | Pure-logic UI state, feature-gated egui binary, PKGBUILD, AppImage scaffolding, 16 tests |
| S06: Documentation | ✅ complete | README rewrite, CONTRIBUTING.md, troubleshooting, configuration docs, 11 structural tests |
| S07: Wake Word ORT | ✅ complete | Shared ORT linking for sherpa-onnx, ORT_DYLIB_PATH auto-discovery, coexistence test |

**Next milestone: TBD — system tray, full config window, AppImage build, AUR submission, runtime CI confirmation**

## Requirements

### Validated

- ACT-01: Push-to-talk via evdev — Validated in M001/S01 (Phase 1 Foundation)
- MCRO-01: Key sequences with configurable inter-key delays — Validated in M001/S01 (Phase 1 Foundation)
- MCRO-02: Key/button holds (press-and-hold dwell) — Validated in M001/S01 (Phase 1 Foundation)
- MCRO-05: uinput/evdev key events on Wayland — Validated in M001/S01 (Phase 1 Foundation)
- UI-01: Headless daemon mode — Validated in M001/S01 (Phase 1 Foundation)
- DIST-03: AGPL-3.0 license + LICENSES.md inventory — Validated in M001/S01 (Phase 1 Foundation)

### Advanced (structural foundation complete, runtime validation pending)

- PACK-01: HD2 pack bundle — 22 tests prove lifecycle; runtime CI confirmation pending
- UI-04: First-run wizard — FirstRunState struct models wizard state machine; GUI integration not yet built
- DIST-01: AppImage — build script scaffolded; actual AppImage build not yet run
- DIST-02: AUR/PKGBUILD — PKGBUILD present; AUR submission not yet done

### Active

- [ ] ACT-03: User can switch between PTT and wake-word mode from the config UI at any time
- [ ] ACT-04: A visible status indicator (tray icon) reflects current listening state (idle / listening / muted)
- [ ] STT-02: Phrase matching uses a **confidence threshold** (fuzzy matching) to handle slight mispronunciations
- [ ] STT-03: User can configure the confidence threshold from the config UI
- [ ] MCRO-03: Macro engine supports **conditional logic** (if/else, variables) for VoiceAttack-class scripting
- [ ] MCRO-04: Macros play an optional **sound feedback** on activation (configurable per macro or globally)
- [ ] PACK-02: User can **import packs** from versioned `.hdpack` files (JSON or YAML format with checksum)
- [ ] PACK-03: User can **export packs** to `.hdpack` files for sharing or backup
- [ ] PACK-04: User can create and edit macros via a **built-in editor** (phrase, key sequence, delays, conditions, sound)
- [ ] PACK-05: App supports **multiple named profiles** (e.g. one per game or playstyle), switchable at runtime
- [ ] UI-02: A **system tray icon** provides quick access to status and controls (mute, profile switch, open config)
- [ ] UI-03: A **config window** allows setting audio input device, activation mode, confidence threshold, and keybindings

### Out of Scope

- **Windows / macOS** clients in v1 (Linux-only focus; other OS may be future work).
- **Cloud-hosted** speech recognition as a **required** path for core play (optional pluggable backends may be considered later; v1 core path is local-only per decision).

## Context

- **Inspiration:** VoiceAttack on Windows — phrase recognition, macro scripting, profiles. This project targets **feature depth over time**, not a one-off script.
- **Game focus:** Helldivers 2 stratagems are the primary driver for v1 content and for proving latency, accuracy, and input correctness under pressure.
- **Platform:** Primary display server is **Wayland**; keyboard/mouse injection and focus behavior must be validated on target distros. **X11** may follow; design should isolate input backends.
- **Implementation language:** **Rust** — performance, single-binary deployment, strong systems fit for audio + input, AGPL-compatible ecosystem.
- **Licensing:** Project is **AGPL-3.0**; third-party libraries must be compatible with AGPL distribution.
- **Risks:** Game **anti-cheat / input policies**, exclusive fullscreen vs windowed behavior, audio device contention, achievable **end-to-end latency**. The dual ORT conflict (sherpa-onnx static vs ort crate dynamic) was **resolved in M001/S07** via shared ORT linking.

## Constraints

- **License**: **AGPL-3.0** — third-party speech, UI, and model dependencies must be **AGPL-compatible**.
- **Privacy / offline**: Core path **local-only** recognition; no dependency on cloud for default gameplay.
- **Display**: **Wayland-first** implementation and testing.
- **Distribution**: "Small release" — installer expectations, sane defaults, and supportability matter from early milestones.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Local-only speech for core path | Privacy, offline play, predictable latency | Active |
| Wayland-first input | Matches stated primary environment | Active |
| Packs + built-in editor for stratagems | Bulk updates + user tweaks without blocking each other | Active |
| Open source under **AGPL-3.0** | User correction; aligns with `LICENSE` | Active |
| Long-term VoiceAttack-class depth | User ambition; phased delivery to reduce v1 risk | Active |
| Rust as implementation language | Single-binary deployment, systems-level audio+input, AGPL-compatible ecosystem | Confirmed in M001/S01 |
| Shared ORT linking for sherpa-onnx | Resolves dual-ORT bad_alloc: sherpa-onnx shared feature + ORT_DYLIB_PATH auto-discovery | Resolved in M001/S07 |
| Feature-gated GUI binary (required-features = ["gui"]) | Keeps daemon headless by default; no runtime cfg guards needed | Established in M001/S05 |
| Stdout reserved for JSONL only | Composable with tooling; tracing/stderr for all instrumentation | Established in M001/S02 |
| Dispatcher injected writer | Box<dyn Write + Send> makes dispatch testable without display server or stdout capture | Established in M001/S03 |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):

1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):

1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-25 — M001 (Migration) complete: 7 slices, 80+ tests, full pipeline from audio capture through phrase dispatch, pack system, UI scaffolding, documentation, and dual-ORT conflict resolution*
