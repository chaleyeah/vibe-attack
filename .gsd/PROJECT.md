# hd-linux-voice

## What This Is

An **open source** Linux desktop application in the spirit of [VoiceAttack](https://voiceattack.com/) that binds spoken phrases to keyboard and mouse macros so players can trigger in-game actions by voice. The first concrete target is **Helldivers 2**: ship a maintained set of **strategem** voice commands that map to the correct key sequences, usable during live gameplay. The project is licensed under **GNU AGPL v3.0** (see `LICENSE`); distribution is aimed at a **small release** (other users installing and running it), not a single-developer-only script.

## Core Value

**During Helldivers 2 gameplay, the player can fire the right strategem reliably by voice with minimal delay and without breaking flow** — wake word or push-to-talk, fully **local** speech processing, **Wayland-first** input delivery.

## Current State (2026-04-25)

**M001 Migration — in progress**

| Slice | Status | Delivered |
|-------|--------|-----------|
| S01: Foundation | ✅ complete | Rust toolchain, compilable project skeleton, Cargo.toml |
| S02: Pipeline Core | ✅ complete | VAD/STT/wake-word config scaffolding, opt-in heavy tests |
| S03: Phrase Matching Dispatch | ✅ complete | Unit tests proving phrase-matching-dispatch works |
| S04: Pack System HD2 Bundle | ✅ complete | 22 hermetic integration tests: YAML round-trip, ZIP export/import, ProfileManager, full lifecycle |
| S05: UI + Distribution | ⬜ pending | egui config window, system tray, first-run wizard, AppImage, AUR/PKGBUILD |
| S06: Documentation | ⬜ pending | Usage docs, troubleshooting, contributor guides |
| S07: Wake Word Activation | ⬜ deferred | Blocked on dual ORT conflict (sherpa-onnx static vs ort crate dynamic) |

**Next up: S05 — UI + Distribution**

## Requirements

### Validated

- ACT-01: Push-to-talk via evdev — Validated in Phase 1 (Foundation)
- MCRO-01: Key sequences with configurable inter-key delays — Validated in Phase 1 (Foundation)
- MCRO-02: Key/button holds (press-and-hold dwell) — Validated in Phase 1 (Foundation)
- MCRO-05: uinput/evdev key events on Wayland — Validated in Phase 1 (Foundation)
- UI-01: Headless daemon mode — Validated in Phase 1 (Foundation)
- DIST-03: AGPL-3.0 license + LICENSES.md inventory — Validated in Phase 1 (Foundation)

### Active

- [ ] User can arm listening with a **programmable wake keyword** or a **push-to-talk** control.
- [ ] Recognition runs **fully on-device** (no cloud requirement for core gameplay).
- [ ] User can run **voice-bound macros** that emit keyboard/mouse events suitable for Helldivers 2 strategems on **Wayland** (primary target).
- [ ] Strategem coverage for Helldivers 2 is **data-driven** and **easy to update** when the game adds or changes strategems.
- [ ] **Both**: importable/versioned **packs** for bulk updates and a **built-in editor** for phrases, bindings, and timing tweaks.
- [ ] Long-term parity with **VoiceAttack-style** power (conditions, variables, multiple profiles, etc.), delivered in **phases** — v1 may scope a thin subset if needed, but architecture should not block the full vision.
- [ ] Packaging and UX suitable for **other users** installing the app (clear defaults, recovery paths, documentation), not only the author's machine.

### Out of Scope

- **Windows / macOS** clients in v1 (Linux-only focus; other OS may be future work).
- **Cloud-hosted** speech recognition as a **required** path for core play (optional pluggable backends may be considered later; v1 core path is local-only per decision).

## Context

- **Inspiration:** VoiceAttack on Windows — phrase recognition, macro scripting, profiles. This project targets **feature depth over time**, not a one-off script.
- **Game focus:** Helldivers 2 strategems are the primary driver for v1 content and for proving latency, accuracy, and input correctness under pressure.
- **Platform:** Primary display server is **Wayland**; keyboard/mouse injection and focus behavior must be validated on target distros. **X11** may follow; design should isolate input backends.
- **Implementation language:** **Rust** — performance, single-binary deployment, strong systems fit for audio + input, AGPL-compatible ecosystem.
- **Licensing:** Project is **AGPL-3.0**; third-party libraries must be compatible with AGPL distribution.
- **Risks:** Game **anti-cheat / input policies**, exclusive fullscreen vs windowed behavior, audio device contention, achievable **end-to-end latency**, and the **dual ORT conflict** (sherpa-onnx static vs ort crate dynamic — deferred to S07/Phase 6.5).

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
| Packs + built-in editor for strategems | Bulk updates + user tweaks without blocking each other | Active |
| Open source under **AGPL-3.0** | User correction; aligns with `LICENSE` | Active |
| Long-term VoiceAttack-class depth | User ambition; phased delivery to reduce v1 risk | Active |
| Wake word path disabled (Phase 6.5 deferral) | Dual ORT conflict: sherpa-onnx static ORT + ort crate dynamic ORT causes bad_alloc heap corruption | Deferred to S07 |
| Rust as implementation language | Single-binary deployment, systems-level audio+input, AGPL-compatible ecosystem | Confirmed in S01 |

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
*Last updated: 2026-04-25 — S04 (Pack System HD2 Bundle) complete: 22 hermetic integration tests for pack lifecycle*
