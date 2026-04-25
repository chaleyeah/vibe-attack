# Roadmap: hd-linux-voice

## Overview

Build a Rust-based, AGPL-3.0 voice-macro daemon for Linux that lets Helldivers 2 players speak a stratagem name and have the correct key sequence fire in-game — fully local, Wayland-first, < 500 ms end-to-end. Five phases follow the natural dependency graph: plumb audio and input injection first, wire the recognition pipeline second, prove the core value with a live stratagem demo third, productize with a data-driven pack system fourth, and ship with a GUI and distro packaging last.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Foundation** — Audio capture, uinput/evdev injection backend, config store, headless daemon, PTT via evdev grab (completed 2026-04-22)
- [x] **Phase 2: Pipeline Core** — VAD (Silero), STT engine (whisper.cpp), end-to-end latency instrumentation and baseline proof (completed 2026-04-23)
- [ ] **Phase 3: Phrase Matching + Dispatch** — Fuzzy phrase engine, macro dispatcher, conditional logic, sound feedback; first playable HD2 stratagem demo
- [ ] **Phase 4: Pack System + HD2 Bundle** — .hdpack format, all 80+ HD2 stratagems, ProfileManager, import/export, built-in editor
- [ ] **Phase 5: UI + Distribution** — egui config window, system tray, first-run wizard, AppImage, AUR/PKGBUILD
- [ ] **Phase 6: Documentation** — Usage docs, troubleshooting, and contributor guides

## Phase Details

### Phase 1: Foundation
**Goal**: A CLI daemon can capture microphone audio and inject arbitrary key sequences into a Wayland session via uinput, with PTT working correctly even when a fullscreen game holds exclusive compositor focus
**Depends on**: Nothing (first phase)
**Requirements**: ACT-01, MCRO-01, MCRO-02, MCRO-05, UI-01, DIST-03
**Success Criteria** (what must be TRUE):
  1. Holding the PTT key causes audio capture to begin and releasing it stops capture — observable via console log even when a fullscreen Proton/Steam game is foregrounded
  2. A test macro fires a configurable key sequence (with configurable dwell times and inter-key gaps) into a focused Wayland window including fullscreen Steam/Proton games via uinput, including key-hold (press-and-hold for a specified duration)
  3. The daemon starts headless with no mapped window and does not minimize a running fullscreen game when launched or when PTT fires
  4. Startup emits an actionable error and exits immediately if `/dev/uinput` cannot be opened, displaying the exact fix command and a documentation link — no silent failure
  5. All bundled Rust dependencies carry AGPL-3.0–compatible licenses (MIT/Apache-2.0/LGPL); a `LICENSES.md` inventory exists in the repo
**Plans**: 5 plans across 5 sequential waves

Plans:
- [x] 01-01-PLAN.md — Toolchain + Cargo.toml (13 deps) + module skeleton + test stubs (Wave 1) ✓ 2026-04-22
- [x] 01-02-PLAN.md — Config structs + XDG YAML load + clap CLI + tracing init (Wave 2)
- [x] 01-03-PLAN.md — CPAL audio capture (warm PTT-gated stream) + evdev PTT scanner (Wave 3)
- [x] 01-04-PLAN.md — uinput VirtualDevice + injection thread + dwell/gap timing (Wave 4)
- [x] 01-05-PLAN.md — Daemon main loop + SIGTERM handling + LICENSES.md + docs (Wave 5)

### Phase 2: Pipeline Core
**Goal**: Speaking any phrase into the microphone produces a timestamped transcript on stdout, with per-stage latency instrumented and the full pipeline proven to meet the < 500 ms end-to-end budget on target hardware
**Depends on**: Phase 1
**Requirements**: ACT-02, STT-01, STT-04
**Success Criteria** (what must be TRUE):
  1. Speaking a short phrase while PTT is held produces the correct transcript on stdout within 500 ms of the last spoken syllable, as confirmed by the per-stage timestamp log
  2. The timestamp log shows AudioCapture → VAD → STT → output with individual stage costs; no single stage exceeds its sub-budget (target: VAD ≤ 50 ms, STT ≤ 200 ms on reference hardware)
  3. Speaking the configured wake word (without PTT) causes the daemon to enter LISTENING state and print the trigger event to the console — wake word model runs fully on-device
  4. The whisper.cpp `tiny.en` model is loaded into RAM at daemon startup; recognition produces transcripts with no network access at any point during the session
  5. The STT inference runs on a dedicated OS blocking thread (never on the Tokio executor); the audio RT callback never allocates or blocks; concurrent recognition and audio capture are verified via stress test
**Plans**: 6 plans across 6 sequential waves

Plans:
- [x] 02-01-PLAN.md — Wave 0 scaffolding: deps + config schema + env-gated heavy test harnesses (Wave 1) ✓ 2026-04-22
- [x] 02-02-PLAN.md — Pipeline contracts + VAD segmentation: JSONL schema + timing + bounded utterance jobs (Wave 2) ✓ 2026-04-22
- [x] 02-03-PLAN.md — End-to-end wiring: wake word + STT OS threads + ringbuf drain + instrumentation (Wave 3) ✓ 2026-04-22
- [x] 02-04-PLAN.md — Proof artifacts: latency baseline procedure + concurrency stress test + validation bookkeeping (Wave 4) ✓ 2026-04-22
- [x] 02-05-PLAN.md — Gap closure: add `e2e_ms` + `vad_ms` to utterance JSONL schema; update tests + docs (Wave 5) ✓ 2026-04-22
- [x] 02-06-PLAN.md — Gap closure: add in-repo target-hardware proof artifact templates + wire baseline doc (Wave 6) ✓ 2026-04-22

### Phase 3: Phrase Matching + Dispatch
**Goal**: Speaking a recognized stratagem name fires the correct Helldivers 2 key sequence in a live game session — the core value proposition is proven end-to-end
**Depends on**: Phase 2
**Requirements**: STT-02, MCRO-03, MCRO-04
**Success Criteria** (what must be TRUE):
  1. Speaking "Eagle Airstrike" (and minor mispronunciations within the configured confidence threshold) fires the correct HD2 key sequence in a live game session; the stratagem successfully deploys in-game
  2. Transcripts below the confidence threshold are logged as NO_MATCH and do not trigger any macro — threshold is configurable at runtime without restarting the daemon
  3. A macro containing conditional logic (if/else branch on a boolean variable) executes the correct branch; the executed branch is visible in the key-event log
  4. A macro with sound feedback enabled plays the configured audio clip on activation with no perceptible delay between voice recognition and sound playback; disabling sound feedback per-macro silences it without affecting the key sequence
**Plans**: TBD

### Phase 4: Pack System + HD2 Bundle
**Goal**: All Helldivers 2 stratagems are available as a shipped, data-driven pack; users can load, switch, import, export, and edit macros without touching Rust code
**Depends on**: Phase 3
**Requirements**: ACT-03, STT-03, PACK-01, PACK-02, PACK-03, PACK-04, PACK-05
**Success Criteria** (what must be TRUE):
  1. The bundled HD2 pack covers all 80+ current stratagems across all in-game categories; speaking any stratagem name fires the correct validated key sequence in a live game session
  2. Importing a `.hdpack` file from disk loads its phrases and macros into the active profile immediately without restarting the daemon; malformed or checksum-invalid packs are rejected with a clear error message
  3. Exporting the current profile produces a `.hdpack` file that round-trips cleanly — importing the exported file produces identical behavior to the original profile
  4. Switching between named profiles at runtime (via CLI command) flushes the MacroRegistry and reloads the new profile within one second; the active profile name is visible in the daemon status output
  5. Editing an individual macro's phrase, key sequence, delays, conditions, and sound via the built-in editor persists changes to the config file and the daemon reloads them on next startup or hot-reload trigger
  6. Switching between PTT and wake-word activation mode is configurable from the config file and takes effect on daemon restart without other changes
**Plans**: TBD

### Phase 5: UI + Distribution
**Goal**: A new user can download, install, and reach a working voice-stratagem session in Helldivers 2 within minutes — with a config window, system tray, first-run wizard, and distro-agnostic packaging
**Depends on**: Phase 4
**Requirements**: ACT-04, UI-02, UI-03, UI-04, DIST-01, DIST-02
**Success Criteria** (what must be TRUE):
  1. A first-run wizard guides a new user through microphone selection, PTT keybinding, and loading the HD2 pack; the user reaches a working voice-macro session without reading any external documentation
  2. The system tray icon shows the current listening state (idle / listening / muted) and a right-click menu provides mute toggle, profile switch, and open-config-window actions
  3. The config window allows changing audio input device, activation mode (PTT vs wake word), confidence threshold, and keybindings; changes persist and take effect without manually restarting the daemon
  4. Installing on a fresh Arch Linux / CachyOS system via `yay -S hd-linux-voice` (AUR PKGBUILD) sets up udev rules and group membership in the post-install hook; the user can launch the daemon immediately after install without manual privilege steps
  5. Installing on any modern Linux distro via the AppImage + companion `install.sh` configures `/dev/uinput` permissions and verifies microphone access; the install script provides clear remediation if either check fails
**Plans**: TBD
**UI hint**: yes

### Phase 6: Documentation

**Goal**: Provide clear, concise documentation for users and contributors covering installation, configuration, troubleshooting, and development workflows
**Depends on**: Phase 5
**Requirements**: TBD
**Success Criteria** (what must be TRUE):
  1. A new user can install (AUR + AppImage) and reach a working voice-macro session following only repo docs
  2. Common failures (uinput permissions, input group membership, mic device selection, Wayland focus edge cases) have troubleshooting steps with copy/paste commands
  3. Developer docs cover local build/test, feature flags (e.g. `stt`), and release packaging steps
**Plans**: 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 6 to break down)

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation | 5/5 | Complete    | 2026-04-22 |
| 2. Pipeline Core | 6/6 | Complete | 2026-04-23 |
| 3. Phrase Matching + Dispatch | 0/? | Not started | - |
| 4. Pack System + HD2 Bundle | 0/? | Not started | - |
| 5. UI + Distribution | 0/? | Not started | - |
| 6. Documentation | 0/0 | Not started | - |

### Phase 7: Rebrand from hd-linux-voice to vibe-attack

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 6
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 7 to break down)

### Phase 8: Fix compilation errors from dependency updates and API mismatches

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 7
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 8 to break down)
