# Requirements: hd-linux-voice

**Defined:** 2026-04-21
**Core Value:** During Helldivers 2 gameplay, the player can fire the right strategem reliably by voice with minimal delay and without breaking flow.

---

## v1 Requirements

### Listening Activation

- [x] **ACT-01
**: User can activate listening via a configurable **push-to-talk** key/button (evdev grab, works on Wayland)
- [ ] **ACT-02**: User can activate listening via a configurable **wake word** (on-device model, no cloud)
- [ ] **ACT-03**: User can switch between PTT and wake-word mode from the config UI at any time
- [ ] **ACT-04**: A visible status indicator (tray icon) reflects current listening state (idle / listening / muted)

### Speech Recognition

- [ ] **STT-01**: Speech recognition runs **fully on-device** using a bundled local model (whisper.cpp or equivalent)
- [ ] **STT-02**: Phrase matching uses a **confidence threshold** (fuzzy matching) to handle slight mispronunciations
- [ ] **STT-03**: User can configure the confidence threshold from the config UI
- [x] **STT-04
**: Recognition pipeline achieves **< 500 ms** end-to-end latency (end of speech → first key event) on target hardware

### Macro Engine

- [x] **MCRO-01
**: Macros execute **key sequences** with configurable inter-key delays
- [x] **MCRO-02
**: Macros support **key/button holds** (press and hold for a specified duration) — required for HD2 directional inputs
- [ ] **MCRO-03**: Macro engine supports **conditional logic** (if/else, variables) for VoiceAttack-class scripting
- [ ] **MCRO-04**: Macros play an optional **sound feedback** on activation (configurable per macro or globally)
- [x] **MCRO-05
**: All key/mouse events are emitted via **uinput/evdev** and function correctly in Wayland sessions (including fullscreen games)

### Strategem Pack System

- [ ] **PACK-01**: App ships with a **bundled Helldivers 2 pack** covering all current stratagems (80+) with correct key sequences
- [ ] **PACK-02**: User can **import packs** from versioned `.hdpack` files (JSON or YAML format with checksum)
- [ ] **PACK-03**: User can **export packs** to `.hdpack` files for sharing or backup
- [ ] **PACK-04**: User can create and edit macros via a **built-in editor** (phrase, key sequence, delays, conditions, sound)
- [ ] **PACK-05**: App supports **multiple named profiles** (e.g. one per game or playstyle), switchable at runtime

### UI and Configuration

- [x] **UI-01
**: App runs as a **headless daemon** by default (no window required for core gameplay)
- [ ] **UI-02**: A **system tray icon** provides quick access to status and controls (mute, profile switch, open config)
- [ ] **UI-03**: A **config window** allows setting audio input device, activation mode, confidence threshold, and keybindings
- [ ] **UI-04**: A **first-run wizard** guides new users through microphone test, PTT binding, and loading the HD2 pack

### Distribution

- [ ] **DIST-01**: App is packaged as an **AppImage** for distro-agnostic install
- [ ] **DIST-02**: App provides an **AUR / PKGBUILD** for Arch Linux and CachyOS users
- [x] **DIST-03
**: App is licensed **AGPL-3.0**; all bundled dependencies are AGPL-compatible (MIT/Apache-2.0/LGPL)

---

## v2 Requirements

### Advanced Macro Engine

- **MCRO-V2-01**: Mouse movement and click macros
- **MCRO-V2-02**: Full looping and timing constructs (wait, repeat N times)
- **MCRO-V2-03**: Inter-macro communication (call another macro by name)

### Speech Backends

- **STT-V2-01**: Pluggable STT backend interface so users can swap engines
- **STT-V2-02**: Multi-language recognition support
- **STT-V2-03**: User-supplied custom recognition model

### Pack Ecosystem

- **PACK-V2-01**: Auto-update packs from a remote URL (opt-in)
- **PACK-V2-02**: Community pack repository / discovery

### Platform

- **PLAT-V2-01**: X11 input injection backend (in addition to uinput/Wayland path)
- **PLAT-V2-02**: Flatpak packaging

---

## Out of Scope

| Feature | Reason |
|---------|--------|
| Windows / macOS clients | Linux-only focus for v1; cross-platform may be considered in future milestones |
| Cloud speech recognition (required path) | Privacy requirement; local-only is the core constraint |
| Open API / hosted backend | Not part of the product model |
| Real-time game state reading (memory scanning) | Out of scope; input injection only |
| GUI-only (no daemon mode) | Daemon-first is an architectural requirement for Wayland focus correctness |

---

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| ACT-01 | Phase 1 | Complete |
| ACT-02 | Phase 2 | Pending |
| ACT-03 | Phase 4 | Pending |
| ACT-04 | Phase 5 | Pending |
| STT-01 | Phase 2 | Pending |
| STT-02 | Phase 3 | Pending |
| STT-03 | Phase 4 | Pending |
| STT-04 | Phase 2 | Pending |
| MCRO-01 | Phase 1 | Complete |
| MCRO-02 | Phase 1 | Complete |
| MCRO-03 | Phase 3 | Pending |
| MCRO-04 | Phase 3 | Pending |
| MCRO-05 | Phase 1 | Complete |
| PACK-01 | Phase 4 | Pending |
| PACK-02 | Phase 4 | Pending |
| PACK-03 | Phase 4 | Pending |
| PACK-04 | Phase 4 | Pending |
| PACK-05 | Phase 4 | Pending |
| UI-01 | Phase 1 | Complete |
| UI-02 | Phase 5 | Pending |
| UI-03 | Phase 5 | Pending |
| UI-04 | Phase 5 | Pending |
| DIST-01 | Phase 5 | Pending |
| DIST-02 | Phase 5 | Pending |
| DIST-03 | Phase 1 | Complete |

**Coverage:**
- v1 requirements: 25 total
- Mapped to phases: 25
- Unmapped: 0 ✓

---
*Requirements defined: 2026-04-21*
*Last updated: 2026-04-21 after initial definition*
