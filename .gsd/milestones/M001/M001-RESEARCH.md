# Project Research Summary

**Project:** hd-linux-voice — Linux voice-macro application (VoiceAttack-class)
**Domain:** Real-time voice-driven gaming macro engine — Wayland, local STT, Helldivers 2 target
**Researched:** 2026-04-21
**Confidence:** MEDIUM-HIGH (stack and architecture HIGH; anti-cheat risk MEDIUM)

---

## Executive Summary

hd-linux-voice is a real-time voice-to-keypress pipeline that must meet hard gaming latency requirements (< 500 ms end-of-speech → injected key event) while running fully locally on Wayland. The authoritative prior art is VoiceAttack (Windows-only, proprietary), which no mature open-source Linux equivalent yet matches in reliability. The closest Linux attempts — LinVAM, Vocalinux, Vokey — prove the demand but expose painful gaps: X11-only injection, Python GIL jitter, or abandoned maintenance. The correct build path is a Rust binary using whisper.cpp for recognition, Silero-VAD for gating, cpal for audio capture, uinput/evdev for kernel-level input injection, and egui for configuration UI — all MIT/Apache-2.0 licensed and fully AGPL-3.0 compatible.

The single sharpest risk is Wayland's security model, which creates two overlapping failure modes that are not obvious until in-game testing: global hotkeys silently stop working when a fullscreen game holds focus (requiring evdev grab on the physical device rather than any compositor-level API), and daemon UI windows can trigger XWayland focus races that minimize the game. Both require architectural decisions made at Phase 1, not retrofitted later. The second most dangerous risk is cumulative pipeline latency — each stage looks acceptable in isolation, but VAD silence tail (250 ms) + Whisper inference (150 ms) + buffer fill already consumes the full latency budget before macro timing is added. Instrumentation must be built into the pipeline from day one.

The project has a clear v1 scope that delivers genuine user value: speak a stratagem name → correct key sequence fires in Helldivers 2. The bundled HD2 strategem pack (80+ stratagems) is the flagship differentiator. VoiceAttack-class depth (conditions, variables, scripting) belongs in v2+; building it in v1 would sink the timeline. The architecture correctly mirrors this by separating data-driven TOML packs from the engine — community members can ship and maintain packs without touching Rust code, which is the right model for AGPL ecosystem growth.

---

## Key Findings

### Recommended Stack

The full stack is Rust 1.85+ throughout. whisper.cpp (MIT) via the `whisper-rs` crate (Unlicense) provides speech recognition; the `tiny.en` model (~39 MB) delivers ~32× real-time on CPU, giving < 200 ms inference for short gaming commands. Silero-VAD via `silero-vad-rust` (MIT, v6.2.1) gates audio segments before whisper.cpp, eliminating continuous processing. cpal v0.17+ (Apache-2.0) handles audio capture across PipeWire, ALSA, and PulseAudio through a unified interface — PipeWire native support merged January 2026 and is now the default on modern Wayland distros. Input injection uses the `evdev` crate (MIT) against `/dev/uinput` — kernel-level, no daemon, works with Steam/Proton below the Wayland security boundary. egui/eframe v0.34 (MIT/Apache-2.0) provides a self-contained UI with no system GTK runtime dependency, which is essential for AppImage distribution. All dependencies are AGPL-3.0 compatible.

**Core technologies:**

| Layer | Choice | Why |
|-------|--------|-----|
| Language | Rust 1.85+ | No GC pauses in audio RT thread; single binary; deterministic latency |
| STT | whisper.cpp via `whisper-rs` | MIT license, 32× RT on CPU with tiny.en, < 200ms inference |
| VAD | `silero-vad-rust` v6.2.1 | MIT, bundled ONNX model, 512-sample chunks, no download step |
| Audio capture | `cpal` v0.17 | Unified PipeWire/ALSA/PulseAudio abstraction; PipeWire native since Jan 2026 |
| Input injection | `evdev` crate → `/dev/uinput` | Kernel-level; works with Proton/Steam; no daemon; survives compositor focus policies |
| UI | `egui` / `eframe` v0.34 | Self-contained binary; Wayland native; no system GTK runtime dep |
| Config | TOML + `serde` | Human-editable; standard Rust community pattern |
| Async runtime | `tokio` | mpsc channels between pipeline stages; `spawn_blocking` for STT |
| Packaging | AppImage primary, AUR/PKGBUILD for CachyOS | Lowest friction; no Flatpak (uinput/evdev incompatible with sandbox) |

**STT fallback:** Vosk via `vosk-rs` (Apache-2.0) for very low-end CPUs (< 4-core / 4 GB RAM); continuous streaming, lower accuracy, no end-of-utterance wait needed.

**Vosk grammar constraint:** For both engines, restrict to the recognized phrase vocabulary (Vosk `SetGrammar`, Whisper `initial_prompt`). Open-vocabulary transcription mis-recognizes gaming jargon; constrained vocab boosts accuracy and cuts inference time.

### Expected Features

**Must have — table stakes (v1 blocks on these):**

- Push-to-talk activation (configurable hotkey; evdev grab on physical device — not compositor global shortcut)
- Wake-word / listen-toggle gate (decoupled from PTT; both modes simultaneously available)
- Local speech recognition, model hot in RAM (no cloud; offline play must work)
- Exact phrase → key-sequence macro execution with configurable inter-key timing
- Wayland input injection via uinput (Steam/Proton compatible)
- Profile system (named command sets; one active at a time)
- Recognition log / feedback (terminal output acceptable in v1)
- Mic level indicator
- Importable/exportable command packs (.hdpack format)
- Bundled HD2 stratagem pack (~80 stratagems, all categories, versioned)
- Confidence threshold (global; per-command in v2)

**Should have — differentiators (v1 scope, minimal implementation acceptable):**

- Built-in phrase editor (CLI-based in v1; GUI in v2)
- Command chaining / sub-commands (HD2 profiles use open-menu → sequence → close-menu pattern)
- Configurable key dwell times and inter-key gaps per command (critical for stratagem reliability)
- Conflict detection on phrase authoring (warn when two phrases share a prefix)

**Defer to v2+:**

- Optional/dynamic phrase segments (`[Orbital;] Airstrike` syntax)
- Conditions, variables, if-then logic
- Auto profile switching by window focus
- TTS confirmation feedback, soundboard
- GUI phrase editor
- Multiple STT backends (Vosk swappable for Whisper)
- Plugin/extension API

**Anti-features (explicitly excluded from v1):**

- Natural language / LLM disambiguation (latency 300–2000 ms; game input window is < 100 ms)
- Cloud speech recognition as primary path
- Windows/macOS support
- Auto-update / OTA pack fetching (CDN ops complexity)
- Gamepad/joystick voice-combo triggers

**HD2 pack requirements:** Each stratagem entry needs `id`, `phrase[]` (multiple spoken variants), `key_sequence[]`, `inter_key_delay_ms`, `category`, `description`. Pack must be versioned (`pack_version` + `game_version`). Key sequence notes from VoiceAttack HD2 community: remap strategem directionals to arrow keys in-game; use PRESS mode not HOLD; 80–120 ms dwell time per key; 30–50 ms inter-key gap.

### Architecture Approach

The system is a **two-stage, channel-connected pipeline** running on a shared Tokio async runtime plus two dedicated OS threads. The audio RT thread runs at elevated priority with a lock-free ring buffer — it never allocates and never blocks. Stage 1 (idle) runs a lightweight wake-word detector against the ring buffer consuming < 5% CPU; Stage 2 (active) triggers Silero-VAD to segment the utterance and passes the segment to whisper.cpp running in `spawn_blocking`. A typed async channel connects every stage: AudioCapture → WakeWordDetector → VAD → STT → PhraseEngine → MacroDispatcher → InputBackend. No stage calls another synchronously except PhraseEngine (pure/sync, negligible cost). This channel-only contract means any stage can be mocked in isolation.

**Major components and build order:**

| Level | Components | Deliverable |
|-------|-----------|-------------|
| 0 — Foundation | AudioCapture, UinputBackend, ConfigStore | Capture audio + inject keystrokes; CLI only |
| 1 — Pipeline core | WakeWordDetector, VAD, STT Engine | Speak phrase → transcript printed; latency benchmarkable |
| 2 — Dispatch | PhraseEngine, MacroRegistry, MacroDispatcher | Speak stratagem → key sequence fires in-game; first playable |
| 3 — Pack system | PackLoader, ProfileManager, LibeiBackend | Install .hdpack → phrases/macros load automatically |
| 4 — UI + distribution | egui UI, hot-reload, AppImage packaging | Installable app with pack editor |
| 5+ — VA parity | Conditions, variables, plugin ABI | Future milestones |

**Seven architecture invariants that must hold from Phase 1:**
1. Audio RT thread never blocks; no allocations in CPAL callback
2. STT never runs on the Tokio executor — always `spawn_blocking`
3. `InputBackend` is a trait from day one; UinputBackend ships first, LibeiBackend later
4. Packs are data only (TOML) — no scripting, no eval; keeps AGPL compliance clean
5. One active profile at a time; switch flushes MacroRegistry and reloads
6. WakeWordDetector paused via `AtomicBool` during STT to prevent re-trigger on audio bleed
7. Channels are the only inter-component API; no direct method calls across component boundaries

**Listening state machine:** IDLE → (wake event or PTT) → LISTENING → (VAD silence 250 ms or PTT release) → RECOGNIZING (`spawn_blocking`) → MATCHING → EXECUTING or NO_MATCH → IDLE. PTT held bypasses WakeWordDetector entirely and drives VAD directly.

### Critical Pitfalls

**🔴 Critical — rewrite risk or blocks core journey:**

1. **Wayland global hotkey dead zone (P-01)** — compositor APIs stop delivering key events when a fullscreen game holds focus. *Prevention:* Read the physical keyboard device via `/dev/input/eventN` with `EVIOCGRAB` exclusive grab. Re-inject non-PTT keys via a separate uinput pass-through device. This must be the PTT design from Phase 1; retrofitting is expensive.

2. **uinput inaccessible by default (P-02)** — `/dev/uinput` is root-owned `crw-------`. Silent failure: app recognizes speech, nothing happens in game. *Prevention:* Ship udev rules + `modules-load.d/uinput.conf` in every package post-install hook. Startup check fails fast with an actionable error (exact fix command + link to docs) when `/dev/uinput` can't be opened. Never continue silently.

3. **XWayland focus race minimizes game (P-04)** — Any UI window mapped by the daemon (tray icon tooltip, notification, config window) can trigger a compositor focus event that self-minimizes a fullscreen XWayland game. *Prevention:* Daemon is headless by default — no tray icon, no persistent mapped windows. Config UI is a separate process launched/closed explicitly. Use `gamescope` as game compositor layer for full isolation.

4. **Flatpak sandbox incompatible with uinput + evdev (P-03)** — Flatpak cannot grant `/dev/uinput` write access or `/dev/input/*` exclusive read without a de-facto sandbox escape. *Prevention:* Do not use Flatpak as primary distribution. Ship `.deb`/`.rpm` (post-install script handles udev/groups) and AppImage with a companion `install.sh` for privileged setup. Flatpak only after XDG portals add uinput support.

5. **End-to-end latency pipeline blowout (P-05)** — Individual stages look fast; combined budget (audio buffer 50 ms + VAD silence tail 250 ms + Whisper inference 150 ms) already fills 450 ms before macro timing. *Prevention:* Instrument every stage with timestamps from day one. Hard budget: **< 300 ms from end of last spoken syllable to first injected key event** in default configuration. Use `tiny.en`, greedy decoding (no beam search), and short max-audio-duration. Tune VAD aggressiveness down.

**🟠 High — painful if not addressed early:**

6. **Gaming vocabulary misrecognition (P-06)** — Open-vocabulary ASR mis-hears stratagem names. *Prevention:* Constrain to recognized phrase list (Vosk `SetGrammar`; Whisper `initial_prompt`). Add post-recognition fuzzy matcher against MacroRegistry. Log recognized-vs-matched for debug.

7. **Stratagem timing too fast for game input buffer (P-10)** — Machine-speed injection misses inputs sampled at 60 Hz (16.6 ms/frame). *Prevention:* Default dwell time 80–120 ms per directional key; 30–50 ms inter-key gap; both configurable per-command and per-profile.

8. **uinput EV_REP double-fire (P-09)** — If `EV_REP` capability is set on the virtual device, kernel generates repeat events on top of userspace's. *Prevention:* Omit `EV_REP` from capability bitmask on device creation. Send only `KEY_PRESS` + `KEY_RELEASE` + `SYN_REPORT`; never `KEY_REPEAT` from userspace.

9. **Anti-cheat flags virtual device (P-08)** — Helldivers 2 uses nProtect GameGuard (userspace under Proton), currently permissive toward Linux uinput. Community reports indicate no bans, but this is a soft guarantee. *Prevention:* Run injection as regular user (never root); use a neutral virtual device name; build in realistic dwell times; include a "tested game versions" list in docs; design InputBackend so injection can be disabled without stopping recognition.

---

## Implications for Roadmap

Based on combined research, the architecture's Level 0–4 build order maps directly to roadmap phases. Each phase has a concrete, playable deliverable. Dependencies flow strictly downward — no phase requires skipping ahead.

### Phase 1: Foundation — Audio Capture + Input Injection + Config

**Rationale:** Everything else in the pipeline depends on audio in and keystrokes out. AudioCapture, UinputBackend, and ConfigStore have zero dependencies on each other and can be built in parallel. This phase is also where the two most critical architectural decisions (evdev PTT grab, headless daemon) must be locked in — retrofitting either later is a significant rework.

**Delivers:** CLI daemon that captures microphone audio and injects keystrokes via uinput. No recognition yet. Includes udev rules, post-install hook, startup permission check, and `modules-load.d` persistence.

**Addresses features:** Wayland input injection, configurable key-sequence timing (infrastructure), mic level indicator (raw RMS from audio buffer).

**Avoids pitfalls:** P-01 (evdev PTT grab designed in), P-02 (uinput permission bootstrap), P-03 (no Flatpak), P-04 (headless daemon from day one), P-09 (EV_REP omitted at device creation), P-14 (no ydotool/wtype).

**Research flag:** Standard patterns — evdev/uinput API is well-documented. Skip research-phase.

---

### Phase 2: Pipeline Core — VAD + STT Engine + Latency Baseline

**Rationale:** With audio capture running, the recognition pipeline (VAD → STT) can be wired up. This is the highest-risk technical phase — whisper.cpp FFI build configuration, Silero-VAD ONNX Runtime bundling, Tokio/blocking thread interaction, and latency profiling all happen here. Discovering a blocking architectural issue here (e.g., latency > 500 ms on target hardware) is recoverable; discovering it in Phase 3+ is not.

**Delivers:** Speak a phrase → transcript printed to terminal. Latency measured at every stage with a timestamp log. whisper.cpp `tiny.en` model hot in RAM at startup.

**Addresses features:** Local speech recognition (offline, on-device), model hot in RAM.

**Avoids pitfalls:** P-05 (instrumentation and latency budget enforced here), P-06 (phrase list constraint applied at this stage), P-07 (VAD noise gate, monitor source exclusion), P-11 (CPAL abstraction; device by name not index), P-16 (dedicated audio RT thread, ring buffer, separate STT blocking thread).

**Research flag:** whisper-rs Codeberg integration may need empirical build testing on CachyOS. Silero-VAD ONNX Runtime bundling (`ort` crate + dylib) needs verification. **Recommend research-phase for this phase.**

---

### Phase 3: Phrase Matching + Macro Dispatch — First Playable

**Rationale:** With transcripts available, wiring PhraseEngine → MacroRegistry → MacroDispatcher converts the pipeline into a working macro engine. This phase produces the first "speak Eagle Airstrike → correct key sequence fires in-game" demo — the core value proposition is proven here.

**Delivers:** Hardcoded test macro set with exact phrase matching. Voice command fires the correct Helldivers 2 stratagem in-game. Configurable inter-key dwell times validated against live game.

**Addresses features:** Exact phrase → key-sequence execution, configurable inter-key timing, PTT gate (both modes), recognition log (terminal).

**Avoids pitfalls:** P-06 (exact + normalized + fuzzy matcher chain), P-08 (neutral virtual device name, realistic dwell times), P-10 (80–120 ms dwell, 30–50 ms gap defaults, configurable per-command).

**Research flag:** Standard patterns for string matching. Anti-cheat behavior requires empirical testing against live HD2 instance — **flag as validation task, not research-phase**.

---

### Phase 4: Pack System + HD2 Bundle + Profiles

**Rationale:** The hardcoded macros from Phase 3 become a data-driven pack system. This phase unlocks the bundled HD2 stratagem pack (80+ stratagems), the pack import/export format, the profile system, and conflict detection. It is what transforms a proof-of-concept into a product users can install and configure.

**Delivers:** `.hdpack` format implemented. Bundled HD2 pack ships all ~80 stratagems covering all categories. PackLoader with inotify hot-reload. ProfileManager with load/switch. Pack import from file.

**Addresses features:** HD2 stratagem pack (flagship differentiator), importable/exportable packs, profile concept, command chaining (sub-commands at pack level).

**Avoids pitfalls:** P-06 (pack includes multiple phrase variants per stratagem to handle recognition variants like "Eagle Air Strike" vs "Eagle Airstrike"), conflict detection on pack load warns on ambiguous phrase prefixes.

**Research flag:** HD2 stratagem key sequences should be sourced from VoiceAttack community forum data and validated in-game, not authored from wiki descriptions alone. **Validate pack accuracy empirically before release.**

---

### Phase 5: UI Layer + Distribution

**Rationale:** Once the engine is validated, the egui configuration UI and distribution packaging can be built. The UI is explicitly not blocking the core engine — it is an addition on top of a working system.

**Delivers:** egui tray-accessible config window (phrase editor, profile picker, log view, mic level indicator). AppImage packaging with companion `install.sh`. AUR/PKGBUILD for CachyOS. Bundled `tiny.en` model or first-run setup wizard. SBOM / `LICENSES.md`.

**Addresses features:** Built-in phrase editor (minimal), recognition log in UI, mic level visual indicator. Full installable distribution.

**Avoids pitfalls:** P-13 (no silent surprise model download — bundled or wizard), P-15 (compositor matrix documented: GNOME Wayland, KDE Plasma Wayland, Sway supported; Hyprland best-effort), P-12 (license audit completed; all model licenses verified).

**Research flag:** egui/eframe patterns are well-documented. AppImage packaging with `cargo-appimage` is established. Skip research-phase.

---

### Phase 6+: VoiceAttack Parity (Future Milestones)

Conditions, variables, if-then logic, auto profile switching by window, TTS confirmation, soundboard, LibeiBackend as secondary input backend, multiple STT engine backends (Vosk swappable), plugin/extension API (`.so` or WASM). None of these are v1; all are architected for but not implemented.

---

### Phase Ordering Rationale

- **Levels 0 → 1 → 2 → 3 → 4** maps to the architecture's natural dependency graph — each level's components can only be tested against the previous level's outputs.
- **Latency must be validated in Phase 2**, not Phase 4. Discovering a 1+ s pipeline at Phase 4 requires rearchitecting audio/VAD/STT — a regression at the worst possible time.
- **PTT via evdev grab** is a Phase 1 architectural decision because it affects how AudioCapture and UinputBackend interact with the physical device. Making it a Phase 3 add-on requires reopening the audio subsystem.
- **Headless daemon** is a Phase 1 architectural decision. The UI (Phase 5) is a separately launched process. This ordering prevents the XWayland focus race from being discovered during the Phase 5 UI build.
- **Pack system before UI** — the pack format and import logic (Phase 4) must be stable before the UI editor (Phase 5) is built against it. Building the editor first would require revising it to match the data model.

### Research Flags

**Needs `/gsd-research-phase` during planning:**
- **Phase 2 (VAD + STT):** whisper-rs Codeberg build integration; Silero-VAD ONNX Runtime dylib bundling for AppImage; Tokio `spawn_blocking` interaction with whisper.cpp C FFI across platform targets.
- **Phase 3 (Dispatch):** Anti-cheat behavior of uinput virtual device against live HD2 nProtect GameGuard — this is empirical validation, not library research, but needs a dedicated testing plan.

**Standard patterns — skip research-phase:**
- **Phase 1 (Foundation):** evdev/uinput API is kernel-documented; udev rule patterns are established (AntiMicroX, Solaar).
- **Phase 4 (Packs):** TOML parsing, zip archive handling, inotify watcher — all well-documented Rust crates.
- **Phase 5 (UI + Distribution):** egui patterns are well-documented; `cargo-appimage` packaging is established.

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All crate licenses verified; PipeWire/cpal merge confirmed; whisper-rs actively maintained on Codeberg |
| Features | HIGH | VoiceAttack HD2 community data is primary source; stratagem counts patch-sensitive but format is stable |
| Architecture | HIGH | Two-stage pipeline pattern verified against sherpa-onnx, SL5-aura-service, atlas-voice production apps |
| Pitfalls | MEDIUM-HIGH | Wayland/uinput pitfalls verified against official sources; anti-cheat risk is MEDIUM (community reports, not controlled testing) |

**Overall confidence:** MEDIUM-HIGH

### Gaps to Address During Implementation

- **Anti-cheat empirical validation:** nProtect GameGuard behavior is based on community reports from Arch/Steam Deck users, not controlled testing. Must be validated in Phase 3 against a live HD2 instance before shipping the HD2 pack.
- **whisper-rs Codeberg build reliability:** The crate migrated from GitHub to Codeberg; build pipeline and cmake integration on CachyOS needs empirical verification in Phase 2.
- **Silero-VAD ONNX Runtime bundling in AppImage:** ONNX Runtime requires a bundled `.so`. The interaction between `ort` crate feature flags and AppImage FUSE mounts needs testing. Flag for Phase 5 packaging.
- **Wake-word model choice:** ARCHITECTURE.md references OpenWakeWord (AGPL-3.0 compatible, Apache-2.0) and sherpa-onnx. The specific model to ship as default is not resolved. Recommend sherpa-onnx keyword spotter for v1 (no external download needed for built-in models; ONNX Runtime already in dep tree for Silero-VAD).
- **HD2 stratagem key sequences:** Pack content must be validated against a live HD2 instance with arrow-key remapping active. Wiki/forum sources are MEDIUM confidence; timing values (dwell, inter-key) will require tuning per-system.
- **Compositor compatibility matrix:** Wayland focus and grab behavior is compositor-specific. GNOME and KDE Plasma should be the primary CI targets; Sway/Hyprland as best-effort. A headless compositor test harness (Sway in CI) is needed from Phase 1.

---

## Sources

### Primary (HIGH confidence)

- `whisper.cpp` GitHub (ggml-org/whisper.cpp) — license, model tiers, streaming API
- `whisper-rs` Codeberg (tazz4843/whisper-rs) — Rust FFI, maintained status
- OpenAI Whisper GitHub (openai/whisper) — model weight license (MIT confirmed)
- `silero-vad-rust` crates.io v6.2.1 — API, bundled ONNX model, sampling rates
- cpal GitHub (RustAudio/cpal) — PipeWire PR #938 merged Jan 2026; ALSA dep requirement
- egui GitHub (emilk/egui) v0.34 — Wayland native feature, license
- libei freedesktop.org — EI protocol spec, compositor support matrix
- uinput kernel documentation — capability flags, ioctl interface
- VoiceAttack Help PDF (voiceattack.com) — feature model, profile/pack design
- VoiceAttack HD2 community profile thread (forum.voiceattack.com/topic=4733) — timing values, arrow-key remap, phrase disambiguation
- AntiMicroX uinput wiki — udev rule pattern, group membership
- Flatpak GitHub issues #4137 and #5459 — uinput sandbox incompatibility
- XWayland focus race issues (cosmic-comp #1874, sway PR #8699) — compositor-specific focus race behavior
- LinVAM GitHub (stele95/LinVAM) — prior art; Python approach pitfalls

### Secondary (MEDIUM confidence)

- sherpa-onnx deepwiki — two-stage VAD + wake-word pipeline patterns
- Thomas Thelliez blog — VAD + Whisper latency pipeline community validation
- Arch Linux BBS — nProtect GameGuard Linux community reports
- Murmur (murmurlinux.com) — Rust+Tauri voice app binary size reference
- cargo-appimage crates.io v2.4.0 — packaging toolchain

### Tertiary (LOW confidence, needs validation)

- Helldivers 2 stratagem lists (divers.gg, thegamer.com) — game wikis; patch-sensitive; must be validated in-game
- Feros voice AI stack (dev.to) — Rust GC latency advantage claim; single blog post

---

*Research completed: 2026-04-21*
*Ready for roadmap: yes*

# Architecture Patterns: Linux Voice-Macro Desktop Application

**Domain:** Linux voice-control / macro-engine (VoiceAttack-class)
**Researched:** 2026-04-21
**Confidence:** HIGH (libei, CPAL/PipeWire, whisper.cpp — all verified against official/current sources)

---

## Overview

A VoiceAttack-class Linux app is a **real-time event-driven pipeline** with hard latency requirements
(end-of-speech → key event < 500 ms) and a permanently-running background listener. The architecture
must separate concerns cleanly enough that new strategem packs, STT backends, and input backends can
be added without touching the audio hot path.

The dominant pattern in production Linux voice apps (sherpa-voice-assistant, SL5-aura-service, atlas-voice)
is a **two-stage pipeline** — cheap wake-word gating followed by heavier full STT — layered over an
async message-passing runtime that decouples audio capture from macro execution.

---

## Component Diagram (ASCII)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          hd-linux-voice process                         │
│                                                                         │
│  ┌──────────────┐   16 kHz mono PCM   ┌───────────────────────────┐    │
│  │ AudioCapture │ ──────ring-buf─────► │   WakeWordDetector        │    │
│  │  (CPAL /     │                     │  (OpenWakeWord / sherpa-   │    │
│  │  PipeWire)   │                     │   onnx, always-on, < 5 %  │    │
│  │  RT thread   │                     │   CPU)                     │    │
│  └──────────────┘                     └──────────┬────────────────┘    │
│         │                                        │ wake event OR        │
│         │         ┌──────────────────────────────│─── PTT event         │
│         │         │                              ▼                      │
│         │         │               ┌─────────────────────────┐           │
│         │         │               │  VAD                    │           │
│         │   ring  │               │  (Silero-VAD / sherpa-  │           │
│         └────buf──┘               │   onnx, 32ms windows)  │           │
│                                   │  detects end-of-speech  │           │
│                                   └──────────┬──────────────┘           │
│                                              │ utterance segment        │
│                                              ▼                          │
│                                   ┌─────────────────────────┐           │
│                                   │  STT Engine             │           │
│                                   │  (whisper.cpp FFI or    │           │
│                                   │   Vosk; blocking thread │           │
│                                   │   pool, GPU optional)   │           │
│                                   └──────────┬──────────────┘           │
│                                              │ transcript: String       │
│                                              ▼                          │
│                                   ┌─────────────────────────┐           │
│                                   │  PhraseEngine           │           │
│                                   │  (exact → normalized →  │           │
│                                   │   fuzzy/edit-distance   │           │
│                                   │   match against active  │           │
│                                   │   MacroRegistry)        │           │
│                                   └──────────┬──────────────┘           │
│                                              │ MacroId + confidence     │
│                                              ▼                          │
│                                   ┌─────────────────────────┐           │
│                                   │  MacroDispatcher        │           │
│                                   │  (sequence player:      │           │
│                                   │   steps, delays, loops, │           │
│                                   │   conditions)           │           │
│                                   └──────────┬──────────────┘           │
│                                              │ key/mouse events         │
│                                              ▼                          │
│                         ┌────────────────────────────────────┐          │
│                         │         InputBackend (trait)        │          │
│                         │  ┌──────────┐  ┌────────────────┐  │          │
│                         │  │ LibeiB.  │  │ UinputBackend  │  │          │
│                         │  │ (Wayland │  │ (kernel uinput,│  │          │
│                         │  │  native) │  │  universal)    │  │          │
│                         │  └──────────┘  └────────────────┘  │          │
│                         └────────────────────────────────────┘          │
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │  Supporting subsystems (async runtime / main thread)             │   │
│  │                                                                  │   │
│  │  PackLoader ──► MacroRegistry ◄── PhraseEngine                  │   │
│  │  ProfileManager ──► { active pack(s), STT config, InputBackend} │   │
│  │  ConfigStore (TOML, ~/.config/hd-linux-voice/)                  │   │
│  │  UI Layer (GTK4 / egui) ──► ConfigStore, ProfileManager, Log    │   │
│  └──────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Component Boundaries

| Component | Responsibility | Inputs | Outputs | Talks To |
|-----------|---------------|--------|---------|----------|
| **AudioCapture** | Capture raw PCM from mic; write into shared ring buffer | Mic device | Ring buffer (16 kHz mono, F32) | WakeWordDetector, VAD |
| **WakeWordDetector** | Always-on phrase spotting; fire wake event; negligible CPU | Ring buffer | `WakeEvent` | VAD (enables recording window) |
| **VAD** | Gate recording; detect end-of-utterance silence | Ring buffer + wake/PTT gate | Utterance `Vec<f32>` segment | STT Engine |
| **STT Engine** | Convert audio segment to text transcript | Utterance segment | `String` transcript | PhraseEngine |
| **PhraseEngine** | Match transcript against registered phrases | Transcript + MacroRegistry | `Option<MacroId>` + confidence | MacroDispatcher |
| **MacroDispatcher** | Execute key/mouse sequences; manage timing and looping | MacroId + MacroDefinition | Input events | InputBackend |
| **InputBackend** | Abstract input injection; Wayland/uinput/X11 impls | Key/mouse events | OS input events | OS kernel / compositor |
| **PackLoader** | Load, parse, validate, and hot-reload `.hdpack` bundles | Filesystem (packs dir) | MacroRegistry population | MacroRegistry |
| **MacroRegistry** | In-memory store of phrase→macro mappings for active profile | PackLoader writes, PhraseEngine reads | — | PhraseEngine, PackLoader |
| **ProfileManager** | Load/switch profiles; own active config surface | Config files | Profile state | PackLoader, STT config, InputBackend config |
| **ConfigStore** | Persist user settings to TOML files | UI / API writes | Loaded config structs | ProfileManager, UI |
| **UI Layer** | Tray icon, phrase editor, profile switcher, log view | User input | Config writes, profile commands | ConfigStore, ProfileManager, MacroRegistry |

---

## Data Flow Direction

```
Microphone (hardware)
    │  raw PCM (real-time, RT audio thread)
    ▼
Ring Buffer (lock-free, shared between RT thread and consumer threads)
    │
    ├──[always] WakeWordDetector (low-CPU ONNX inference, async consumer)
    │               │ wake event → channel
    │               ▼
    │          [gate opens] VAD starts consuming ring buffer
    │
    └──[PTT] PTT keybind event → skip WakeWordDetector → gate VAD directly
                │
                ▼
            VAD (32ms window polling, silence detection)
                │ utterance segment (Vec<f32>) → async channel
                ▼
            STT Engine (blocking thread pool — whisper.cpp or Vosk)
                │ transcript String → async channel
                ▼
            PhraseEngine (sync, fast — exact → normalized → fuzzy match)
                │ MacroId → async channel
                ▼
            MacroDispatcher (async task, sequences with sleep/delay)
                │ key/mouse events
                ▼
            InputBackend (libei socket write or uinput ioctl)
                │
                ▼
            Game / OS receives events
```

**Key property:** Each stage communicates only via typed async channels (mpsc or broadcast).
No stage calls into another synchronously (except PhraseEngine which is pure/sync and fast enough
to call inline from the STT result handler). This means each stage can be tested in isolation.

---

## Async / Threaded Model Recommendation

**Recommendation: Tokio async runtime + dedicated real-time audio thread + blocking thread pool for STT.**

Rationale: Rust's `tokio` provides the best mix of async channel ergonomics and `spawn_blocking`
for CPU-heavy STT work. The audio capture **must** run on a dedicated OS thread with `SCHED_FIFO`
or elevated priority — never inside the async runtime, which would introduce jitter.

```
┌──────────────────────────────────────────────────────────────┐
│ Thread model                                                 │
│                                                              │
│  OS Thread: audio-rt (RT priority, SCHED_FIFO preferred)     │
│    └─ CPAL audio callback → writes to lock-free ring buffer  │
│                                                              │
│  OS Thread: wake-word-worker                                 │
│    └─ Reads ring buffer → runs ONNX inference → sends event  │
│                                                              │
│  tokio::task (async): vad-task                               │
│    └─ Woken by wake event → polls ring buffer → segments     │
│                                                              │
│  tokio::task::spawn_blocking: stt-task                       │
│    └─ whisper.cpp / Vosk inference (CPU/GPU heavy)           │
│                                                              │
│  tokio::task (async): phrase-dispatch-task                   │
│    └─ Phrase matching + macro dispatch coordination          │
│                                                              │
│  tokio::task (async): macro-executor-task                    │
│    └─ Sequence steps with tokio::time::sleep between events  │
│                                                              │
│  OS Thread: ui-thread (GTK4 main loop or egui event loop)   │
│    └─ Communicates via mpsc channels with async runtime      │
└──────────────────────────────────────────────────────────────┘
```

**Do not** run STT inference inside a `tokio` async task directly — it will starve the executor.
Always use `spawn_blocking` or a dedicated thread with a channel result.

---

## Two-Stage Pipeline: Wake Word → Full STT

The wake-word stage and STT stage serve different cost profiles and must be independently gated:

```
Stage 1: IDLE (always running)
─────────────────────────────
Cost: < 5% CPU, no GPU required
Model: OpenWakeWord (ONNX, ~20 MB) or sherpa-onnx keyword spotter
Operation: Sliding window ONNX inference over ring buffer (continuous)
Output: WakeEvent { confidence: f32, timestamp: Instant }

Stage 2: ACTIVE (triggered by Stage 1 or PTT)
──────────────────────────────────────────────
Cost: 200–1500 ms CPU burst (whisper.cpp) or < 100 ms (Vosk tiny model)
Model: whisper.cpp (ggml quantized, e.g. base.en = 140 MB) or Vosk (50 MB)
Operation: Run once per utterance segment; NOT continuous streaming (for latency)
Output: transcript String

Gate logic:
  - WakeEvent received → enable VAD → record until silence → send to STT
  - PTT pressed       → enable VAD immediately → record until PTT released or silence
  - Timeout (5s of no speech after wake) → return to IDLE without STT call
```

**Integration concern:** During Stage 2, WakeWordDetector must be paused or its ring-buffer
read pointer advanced without inference to avoid re-triggering on STT audio bleed-through.
Implement via a shared `AtomicBool` gate checked at the top of the wake-word loop.

**Latency budget (target < 500 ms end-to-speech → key event):**

| Stage | Budget | Notes |
|-------|--------|-------|
| VAD silence detection | ~250 ms | Configurable; balance false-cuts vs delay |
| STT inference | ~150 ms | whisper.cpp base.en on modern CPU; < 50 ms Vosk |
| Phrase matching | ~1 ms | In-memory hash + fuzzy, negligible |
| Macro dispatch first step | ~5 ms | First key event |
| **Total** | **~400 ms** | Leaves 100 ms headroom |

---

## Strategem Pack / Plugin System

### Pack Format

Packs are **zip archives** (`.hdpack`) containing:

```
my-helldivers-pack-1.2.0.hdpack
├── manifest.toml          ← required: name, version, game, author, min-app-version
├── phrases/
│   ├── stratagems.toml    ← phrase → macro_id mappings
│   └── callouts.toml      ← optional callout phrases
└── macros/
    ├── eagle_airstrike.toml    ← key sequence, timing, conditions
    ├── orbital_cannon.toml
    └── ...
```

**manifest.toml** (example):
```toml
name = "Helldivers 2 Stratagems"
version = "1.2.0"
game = "helldivers2"
author = "hd-linux-voice team"
min_app_version = "0.3.0"
license = "CC-BY-4.0"
```

**phrases/stratagems.toml** (example):
```toml
[[phrase]]
text = ["eagle airstrike", "eagle air strike", "call eagle"]
macro_id = "eagle_airstrike"
confidence_threshold = 0.75

[[phrase]]
text = ["orbital cannon"]
macro_id = "orbital_cannon"
```

**macros/eagle_airstrike.toml** (example):
```toml
macro_id = "eagle_airstrike"
description = "Eagle Airstrike stratagem"
[[steps]]
  action = "key_down"
  key = "ctrl"
[[steps]]
  action = "key_tap"
  key = "up"
[[steps]]
  action = "key_tap"
  key = "right"
[[steps]]
  action = "key_tap"
  key = "down"
[[steps]]
  action = "key_tap"
  key = "right"
[[steps]]
  action = "key_up"
  key = "ctrl"
```

### PackLoader Architecture

```
PackLoader
  ├── scan_dir(~/.local/share/hd-linux-voice/packs/)
  ├── validate_manifest(version semver, license field)
  ├── deserialize phrases + macros into MacroRegistry
  ├── hot_reload: inotify watcher → reload on .hdpack change
  └── conflict_detection: warn on duplicate phrase across packs

MacroRegistry (in-memory)
  ├── phrase_index: HashMap<NormalizedPhrase, MacroId>
  ├── macro_defs: HashMap<MacroId, MacroDefinition>
  └── owned_by: HashMap<MacroId, PackId>  ← for conflict resolution
```

**Pack resolution order:** User's personal pack overrides game pack overrides defaults.
This mirrors VoiceAttack's profile inclusion model.

### Future Plugin API (beyond v1)

For v2+, expose a plugin ABI allowing native Rust shared libraries (`.so`) or WASM modules
to register custom action types, additional STT backends, and overlay hooks.
In v1, focus on data-driven TOML packs only — no plugin ABI surface to maintain.

---

## Profile System and State Management

### Profile Structure

```toml
# ~/.config/hd-linux-voice/profiles/helldivers2.toml
name = "Helldivers 2"
active_packs = ["helldivers2-stratagems-1.2.0", "personal-overrides"]
stt_model = "whisper-base-en"
wake_word = "hey diver"           # custom or PTT only
ptt_key = "CapsLock"             # optional
input_backend = "libei"          # or "uinput"
confidence_threshold = 0.75      # global fallback
```

### Listening State Machine

```
        ┌────────────────────────────────────┐
        │              IDLE                  │◄────────────────────┐
        │  WakeWordDetector running          │                     │
        │  STT Engine idle                   │                     │
        └──────────────┬─────────────────────┘                     │
                       │ WakeEvent (confidence > threshold)         │
                       │   OR PTT key pressed                      │
                       ▼                                           │
        ┌────────────────────────────────────┐                     │
        │            LISTENING               │                     │
        │  VAD recording audio               │                     │
        │  Visual indicator active (UI)      │                     │
        └──────────────┬─────────────────────┘                     │
                       │ VAD: silence > 250ms                      │
                       │   OR PTT released                         │
                       │   OR hard timeout (5s)                    │
                       ▼                                           │
        ┌────────────────────────────────────┐                     │
        │           RECOGNIZING              │                     │
        │  STT Engine running (spawn_blocking│                     │
        │  WakeWordDetector paused           │                     │
        └──────────────┬─────────────────────┘                     │
                       │ transcript ready                           │
                       ▼                                           │
        ┌────────────────────────────────────┐                     │
        │            MATCHING                │                     │
        │  PhraseEngine: exact / fuzzy match │                     │
        └──────────────┬─────────────────────┘                     │
                       │ MacroId found         │ no match / low conf
                       ▼                       ▼                   │
        ┌──────────────────────┐   ┌───────────────────┐           │
        │      EXECUTING       │   │   NO_MATCH        │           │
        │  MacroDispatcher     │   │  (optional audio  │           │
        │  sends input events  │   │   feedback beep)  │           │
        └──────────┬───────────┘   └─────────┬─────────┘           │
                   │ sequence done            │                     │
                   └──────────────────────────┴─────────────────────┘
```

### Listening Modes

| Mode | Trigger | Behavior |
|------|---------|----------|
| **Wake-word** | Programmable hotphrase (default) | WakeWordDetector always runs; STT fires on match |
| **Push-to-talk** | Configurable key/button | WakeWordDetector disabled; VAD gates on key hold |
| **Always-on** (future) | Config option | Continuous VAD + STT; high CPU; not recommended for gameplay |

---

## Input Backend Abstraction

The input system must be isolatable — Wayland vs X11 vs uinput are different APIs.
Define a trait early; ship two concrete impls.

```rust
// Pseudocode trait (Rust)
trait InputBackend: Send + 'static {
    fn key_down(&mut self, key: Key) -> Result<()>;
    fn key_up(&mut self, key: Key) -> Result<()>;
    fn key_tap(&mut self, key: Key, hold_ms: u64) -> Result<()>;
    fn mouse_move(&mut self, x: i32, y: i32) -> Result<()>;
    fn mouse_click(&mut self, button: MouseButton) -> Result<()>;
}
```

| Backend | Mechanism | Works On | Caveats |
|---------|-----------|----------|---------|
| **UinputBackend** | `/dev/uinput` kernel module (ioctl) | All Linux (Wayland + X11) | Needs `input` group or `uinput` capability; events appear as new virtual device |
| **LibeiBackend** | `libei` + XDG RemoteDesktop portal | Wayland compositors supporting EI | Requires compositor support (GNOME 43+, KDE 5.27+); portal permission prompt on first run |
| **X11Backend** (future) | `xtest` / `xdotool` | X11 / XWayland | Lower priority; XWayland handles most legacy cases |

**Recommended v1 strategy:** Ship `UinputBackend` first (works universally with just group membership).
Add `LibeiBackend` once basic pipeline is working — libei is the long-term correct solution for
Wayland-native apps.

**Anti-cheat note:** Both uinput and libei events appear as real input devices to the OS/game.
The risk of anti-cheat flagging synthetic input exists regardless of backend; this must be
validated against Helldivers 2 specifically before shipping game-focused packs.

---

## Suggested Build Order

Components are ordered by dependency depth. Each level can be developed and tested independently
before the next level requires it.

### Level 0 — Foundation (no dependencies on other app components)

| Component | What | Why First |
|-----------|------|-----------|
| `AudioCapture` | CPAL + PipeWire ring-buffer capture | Everything else needs audio |
| `UinputBackend` | `/dev/uinput` input injection | Needed for any macro test; kernel API, no compositor |
| `ConfigStore` | TOML read/write, config structs | All other components need config |

**Milestone deliverable:** Can capture mic audio and inject keystrokes into a terminal. CLI-only.

### Level 1 — Pipeline Core (depends on Level 0)

| Component | Depends On | What |
|-----------|-----------|------|
| `WakeWordDetector` | AudioCapture | OpenWakeWord ONNX or sherpa-onnx wake; single hardcoded phrase |
| `VAD` | AudioCapture | Silero-VAD or sherpa-onnx VAD; utterance segmentation |
| `STT Engine` | VAD output | whisper.cpp FFI wrapper; returns transcript |

**Milestone deliverable:** Speak a phrase → get a transcript printed. Latency benchmark possible.

### Level 2 — Phrase Matching + Dispatch (depends on Level 1)

| Component | Depends On | What |
|-----------|-----------|------|
| `PhraseEngine` | STT Engine, MacroRegistry | Exact + normalized + fuzzy match |
| `MacroRegistry` | ConfigStore | In-memory store, hardcoded test macros |
| `MacroDispatcher` | MacroRegistry, InputBackend | Execute key sequences from definition |

**Milestone deliverable:** Speak "eagle airstrike" → correct key sequence fires in-game. First playable.

### Level 3 — Pack System (depends on Level 2)

| Component | Depends On | What |
|-----------|-----------|------|
| `PackLoader` | MacroRegistry, ConfigStore | TOML/zip pack parsing, validation, conflict detection |
| `ProfileManager` | PackLoader, ConfigStore | Profile load/switch, state ownership |
| `LibeiBackend` | InputBackend trait | Wayland-native input; replaces uinput for Wayland targets |

**Milestone deliverable:** Install a `.hdpack` file → phrases and macros load automatically.

### Level 4 — UI + Distribution (depends on Level 3)

| Component | Depends On | What |
|-----------|-----------|------|
| `UI Layer` | All above | GTK4 or egui: tray, editor, profile picker, log |
| Hot-reload | PackLoader | inotify watcher for pack directory |
| Packaging | Everything | `.deb`, `.rpm`, AppImage, install docs |

**Milestone deliverable:** Full user-installable application with pack editor.

### Level 5 — VoiceAttack Parity (future milestones)

Conditions, variables, expressions, multiple simultaneous macros, plugin ABI (`.so` / WASM),
additional STT backends (Vosk, cloud-optional), accessibility features.

---

## Key Architecture Invariants

1. **Audio RT thread never blocks.** Ring buffer writes must be wait-free. No allocations in the CPAL callback.
2. **STT never runs on the tokio executor.** Always `spawn_blocking` or dedicated OS thread.
3. **InputBackend is a trait from day one.** Never call uinput directly from MacroDispatcher — always through the abstraction.
4. **Packs are data, not code.** v1 packs are TOML only. No scripting, no eval, no dynamic loading. Keeps AGPL compliance simple — pack authors cannot embed proprietary code.
5. **One active profile at a time.** Profile switch flushes MacroRegistry and reloads. No merge-at-runtime complexity in v1.
6. **WakeWordDetector pause during STT.** Prevents re-triggering on synthesized audio or the next wake-word check eating CPU while the STT thread pool is saturated.
7. **Channels are the only inter-component API.** Components expose channel endpoints, not method calls. This enables mocking any stage in tests.

---

## Pitfalls This Architecture Avoids

| Pitfall | How Architecture Avoids It |
|---------|---------------------------|
| Ring buffer overflow at high CPU load | Lock-free SPSC ring buffer; wake-word consumer tracks read pointer independently |
| STT blocking audio pipeline | STT runs in separate blocking thread pool; audio capture is unaffected |
| Wayland input rejection | `InputBackend` trait lets us swap libei / uinput at runtime; libei goes through portal with explicit permission |
| Pack version conflicts | Manifest semver + conflict detection in PackLoader; user pack wins over default pack |
| Anti-cheat false positives | Uinput events look like real hardware; identical to physical keyboard at driver level (cannot be avoided; requires game-specific testing) |
| Profile state corruption on crash | ConfigStore writes atomically (temp file + rename); dirty flag prevents partial writes |

---

## Sources

- [libei EI Protocol](https://libinput.pages.freedesktop.org/libei/) — HIGH confidence; official freedesktop.org docs
- [CPAL PipeWire PR #1093](https://github.com/RustAudio/cpal/pull/1093) — HIGH confidence; merged March 2026
- [ydotool / uinput approach](https://github.com/ReimuNotMoe/ydotool) — HIGH confidence; established tool
- [sherpa-onnx VAD + wake word](https://deepwiki.com/k2-fsa/sherpa-onnx/3.3-voice-activity-detection-(vad)) — HIGH confidence; official wiki
- [Two-stage VAD + Whisper pipeline](https://thomasthelliez.com/blog/voice-activity-detection-and-wake-word-setup-for-whisper-based-voice-interfaces/) — MEDIUM confidence; community blog verified against sherpa-onnx docs
- [VoiceAttack plugin/profile model](https://voiceattack.com/VoiceAttackHelp.pdf) — HIGH confidence; official help PDF
- [OpenWakeWord](https://github.com/dscripka/openWakeWord) — HIGH confidence; active project, AGPL-3.0 compatible

# Technology Stack

**Project:** hd-linux-voice — Linux Wayland voice-macro application  
**Researched:** 2026-04-21  
**Overall confidence:** MEDIUM-HIGH (verified per library; a few integration points need empirical validation)

---

## Quick Recommendation Table

| Layer | Recommended | License | AGPL-compatible | Confidence |
|-------|-------------|---------|-----------------|------------|
| **Language** | Rust | N/A | N/A | HIGH |
| **Speech recognition** | whisper.cpp via `whisper-rs` | MIT (code + weights) | ✅ Yes | HIGH |
| **Voice activity detection** | `silero-vad-rust` | MIT | ✅ Yes | HIGH |
| **Audio capture** | `cpal` v0.17+ | MIT/Apache-2.0 | ✅ Yes | HIGH |
| **Input injection** | `evdev`/`uinput` via udev rules | GPL-2.0 kernel interface; crate MIT | ✅ Yes | HIGH |
| **UI framework** | `egui` / `eframe` | MIT/Apache-2.0 | ✅ Yes | HIGH |
| **Config/data format** | TOML (`serde` + `toml`) | MIT/Apache-2.0 | ✅ Yes | HIGH |
| **Packaging** | AppImage (primary), Flatpak (stretch) | N/A | N/A | MEDIUM |

---

## Programming Language: Rust

**Recommendation: Rust.** Use Rust for the entire application.

### Why Rust over Python, Go, or C++

| Criterion | Rust | Python | Go | C++ |
|-----------|------|--------|----|-----|
| GC pauses / audio jitter | None (ownership) | GC pause risk | GC | None |
| End-to-end latency predictability | Deterministic | Non-deterministic | Non-deterministic | Deterministic |
| whisper.cpp FFI | Mature (whisper-rs, Unlicense) | Mature (faster-whisper) | Thin | Native |
| uinput/evdev | `evdev` crate, MIT | `evdev` package | syscall manually | libevdev |
| Single-binary distribution | Yes | No (interpreter + venv) | Yes | Yes (with care) |
| AGPL distribution | Straightforward | Complex venv licensing | Straightforward | Complex |
| Linux desktop UI | egui, gtk4-rs, Tauri | PyGTK, PyQt6 (LGPL) | Fyne | Qt (LGPL) |
| Async audio pipeline | `tokio` + `cpal` | asyncio (GIL) | goroutines | threads |
| Precedent in voice apps | Murmur (Rust+Tauri), HootVoice | Vocalinux, LinVAM | Rare | Vokey (C++) |

**The decisive factor for this use case is latency predictability in the audio pipeline.** A 50–100 ms GC pause during a 20 ms audio frame causes audible glitches and missed VAD boundaries. Rust's ownership model eliminates this class of problem. Combined with its ability to produce a single distributable binary with no runtime dependency, it is the correct choice.

**Python is a viable MVP shortcut** if initial iteration speed is paramount — `faster-whisper` + `evdev` + `PyQt6` already powers LinVAM. However, Python's GIL and GC make the final-quality audio pipeline harder to tune, and Python packaging for end-users (bundling models, venv, system audio deps) is significantly more painful than a Rust AppImage.

**Minimum Rust version:** 1.85+ (required for cpal's PipeWire backend; MSRV should be pinned in `Cargo.toml`).

---

## Speech Recognition

### Primary: whisper.cpp via `whisper-rs`

**Confidence: HIGH**

| Property | Value |
|----------|-------|
| Upstream | [ggml-org/whisper.cpp](https://github.com/ggml-org/whisper.cpp) |
| Rust bindings | [whisper-rs](https://codeberg.org/tazz4843/whisper-rs) (migrated from GitHub; maintained) |
| License | whisper.cpp: **MIT**; whisper-rs: **Unlicense (public domain)** |
| OpenAI model weights | **MIT** (confirmed for code + weights) |
| AGPL-compatible | ✅ Yes — permissive licenses can be bundled with AGPL |
| CPU performance (tiny model) | ~32× real-time on CPU → 10 s audio processes in ~0.3 s |
| CPU performance (base model) | ~16× real-time |
| CPU performance (large-v3) | ~1× real-time on CPU; GPU needed for real-time |
| GPU acceleration | CUDA, ROCm/hipBLAS, Vulkan (feature flags) |
| Model sizes | tiny (~39 MB), base (~74 MB), small (~244 MB) |
| Streaming support | Yes — via `whisper-rs` real-time PCM API |

**Recommended model tier for gaming macros:** `base.en` or `tiny.en` — English-only models are ~30–40% smaller and faster. For strategem commands (short, distinct phrases), `tiny.en` with VAD-gated segments should give < 400 ms end-to-end latency on a mid-range CPU.

**Newer alternative:** `whisper-cpp-plus-rs` (operator-kit, MIT) adds async streaming and native VAD support on top of whisper.cpp; worth evaluating when the Codeberg mirror of whisper-rs is harder to integrate.

### Fallback: Vosk via `vosk-rs`

**Confidence: MEDIUM**

| Property | Value |
|----------|-------|
| Upstream | [alphacep/vosk-api](https://github.com/alphacep/vosk-api) |
| Rust bindings | [vosk-rs](https://github.com/Bear-03/vosk-rs) v0.3.1, MIT |
| Vosk API library | **Apache-2.0** |
| AGPL-compatible | ✅ Yes |
| Model sizes | ~40–50 MB (small English model) |
| Performance | Continuous streaming; very low RAM; runs on low-end CPUs |
| Accuracy vs. whisper | Lower for unconstrained speech; acceptable for fixed command phrases |

**Use Vosk if:** target hardware is very low-end (< 4-core CPU, 4 GB RAM) or latency budget is extremely tight (Vosk streams word-by-word as it recognizes, no end-of-utterance required). For the Helldivers 2 gaming use case, whisper.cpp tiny accuracy will win; Vosk is the fallback for older machines.

### Do Not Use

- **OpenAI API / cloud Whisper** — violates the local-only requirement and AGPL distribution intent.
- **Coqui TTS/STT** — [Coqui STT is deprecated and archived](https://github.com/coqui-ai/STT) (2023).
- **CMU PocketSphinx** — used by Vokey (2022, abandoned); accuracy far below modern Whisper/Vosk.
- **DeepSpeech** — Mozilla discontinued it in 2021; do not use.

---

## Voice Activity Detection (VAD)

### Primary: `silero-vad-rust`

**Confidence: HIGH**

| Property | Value |
|----------|-------|
| Crate | [silero-vad-rust](https://crates.io/crates/silero-vad-rust) v6.2.1 (Nov 2025) |
| License | MIT |
| AGPL-compatible | ✅ Yes |
| Model | Bundled ONNX model (opset 15/16); no download step |
| Sampling rates | 8 kHz, 16 kHz |
| Inference | CPU via ONNX Runtime (`ort` crate); CUDA optional |
| API | `VadIterator` emitting `VadEvent::Start` / `VadEvent::End` per chunk |

VAD gates audio segments before passing to whisper.cpp, which is critical for macro latency — without VAD, whisper.cpp must wait for a silence timeout or process continuous audio. Silero VAD processes in 512-sample chunks, emitting `End` events that define segment boundaries in real-time.

**ONNX Runtime dependency:** Silero requires `ort` crate and ONNX Runtime 1.22.x dylib. This must ship as a bundled `.so` or be documented as a system dep in packaging.

---

## Audio Capture

### Primary: `cpal` v0.17.x

**Confidence: HIGH**

| Property | Value |
|----------|-------|
| Crate | [cpal](https://github.com/RustAudio/cpal) v0.17.3 (Feb 2026) |
| License | Apache-2.0 |
| AGPL-compatible | ✅ Yes |
| Linux backends | PipeWire (default, ≥ Rust 1.85), ALSA, JACK, PulseAudio |
| PipeWire support | ✅ Merged Jan 2026 (PR #938); now first-class |
| ALSA still needed | Build-time dep `libasound2-dev` required even on PipeWire systems |

**Why PipeWire matters:** Modern Wayland distros (Fedora, Ubuntu 22.04+, Arch, NixOS) route audio through PipeWire. ALSA still works via the PipeWire ALSA compatibility layer, but native PipeWire gives lower-latency buffer configuration and better device management. cpal's PipeWire merge makes this transparent.

**Recommended pipeline:**
```
Microphone → cpal (16 kHz, mono, f32) → ring buffer → VAD thread → segment queue → whisper.cpp thread → recognized text
```

**Runtime audio device selection** should be configurable so users can pick their microphone without editing config files (important for headsets vs. desk mics in gaming).

---

## Input Injection (Wayland)

This is the most complex area. Multiple approaches exist; the architecture must abstract them behind a trait.

### Recommended: `uinput` (kernel virtual device) via `evdev` crate

**Confidence: HIGH for game input; MEDIUM for compositor-specific apps**

| Property | Value |
|----------|-------|
| Crate | [evdev](https://crates.io/crates/evdev) (MIT) |
| Mechanism | Creates a virtual `/dev/input/eventX` device via `/dev/uinput` |
| Compositor support | All Wayland compositors (kernel-level, not compositor-level) |
| Game support | ✅ Works with Steam/Proton (evdev device appears to Steam Input) |
| Privilege requirement | Read/write access to `/dev/uinput`; solved via udev rule |
| Wayland security model | Does NOT bypass Wayland's window focus restriction for compositor UI — but games via Steam work because Steam reads raw evdev |

**Why uinput wins for gaming:** Helldivers 2 runs via Proton through Steam. Steam reads input at the evdev/uinput level before Wayland compositor focus policies apply. A uinput virtual device is indistinguishable from a real gamepad/keyboard to Steam and the game. This is how Heroic Games Launcher's gamepad support, game controllers, and tools like AntiMicroX work.

**Setup required:** Ship a udev rule in packaging:
```
# /etc/udev/rules.d/70-hd-linux-voice.rules
KERNEL=="uinput", GROUP="input", MODE="0660"
```
Add users to the `input` group (standard pattern used by Solaar, AntiMicroX, etc.).

**Anti-cheat consideration:** Easy Anti-Cheat (used by Helldivers 2) allows uinput devices on Linux as they represent accessible/legitimate hardware emulation. **Flag for empirical validation in Phase 1.**

### Secondary / Future: `libei` via `enigo` or direct FFI

**Confidence: LOW — use as a future backend, not primary**

| Property | Value |
|----------|-------|
| Library | libei 1.x |
| Compositor support | GNOME (functional), KDE Plasma 6 (partial — missing `ConnectToEIS` portal method as of 2025) |
| Rust support | `enigo` crate (MIT) has experimental libei feature — has known issues (Tokio runtime conflicts, works once then fails) |
| Appropriate for | UI automation, accessibility tools that target focused windows |
| NOT appropriate for | Game input injection (game receives input through Steam/evdev bypass) |

libei is the long-term Wayland standard for sandboxed input emulation (Flatpak portals, remote desktop). For this project's primary use case (game macros), uinput is simpler and more reliable today. Architecture should isolate the input backend so libei can be added later as a backend option.

### Do Not Use

- **xdotool** — X11 only; does not work on native Wayland windows.
- **wtype** — Only works via `wlr-virtual-keyboard-unstable-v1` protocol; not supported by all compositors (GNOME does not support it); unmaintained (no commits in ~2 years).
- **ydotool** — Wraps uinput through a daemon (`ydotoold`); adds an unnecessary layer of complexity when direct uinput via `evdev` crate achieves the same. Use `evdev` directly.
- **X11/XTest** — Do not fall back to X11 input injection even via XWayland. Games run under Proton use Wayland-native input path.

---

## UI Framework

### Primary: `egui` / `eframe` v0.34.x

**Confidence: HIGH**

| Property | Value |
|----------|-------|
| Crate | [egui](https://github.com/emilk/egui) v0.34.1 (Mar 2026) |
| License | MIT OR Apache-2.0 |
| AGPL-compatible | ✅ Yes |
| Wayland support | ✅ Native (Wayland is a default feature of `eframe` on Linux) |
| Rendering | GPU-accelerated via `wgpu` or `glow`; software fallback available |
| Binary size contribution | ~17–18 MB total binary typical for a small app |
| Use case fit | Immediate mode; excellent for config panels, macro editors, status widgets |
| Distribution | No system UI library deps needed (unlike GTK, which requires GTK4 runtime) |

**Why egui over GTK4:** egui produces a self-contained binary with no system GTK runtime dependency — essential for AppImage distribution. GTK4 bindings (`gtk4-rs`) require GTK4 at runtime, which is version-sensitive across distros. egui's immediate mode also makes a macro editor (with real-time preview) simpler to build.

**Why egui over Tauri:** Tauri uses a system WebView (WebKitGTK2 → GTK4 migration ongoing as of late 2025), which has version fragmentation on Linux distros. egui has no webview dependency.

**UI pattern recommendation:** Run egui as a separate tray-accessible window (shown on demand), not always-on-top. The app should run primarily as a background service; the UI is for configuration, not for gameplay overlay.

### Alternative: GTK4 via `gtk4-rs`

Suitable if VoiceAttack-class desktop integration (system theme, notifications, GNOME/KDE native look) becomes a priority. Requires GTK4 runtime on user systems. License: MIT — AGPL-compatible.

---

## Configuration / Data

| Component | Library | License | Notes |
|-----------|---------|---------|-------|
| Config file format | TOML | N/A | Human-editable, well-supported in Rust community |
| Serialization | `serde` + `serde_toml` | MIT/Apache-2.0 | Standard Rust pattern |
| Macro pack format | TOML (version-tagged) | N/A | Importable packs; schema-versioned for upgrades |
| Persistent state | `serde_json` or SQLite (`rusqlite`) | MIT/Apache-2.0 | SQLite for profiles/variables (VoiceAttack-class depth) |

---

## Async Runtime

| Component | Library | License | Notes |
|-----------|---------|---------|-------|
| Async runtime | `tokio` | MIT | Industry standard; handles mic input, STT queuing, command dispatch |
| Channels | `tokio::sync::mpsc` | — | Audio → VAD → STT pipeline |

**Note:** `silero-vad-rust` has a known conflict when called from within a Tokio runtime (panic). Workaround: run the VAD on a `std::thread` (blocking thread), communicate results via an `mpsc` channel into the Tokio executor. Verify this is resolved in current versions before integrating.

---

## Packaging and Distribution

| Format | Tool | Effort | Best for |
|--------|------|--------|---------|
| **AppImage** (primary) | `cargo-appimage` v2.4.0 | Low | Distro-agnostic; single-file; drops into any Linux system |
| **Flatpak** | `flatpak-builder` + manifest | Medium | Sandboxed distribution; GNOME Software / KDE Discover integration |
| **AUR / PKGBUILD** | Manual | Low | Arch/CachyOS users (project's own distro likely) |
| **Debian/RPM** | `cargo-deb` / `cargo-rpm` | Medium | Ubuntu PPA / Fedora COPR |

**Start with AppImage for v1** — lowest friction for "download and run" distribution. Include:
- The compiled binary
- Bundled ONNX Runtime `.so` (for Silero VAD)
- Default `tiny.en` Whisper model (or a download script for first-run)
- Udev rule installer script

**Model distribution note:** Whisper `.gguf` model files (~39–244 MB) should not be bundled in the AppImage. Provide a first-run downloader that pulls from Hugging Face or a project mirror. Vosk models similarly.

---

## Alternatives Considered

| Category | Recommended | Rejected | Why Rejected |
|----------|-------------|----------|--------------|
| Language | Rust | Python | GC latency, packaging complexity |
| Language | Rust | Go | No mature whisper.cpp FFI; less control over audio threading |
| STT | whisper.cpp | Coqui STT | Archived/deprecated 2023 |
| STT | whisper.cpp | DeepSpeech | Discontinued 2021 |
| STT | whisper.cpp | cloud APIs | Violates local-only requirement |
| Input | uinput/evdev | wtype | Unmaintained; limited compositor support; not game-compatible |
| Input | uinput/evdev | xdotool | X11-only |
| Input | uinput/evdev | enigo+libei | Tokio conflicts; incomplete KDE support; not game-compatible |
| UI | egui | Tauri | WebKitGTK fragmentation on Linux |
| UI | egui | GTK4 | Runtime dep fragmentation for AppImage |
| Audio | cpal | rodio | rodio wraps cpal; use cpal directly for lower-level control |
| Audio | cpal | PortAudio | C dependency; worse Rust FFI story |

---

## AGPL-3.0 Compatibility Summary

| Library | License | AGPL-3.0 Compatible | Notes |
|---------|---------|---------------------|-------|
| whisper.cpp | MIT | ✅ | Permissive; MIT can be distributed alongside AGPL |
| whisper-rs | Unlicense | ✅ | Public domain; no restriction |
| OpenAI Whisper model weights | MIT | ✅ | Confirmed; models + code both MIT |
| Vosk API | Apache-2.0 | ✅ | Apache-2.0 is GPL/AGPL-compatible (one-way) |
| vosk-rs bindings | MIT | ✅ | Permissive |
| silero-vad-rust | MIT | ✅ | Permissive |
| cpal | Apache-2.0 | ✅ | |
| evdev crate | MIT | ✅ | |
| egui / eframe | MIT OR Apache-2.0 | ✅ | |
| serde | MIT OR Apache-2.0 | ✅ | |
| tokio | MIT | ✅ | |
| ONNX Runtime (ort crate) | MIT | ✅ | |

**uinput kernel interface** is GPL-2.0 (Linux kernel), but user-space access to `/dev/uinput` via system calls is not subject to GPL copyleft (standard Linux syscall boundary rule; consistent with how all Linux applications use the kernel).

**No GPL-2.0-only or GPLv2-incompatible libraries are currently in scope.** Track licenses per-crate as dependencies are added. The tightest compatibility requirement is: MIT and Apache-2.0 dependencies can be distributed alongside AGPL-3.0 source; LGPL dependencies can be dynamically linked; GPL-2.0-only would create a compatibility problem and must be avoided.

---

## Installation (Reference)

```toml
# Cargo.toml — core dependencies (versions as of April 2026)
[dependencies]
whisper-rs = { git = "https://codeberg.org/tazz4843/whisper-rs", features = ["cuda"] }
silero-vad-rust = "6"
cpal = { version = "0.17", features = ["pipewire"] }
evdev = "0.12"
egui = "0.34"
eframe = { version = "0.34", features = ["wayland"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
toml = "0.8"

[build-dependencies]
# whisper-rs links against whisper.cpp via cmake; ensure cmake is available
```

```bash
# System build dependencies (Debian/Ubuntu)
sudo apt install build-essential cmake libasound2-dev libpipewire-0.3-dev pkg-config

# System build dependencies (Arch/CachyOS)
sudo pacman -S base-devel cmake alsa-lib pipewire

# Runtime: add user to input group (for /dev/uinput)
sudo usermod -aG input $USER
```

---

## Sources

| Claim | Source | Confidence |
|-------|--------|------------|
| whisper.cpp MIT license | github.com/ggml-org/whisper.cpp/blob/master/LICENSE | HIGH |
| OpenAI model weights MIT | github.com/openai/whisper/blob/main/LICENSE | HIGH |
| whisper-rs Unlicense, Codeberg mirror | codeberg.org/tazz4843/whisper-rs | HIGH |
| Vosk Apache-2.0 | github.com/alphacep/vosk-api/blob/master/COPYING | HIGH |
| vosk-rs MIT | lib.rs/crates/vosk | HIGH |
| cpal PipeWire merged Jan 2026 | github.com/RustAudio/cpal/pull/938 | HIGH |
| silero-vad-rust v6.2.1 Nov 2025 | crates.io/crates/silero-vad-rust | HIGH |
| egui MIT/Apache dual, Wayland default | github.com/emilk/egui; eframe Cargo.toml | HIGH |
| enigo libei Tokio conflict | github.com/enigo-rs/enigo/issues/453 | HIGH |
| ydotool latest v1.0.4 Jan 2023 | github.com/ReimuNotMoe/ydotool | MEDIUM |
| KDE missing ConnectToEIS portal | github.com/feschber/lan-mouse/issues/293 | MEDIUM |
| cargo-appimage v2.4.0 Nov 2025 | crates.io/crates/cargo-appimage | HIGH |
| Murmur (Rust+Tauri) ~15 MB, 50 MB RAM | murmurlinux.com/compare/vocalinux | MEDIUM |
| LinVAM v0.8.4 Dec 2025 | github.com/stele95/LinVAM | MEDIUM |
| Rust GC latency advantage for audio | dev.to/loopbreaker111 (Feros voice AI stack) | MEDIUM |

# Feature Landscape: Linux Voice-Macro / Gaming Voice Control

**Domain:** Voice-driven keyboard/mouse macro application for Linux gaming (Helldivers 2 / VoiceAttack-class)
**Researched:** 2026-04-21
**Reference Implementations:** VoiceAttack (Windows, gold standard), voice-commander (Python/MIT), hyprwhspr, Vocalinux

---

## Table Stakes

Features users expect from day one. Missing any of these = product is unusable or users immediately drop it.

| Feature | Why Expected | Complexity | v1? | Notes |
|---------|--------------|------------|-----|-------|
| **Push-to-talk (PTT) activation** | Without gating, open mic fires macros constantly; PTT is the simplest gate | Low | YES | Configurable hotkey; hold-to-listen semantics |
| **Wake-word / listen-toggle gate** | Alternative to PTT — say a keyword, system starts listening | Medium | YES | Must be decoupled from PTT; both modes needed |
| **Local (offline) speech recognition** | Privacy + latency + offline play; cloud dependency = deal-breaker for gaming | High | YES | Whisper.cpp or Vosk; model hot in RAM for sub-1s latency |
| **Phrase → key-sequence macro execution** | Core value: say word → emit key events | Medium | YES | Must deliver key press + release + timed sequences |
| **Wayland input injection** | Project is Wayland-first; X11 xdotool doesn't work on Wayland | High | YES | ydotool (uinput) or libei/XDG RemoteDesktop portal |
| **Configurable key-sequence timing** | HD2 stratagem entry requires fine timing; wrong delays = missed inputs | Low | YES | Per-command inter-key delay (ms-level control) |
| **Profile concept (named command sets)** | Users want different macro sets per game; profiles are universal in this space | Low | YES | Load/switch profiles; one active at a time |
| **Recognition log / feedback UI** | Users can't tune commands they can't see; log of "heard: X → fired: Y" is essential | Low | YES | Even a terminal log satisfies v1 |
| **Mic level indicator** | Must know if mic is live; otherwise debugging is impossible | Low | YES | Simple RMS meter or visual indicator |
| **Importable/exportable command packs** | Sharing HD2 stratagem packs is the killer v1 use case; manual re-entry is intolerable | Medium | YES | JSON/TOML pack format; import from file |

---

## Differentiators

Features not universally expected but strongly valued — competitive moats vs a shell script or a wine-VoiceAttack setup.

| Feature | Value Proposition | Complexity | v1? | Notes |
|---------|-------------------|------------|-----|-------|
| **Data-driven stratagem pack (Helldivers 2)** | Bundled, maintained, game-accurate pack with all ~80+ stratagems; instantly useful | Medium | YES — flagship | Separate data file from engine; versioned; community-updatable |
| **Built-in phrase editor** | Edit phrases + key sequences without touching JSON; non-developers can onboard | Medium | YES (minimal) | Can be CLI-based in v1; GUI is v2 |
| **Dynamic / optional phrase segments** | VA's `[Orbital;] EMS [Strike;]` syntax: required + optional words → fewer commands to author | Medium | v2 | Massively reduces authoring burden; complex to parse |
| **Confidence threshold control** | Tune false-positive vs false-negative tradeoff per command or globally | Low | YES | Essential for gaming where false triggers cause wipes |
| **Command chaining / sub-commands** | `open_strat_menu` → `key_sequence` → `close_strat_menu` as reusable sub-calls | Medium | YES (minimal) | HD2 profiles use this pattern heavily |
| **Wayland-native (no Wine/Proton layer)** | VoiceAttack via Wine is flaky (2489+ downloads of Wine workaround gists); native = reliability | High | YES — core | Primary technical differentiator vs running VA on Wine |
| **AGPL-licensed, community extensible** | No vendor lock-in; hobbyist community can fork/extend stratagem packs | Low | YES | License is decided; matters for pack ecosystem |
| **Multiple speech engine backends (pluggable)** | Whisper.cpp for GPU users, Vosk for CPU-only; swap without reconfiguring macros | High | v2 | v1 can ship one engine; architecture should allow swapping |
| **Conditions / if-then logic** | Run macro only if X is true (e.g., only fire if game window focused) | High | v2 | VA condition builder; needed for power users |
| **Variables and state** | Counters, toggles, stored values across macro executions | High | v2 | VA has 6 variable types; gaming use: track cooldowns, toggle modes |
| **Auto profile switching (window detection)** | Switch to HD2 profile when game window gains focus | Medium | v2 | VA does process/window-title matching |
| **Text-to-Speech confirmation** | Say "Eagle Airstrike confirmed" after macro fires — positional audio feedback | Low | v2 | Useful for HD2; low engineering cost once audio pipeline exists |
| **Soundboard / audio feedback** | Play a clip on recognition (e.g., HD2 ship radio crackle) | Low | v2 | Community delight feature |

---

## Anti-Features

Things to explicitly NOT build in v1 — scope traps that burn time without user value.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Full graphical profile editor (v1)** | GUI takes 3-5x longer than a minimal CLI/TUI; delays shipping the actual macro engine | Ship JSON editor + terminal preview; GUI in v2 |
| **Cloud speech recognition as primary path** | Privacy violation, adds latency, breaks offline play, dependency risk | Keep local-first; cloud = optional plugin later |
| **Windows/macOS client** | Doubles platform complexity; dilutes Linux focus | Linux-only in v1 per PROJECT.md |
| **Plugin/extension API (v1)** | VA's plugin system took years to stabilize; premature abstraction kills momentum | Hard-code extension points; formalize API in v2 |
| **Natural language / conversational AI integration** | "Hey computer, use the one that kills bugs" is cool but unacceptably slow/flaky for live gaming | Exact phrase matching only in v1 |
| **LLM-powered phrase disambiguation** | Latency is 300–2000ms; stratagem timing is <100ms window | Deterministic matching only |
| **Auto-update / OTA pack fetching** | Needs signing, CDN, rollback — ops complexity out of scope | Ship pack files; users download updated packs manually |
| **Voice training / personal model fine-tuning** | Whisper already generalizes well; training pipeline is a separate product | Use pretrained models; expose confidence tuning |
| **Gamepad / joystick voice-combo triggers** | Joystick + voice combos are a niche; adds input device complexity | Keyboard PTT is sufficient for v1 |

---

## Helldivers 2 Stratagem Pack — Specific Requirements

This section defines what the v1 bundled HD2 pack must look like. The pack is the flagship differentiator.

### Stratagem Coverage

| Category | Count (approx.) | Examples | Notes |
|----------|-----------------|----------|-------|
| Orbital strikes | ~10 | Orbital Laser, Orbital 380mm Barrage, Orbital Railcannon Strike | Codes change rarely |
| Eagle strikes | ~8 | Eagle Airstrike, Eagle Napalm, Eagle Cluster Bomb | "Eagle" prefix ambiguous with "Orbital" — needs distinct phrase design |
| Support weapons | ~15 | Autocannon, Railgun, Recoilless Rifle, Flamethrower | Many share prefix words (Rifle, Gun) — require unambiguous phrases |
| Sentries / emplacements | ~8 | Gatling Sentry, Mortar, Tesla Tower | |
| Backpacks | ~5 | Jetpack, Shield Generator, Supply Pack | |
| Vehicles | ~3 | Hellpod, Exosuit variants | Added post-launch |
| Mission support | ~4 | Resupply, Reinforce, SOS Beacon, SEAF Artillery | Highest frequency — must be fastest |

**Total: ~80+ stratagems** (game continues adding via patches; data file must survive engine updates).

### Pack Data Format Requirements

- **Machine-readable and human-editable** — JSON or TOML; not binary
- **Versioned** — `pack_version` + `game_version` fields; stale pack warnings on load
- **One stratagem per entry** with: `id`, `phrase[]` (multiple spoken variants), `key_sequence[]`, `inter_key_delay_ms`, `category`, `description`
- **Community-patchable** — users can override a single entry without replacing the whole file
- **Conflict detection** — tool should warn when two phrases share a prefix and could be ambiguous

### Key Sequence Implementation Notes (from VA community research)

VoiceAttack forum experience (HD2-specific):
1. **PRESS binding > HOLD binding** — Game's "press" mode for stratagem menu open is faster and more consistent than "hold"; macros using press require only one key event, not hold+release
2. **Arrow key remapping required** — Stratagem directionals must be remapped to arrow keys in-game for reliable macro injection; game's default WASD directionals interfere with movement input
3. **Quick Input method preferred** — Sending `[arrowD][arrowD][arrowU][arrowR]` as a sequence is faster and easier to maintain than calling individual sub-commands per arrow press
4. **Sub-10ms inter-key delays** work reliably once using PRESS mode; HOLD mode needed ~50-100ms hold durations
5. **Ambiguous phrase pairs to handle:** Eagle Smoke Strike vs Orbital Smoke Strike; Eagle vs Orbital prefixes on shared weapon names — phrases need required discriminating words

### Phrase Authoring Approach

| Approach | Description | HD2 Suitability |
|----------|-------------|-----------------|
| Exact phrase | "Orbital Railcannon Strike" | Safe, no ambiguity risk |
| Optional segments `[word;]` | Optional prefix/suffix words | Reduces authoring count; requires disambiguation pass |
| Required keyword only | "Railcannon" | Risky — "rail" sounds similar to other words |

**Recommendation for v1:** Exact phrases only — simpler engine, safer for live play. Optional segments in v2 when confidence tuning exists.

---

## Feature Dependencies

```
PTT / wake-word gate
    └─► Local speech recognition (engine must be running)
            └─► Phrase matching
                    └─► Key-sequence execution
                            └─► Wayland input injection

Profile concept
    └─► Pack import/export (a pack is a profile)
            └─► HD2 stratagem pack (a pack instance)

Command chaining / sub-commands
    └─► Conditions (conditions gate sub-command execution)
            └─► Variables (conditions read variables)

Recognition log
    └─► Confidence threshold control (log shows scores, threshold applied)
```

---

## MVP Recommendation (v1 Scope)

**Ship these — core loop is not working without them:**
1. Local Whisper.cpp speech recognition, model hot in RAM
2. PTT activation (configurable hotkey)
3. Exact-phrase → key-sequence macro execution
4. Wayland input injection (ydotool/uinput path; libei as stretch)
5. JSON stratagem pack format + bundled HD2 pack (all ~80 stratagems)
6. Confidence threshold (global, not per-command)
7. Profile load/switch
8. Recognition log (terminal output acceptable)
9. Configurable inter-key timing per command
10. Pack import from file

**Defer to v2 (do not block v1):**
- Dynamic/optional phrase segments
- Per-command confidence override
- Conditions and variables
- Auto profile switching by window
- TTS/audio confirmation
- GUI editor
- Multiple STT engine backends
- Soundboard

---

## Sources

- VoiceAttack Help PDF (voiceattack.com/VoiceAttackHelp.pdf) — HIGH confidence; official docs
- VoiceAttack HD2 community profile thread (forum.voiceattack.com/topic=4733) — HIGH confidence; real user experience
- voice-commander PyPI/GitHub (spyoungtech/voice-commander) — MEDIUM confidence; codebase inspection
- ydotool GitHub (ReimuNotMoe/ydotool) + libei (libinput.pages.freedesktop.org) — HIGH confidence; official sources
- hyprwhspr.com / Vocalinux STT comparison — MEDIUM confidence; current 2026 sources
- Helldivers 2 stratagem lists (divers.gg, thegamer.com) — MEDIUM confidence; game wikis (patch-sensitive)
- WebSearch — LOW confidence where not corroborated

# Domain Pitfalls: Linux Voice-Macro / Input-Injection for Gaming on Wayland

**Project:** hd-linux-voice
**Domain:** Linux desktop voice-control + input injection for gaming (Wayland-first, Helldivers 2 target)
**Researched:** 2026-04-21
**Overall confidence:** HIGH (Wayland/uinput), MEDIUM (anti-cheat risk), HIGH (audio), HIGH (distribution), HIGH (ASR latency)

---

## Severity Legend

| Symbol | Severity | Meaning |
|--------|----------|---------|
| 🔴 | **Critical** | Causes rewrite, blocks the core user journey, or gets users banned |
| 🟠 | **High** | Causes significant UX failures or requires painful architectural rework |
| 🟡 | **Medium** | Degrades experience; fixable without redesign |

---

## Critical Pitfalls

---

### 🔴 P-01 — Wayland Global Hotkey Dead Zone (Push-to-Talk Blocked by Game Focus)

**What goes wrong:**
Wayland's security model intentionally removes the X11 `XGrabKey` mechanism that allowed any application to register global hotkeys. When a game (via XWayland or native Wayland) holds keyboard focus and captures input, there is no standard mechanism for a background application to intercept the push-to-talk key. The macro app receives nothing while the game is in foreground.

**Why it happens:**
By design: Wayland compositors refuse to let arbitrary applications subscribe to global key events. The security model considers "other applications reading your keystrokes" a privacy violation. The `zwp_keyboard_shortcuts_inhibit_manager_v1` protocol goes the other direction (inhibit *compositor* shortcuts, not capture *application* input).

**Consequences:**
Push-to-talk is the primary interaction model for most users. If it silently stops working when the game takes focus, the entire product is broken during gameplay — the only time it matters.

**Warning signs (detect early):**
- Any approach using XLib's `XGrabKey` or GTK's global shortcut registration works on X11/XWayland wrapper windows but fails when tested inside a fullscreen Proton game
- Desktop-level hotkey libraries (e.g. `x11rb` global grab, `rdev`) work on GNOME/KDE desktops but produce no events during game

**Prevention:**
Read the physical keyboard device via `/dev/input/eventN` directly (evdev grab). This bypasses the Wayland security model because you are reading from the kernel input event device, not intercepting compositor-dispatched events. The daemon holds an exclusive `EVIOCGRAB` on the physical device, receives all key events regardless of focus, and re-injects non-PTT keys back via a separate uinput pass-through device.

**Phase/component to address:**
Input architecture phase — the push-to-talk subsystem must be designed around evdev grab from day one, not retrofitted. Document the `input` group membership requirement alongside the `uinput` group requirement.

---

### 🔴 P-02 — `/dev/uinput` Not Accessible by Default (Permissions Bootstrap)

**What goes wrong:**
`/dev/uinput` is owned by root with mode `crw-------` on most distributions out of the box. The `uinput` kernel module may not even be loaded. New installs silently fail to open the device — the app starts, speech is recognized, and nothing happens in the game.

**Why it happens:**
Linux distributions do not enable uinput for regular users by default for security reasons. This is correct behavior, but it means every installation requires two steps users won't know to perform: (1) load the kernel module, (2) configure udev and group membership.

**Consequences:**
Zero-install experience is broken. Users file bugs saying "nothing happens." Support burden is enormous without automated setup.

**Warning signs (detect early):**
- `open("/dev/uinput", O_WRONLY)` returns `ENOENT` (module not loaded) or `EACCES` (wrong permissions)
- Works when run as root but fails as a regular user
- Works on developer's machine (where setup was done manually) but fails on fresh VMs

**Prevention:**
Ship a post-install hook (systemd `.service` or `ExecStartPre=`) and udev rules file:
```
SUBSYSTEM=="misc", KERNEL=="uinput", GROUP="uinput", MODE="0660", TAG+="uaccess"
```
Install `uinput.conf` to `/etc/modules-load.d/` so the module loads at boot. The installer should also add the user to the `input` and `uinput` groups and print a clear message to log out and back in. Never silently continue when `/dev/uinput` can't be opened — fail fast with an actionable error linking to the setup documentation.

**Phase/component to address:**
Distribution / packaging phase. Also needs a runtime check in the app startup path with a human-readable error dialog.

---

### 🔴 P-03 — Flatpak Sandboxing Is Incompatible With uinput + evdev Grab

**What goes wrong:**
Flatpak's sandbox cannot grant `/dev/uinput` write access or `/dev/input/eventN` exclusive read access unless the *host* system already has the udev rules and group memberships configured — and even then, the `--device=all` flag required to expose these devices is a de-facto sandbox escape that portal reviewers reject for distribution. The microphone additionally requires `xdg-desktop-portal-pipewire` to be installed on the host.

**Why it happens:**
Flatpak is designed for GUI apps that access hardware through XDG portals (camera, microphone, location). Raw character device access (`/dev/uinput`, `/dev/input/*`) has no portal abstraction. The workaround (manual udev rules installed outside the sandbox) defeats the entire purpose of Flatpak for non-technical users.

**Consequences:**
If Flatpak is chosen as the primary distribution format, every user needs manual system configuration that Flatpak is supposed to eliminate. AppImage has a related but different problem: FUSE mounts are read-only, blocking `setcap` on the binary.

**Warning signs (detect early):**
- Flatpak app starts, requests microphone permission via portal, but `/dev/uinput` open fails inside container
- `--device=all` works in testing but is blocked by Flathub review policy
- Bug reports from users who installed via Flatpak but skipped the "install udev rules manually" step

**Prevention:**
Do **not** use Flatpak as the primary release format. Preferred distribution path for v1:
1. **Native packages** (`.deb` / `.rpm`) where the package `%post` script can install udev rules and load the kernel module
2. **AppImage** with a separate `install.sh` that handles privileged setup steps, clearly documented

Keep Flatpak as a future stretch goal if XDG portals eventually add uinput/evdev support.

**Phase/component to address:**
Distribution planning phase. Decide packaging format before building CI pipelines for it.

---

### 🔴 P-04 — XWayland Fullscreen Focus Race Causes Game to Minimize / Lose Input

**What goes wrong:**
Proton/Wine games run as XWayland clients. XWayland's handling of pointer and keyboard grabs interacts badly with some Wayland compositors during focus transitions, causing games to spontaneously:
- Self-minimize when another window (e.g. a daemon tray icon) briefly captures attention
- Lose relative mouse capture (FPS rotation stops working)
- Ignore all keyboard input until the game window is clicked again

The voice-macro daemon is an additional long-running process on the desktop — any UI it displays (system tray, notification, config window) can trigger this race.

**Why it happens:**
XWayland generates `FocusOut` events when Wayland sends input to a different client simultaneously. Fullscreen games use grab state tied to focus; losing focus breaks the grab, and games aren't designed to recover from mid-session focus loss.

**Consequences:**
User says a voice command → the daemon shows a notification → game minimizes → user must alt-tab back. The voice command may then fire in the wrong window. This is a game-breaking UX failure.

**Warning signs (detect early):**
- Running the daemon causes intermittent minimize during testing even without voice input
- Notifications or tray icon tooltip hover causes loss of game mouse capture
- Issue is compositor-specific: repros on Sway/Hyprland but not KDE Plasma

**Prevention:**
- Keep the daemon **headless by default** with no tray icon or floating windows unless explicitly opened by the user
- Use `gamescope` as the game compositor layer (strongly recommended for Helldivers 2) — gamescope isolates the game window from compositor focus events entirely
- Any config UI should be a separate process that the user explicitly launches/closes; it must not linger as a mapped window while gaming
- Test specifically by running the daemon + a fullscreen XWayland test app before testing with the real game

**Phase/component to address:**
UX/UI architecture phase. Core daemon design: headless daemon, separate UI process.

---

### 🔴 P-05 — End-to-End Latency Pipeline Blowout

**What goes wrong:**
Projects underestimate the cumulative latency: audio buffer fill → VAD trigger → recognition → post-processing → macro fire. Individual components seem fast in isolation. Combined, the pipeline can exceed 500–1500ms, which feels broken for real-time gaming ("I said Eagle Airstrike and it triggered two seconds later, in the wrong position").

**Why it happens:**
Each stage adds latency that compounds:
- Audio capture buffer: 50–200ms depending on block size
- VAD silence tail: 300–500ms of silence required to confirm end-of-speech
- Whisper inference: 100–800ms on CPU depending on model size and duration of audio
- Vosk streaming: lower latency but less accurate on game-specific vocabulary
- Key injection: negligible, but macro timing (hold/release) adds 100–300ms for strategems

Default configurations of Whisper (even `tiny`) are designed for transcription, not sub-200ms command detection.

**Warning signs (detect early):**
- Acceptable in quiet testing → unacceptable in actual gameplay
- Latency is consistent in testing but spikes during recognition of longer phrases
- VAD threshold tuned too conservatively (waits too long for silence before triggering)

**Prevention:**
- Instrument every stage with a timestamp log from day one
- Set a hard latency budget: **<300ms from end of last spoken syllable to first injected key event** for the default mode
- For Whisper: use `tiny` or `base.en` models; set short max audio duration; disable beam search (use greedy decoding)
- For Vosk: configure a restricted vocabulary/grammar for the command set; this dramatically speeds recognition
- Tune VAD aggressiveness and silence tail per use-case in the config
- Run the recognition model warm (pre-loaded, not cold-started per utterance)

**Phase/component to address:**
ASR backend selection phase AND a dedicated "latency profiling" milestone task before v1 release.

---

## High Pitfalls

---

### 🟠 P-06 — Unconstrained ASR Misrecognizes Gaming Vocabulary

**What goes wrong:**
General-purpose speech models (Whisper, Vosk default models) are trained on natural language corpora. Helldivers 2 strategem names ("Eagle Cluster Bomb", "Orbital Precision Strike", "Reinforce") are either ambiguous or out-of-distribution. The model transcribes something plausible that doesn't match any command → no action → user thinks the app is broken.

**Why it happens:**
Open vocabulary ASR finds the closest likely sequence given its training data. "Eagle Airstrike" might come back as "Eagle Air Strike" (variant spacing), "Eagle Air Strick", or something completely different in noisy conditions.

**Consequences:**
Core value proposition fails silently. Users see no error; the app seems to have ignored them. This erodes trust quickly.

**Warning signs (detect early):**
- WER (word error rate) on game-specific phrases is much higher than general English WER benchmarks
- Similar-sounding phrases trigger wrong commands

**Prevention:**
- Use a **constrained grammar / vocabulary list** approach, not open-ended transcription. For Vosk, configure `SetGrammar` with the exact recognized phrase set. This reduces the search space and dramatically improves accuracy and speed.
- For Whisper: use the `initial_prompt` parameter to bias toward expected terms, or implement a post-recognition matcher that maps recognized text → command via fuzzy/phonetic matching with a confidence threshold
- Allow per-command alternative phrases ("Reinforce" / "Reinforce call" / "call in support")
- Log what was recognized vs what matched; expose in debug mode

**Phase/component to address:**
ASR integration phase; command matching subsystem.

---

### 🟠 P-07 — Game Noise / Audio Bleed Causes VAD False Triggers

**What goes wrong:**
Game audio (explosions, voice lines, Discord call audio) leaks into the microphone — especially on headsets with poor isolation or open-back headphones. The VAD fires on game sounds. The recognition model then processes game audio and either produces garbled commands or silently fails. In the worst case, an in-game voice line accidentally triggers a strategem.

**Why it happens:**
On Linux, PipeWire/PulseAudio expose both physical microphone sources and "monitor" sources. An incorrectly selected input source (e.g. capturing from the game audio output monitor) will include all game sounds.

**Warning signs (detect early):**
- VAD fires intermittently when user is not speaking
- Recognized text during false triggers contains words from in-game dialogue

**Prevention:**
- Default to the physical microphone source, not any monitor source — filter out `monitor` sources in device enumeration
- Make the input device clearly configurable with a readable name (not ALSA card index)
- Implement a noise gate above the VAD to require sustained energy levels (not just any audio activity)
- Make VAD sensitivity configurable per-user
- Document headset/microphone recommendations prominently

**Phase/component to address:**
Audio capture subsystem; configuration/setup wizard.

---

### 🟠 P-08 — Anti-Cheat Detection of Virtual Input Devices

**What goes wrong:**
Anti-cheat systems can enumerate connected HID/input devices. A uinput-created virtual keyboard appears as an additional input device in `/proc/bus/input/devices` and in kernel input event lists. Some anti-cheat systems flag the presence of unexpected virtual input devices as suspicious, even if the inputs themselves are legitimate.

**Current risk for Helldivers 2:**
Helldivers 2 uses **nProtect GameGuard**, which runs in **userspace** under Proton (not kernel-level on Linux). Current community reports indicate it does not ban for Linux uinput usage, and the game works on Linux/Steam Deck. However:
- This is a soft guarantee — GameGuard behavior can change in updates
- Other games this project might expand to (BattlEye, EAC) carry higher risk
- Running as root to inject input is a much larger anti-cheat flag than using uinput with correct permissions

**Warning signs (detect early):**
- Game kicks to main menu after command injection
- Account flagged or banned (rare but catastrophic)
- Anti-cheat driver update causes regression

**Prevention:**
- Always use **user-level uinput** (never run the injection as root)
- Name the virtual device something neutral, not "voicemacro-bot-keyboard"
- Do not inject during loading screens or menus where timing looks inhuman (instant key sequences)
- Build in realistic key press dwell times (50–150ms) matching physical human input
- Monitor community reports after each GameGuard update; maintain a "tested versions" list in docs
- Design the architecture so input injection can be **disabled** without stopping recognition — useful if a specific game update breaks compatibility

**Phase/component to address:**
Input injection backend; game compatibility testing phase.

---

### 🟠 P-09 — uinput Key Repeat Double-Fire

**What goes wrong:**
When the `EV_REP` capability is advertised on a uinput virtual device AND userspace also sends its own repeat events, the kernel generates its own repeat events *in addition* to userspace-generated ones. The result is double key repeat — macro sequences fire twice or at double speed.

**Why it happens:**
The kernel automates `EV_REP` key repetition when the capability bit is set on a device. If the application also sends explicit repeat events, both fire simultaneously.

**Warning signs (detect early):**
- Strategem sequences produce double key presses or extra characters in any text field
- Only manifests with longer held keys, not single-press events

**Prevention:**
When creating the uinput virtual keyboard device, explicitly **do not set `EV_REP`** in the capabilities bitmask. Only report `EV_KEY` and `EV_SYN`. Send only explicit `KEY_PRESS` (value=1) and `KEY_RELEASE` (value=0) events followed by `SYN_REPORT`. Do not send `KEY_REPEAT` (value=2) events from userspace.

**Phase/component to address:**
uinput device creation code; tested in the key injection unit test suite.

---

### 🟠 P-10 — Strategem Macro Timing Too Fast for Game Input Buffer

**What goes wrong:**
Helldivers 2 strategems require specific directional key sequences (e.g. Down, Right, Up, Left for an Eagle). Injecting these at machine speed (microsecond intervals) causes the game to drop inputs — the input buffer doesn't process all events, or the game's frame rate creates a sampling window that misses some keys.

**Why it happens:**
Games sample input at their own frame rate (often 60Hz = 16.6ms per frame). An input event injected and released within the same frame may not register. Physical humans hold keys for 50–200ms naturally.

**Warning signs (detect early):**
- Strategems sometimes fail to call the correct result even though the recognized phrase was correct
- Failure rate is higher at high frame rates
- Manually pressing the sequence works but voice-injected sequence misses steps

**Prevention:**
- Default key dwell time: **80–120ms per directional key** for strategem sequences
- Default inter-key gap: **30–50ms** between key release and next key press
- Make these configurable per-profile and per-command
- Test with a running Helldivers 2 instance during development, not just a key-logger

**Phase/component to address:**
Macro execution engine; Helldivers 2 strategem pack validation.

---

### 🟠 P-11 — PipeWire/PulseAudio/ALSA Enumeration Fragmentation

**What goes wrong:**
Linux audio is in a transitional era: most systems use PipeWire (which also emulates PulseAudio), some run native PulseAudio, some use ALSA directly. The same hardware surfaces as different device names and indices across these backends. A hardcoded ALSA device name that works on the developer's machine fails on a user running PulseAudio, and vice versa.

Additionally:
- Bluetooth microphones require `xdg-desktop-portal-pipewire` to be installed for portal-based access
- PipeWire's float32le audio format negotiation fails with modules expecting integer formats
- Device indices can change between reboots (ALSA)

**Warning signs (detect early):**
- "It works on my machine" — developer uses PipeWire, tester uses PulseAudio
- Bluetooth headset mic works with other apps but not the voice-macro daemon
- Device selects the wrong microphone after OS update

**Prevention:**
- Use a library that abstracts audio backends: **CPAL** (Rust) handles PipeWire, PulseAudio, and ALSA through a unified interface
- Enumerate devices by name, not index; persist the selected device name in config
- Test on both a PipeWire system and a native PulseAudio system in CI
- Do not depend on any specific sample rate or format — accept what the device offers and resample if needed
- Document Bluetooth microphone requirements (portal must be installed)

**Phase/component to address:**
Audio capture subsystem. Treat backend abstraction as a first-class design constraint.

---

### 🟠 P-12 — AGPL License Compatibility of ASR Model Dependencies

**What goes wrong:**
The project is AGPL-3.0. Third-party libraries and models included in distribution must be AGPL-compatible. Speech model licenses are often separate from their inference engine licenses and may include non-commercial or "research only" restrictions that are incompatible with public distribution.

**Known landscape:**
| Component | License | AGPL-compatible? |
|-----------|---------|-----------------|
| whisper.cpp (engine) | MIT | Yes |
| OpenAI Whisper models | MIT-like (cc0/MIT) | Yes — verify per variant |
| Vosk API (engine) | Apache 2.0 | Yes |
| Vosk models | Apache 2.0 (most) | Yes — verify per model |
| Coqui STT engine | MPL 2.0 | Yes (weak copyleft, compatible) |
| Coqui TTS models | Various — some CC-NC | **No** — CC non-commercial incompatible |
| Silero VAD | MIT | Yes |
| OpenWakeWord | Apache 2.0 | Yes |

**Warning signs (detect early):**
- A model download URL points to a `.gguf` with no accompanying license file
- Using a "fine-tuned" Whisper variant from HuggingFace without checking the model card
- A dependency is a Python wheel that wraps a binary without clear license disclosure

**Prevention:**
- Maintain a `LICENSES.md` or SBOM from the first dependency decision
- Audit model licenses separately from inference engine licenses — they are different artifacts
- For bundled or downloaded models, check the original model card on HuggingFace or the official project site
- Default to the OpenAI upstream Whisper models (MIT) or confirmed Apache Vosk models
- Add a CI check that fails if new dependencies are added without a license entry

**Phase/component to address:**
Dependency selection phase; legal review checkpoint before first public release.

---

## Medium Pitfalls

---

### 🟡 P-13 — Whisper Model Download on First Launch (Bad Cold-Start UX)

**What goes wrong:**
If the application downloads the speech model on first run without warning, users see a progress bar downloading 75MB–1.5GB before anything works. On slow connections this takes minutes. The app appears broken.

**Prevention:**
- Bundle the smallest viable model (`whisper-tiny.en` at ~75MB) with the package, or
- Show a setup wizard on first launch that explains the download, estimates time, and allows model selection
- Never silently block on a model download during startup; make it an explicit step

**Phase/component to address:**
Distribution / first-run experience.

---

### 🟡 P-14 — ydotool / wtype Daemon Dependency (Running Services the User Didn't Opt Into)

**What goes wrong:**
Tools like `ydotool` require a background daemon (`ydotoold`) to be running. If this daemon is not started, all injection silently fails. Running it as a system-level root service causes ghost input events and instability on some compositors (reported on Ubuntu 24.04).

**Prevention:**
Do not use `ydotool` or `wtype` as the injection mechanism. Use the **`uinput` kernel interface directly** — it requires no daemon, no external process, and no root (only the correct group membership). This eliminates a whole class of daemon lifecycle problems.

**Phase/component to address:**
Input injection backend selection.

---

### 🟡 P-15 — Compositor-Specific Behavior Divergence

**What goes wrong:**
GNOME (Mutter), KDE Plasma (KWin), Sway, Hyprland, COSMIC, and labwc each implement the Wayland protocol with different extensions and different behavior around input grabs, pointer lock, and XWayland interop. A feature that works on KDE Plasma may silently fail on Sway.

**Warning signs (detect early):**
- Push-to-talk works on GNOME/X11 fallback but not Sway pure Wayland
- Mouse capture works in testing on KDE but breaks on Hyprland

**Prevention:**
- Define a supported compositor matrix early (e.g., "supported: GNOME Wayland, KDE Plasma Wayland, Sway; best-effort: Hyprland, COSMIC")
- Test on at least GNOME and Sway in CI/CD (e.g., using a headless compositor)
- Use `uinput`-based injection (kernel-level) rather than compositor protocol-level injection — kernel input bypasses compositor-specific behavior for injection

**Phase/component to address:**
Integration testing phase; compatibility matrix documentation.

---

### 🟡 P-16 — Audio Capture Blocking the Recognition Thread

**What goes wrong:**
If audio capture and ASR inference share a thread, audio buffer overruns cause dropped audio frames while inference runs. The next utterance is silently lost because the capture buffer overflowed.

**Prevention:**
- Run audio capture in a dedicated thread/task with a ring buffer
- Run ASR inference in a separate thread consuming from the ring buffer
- Monitor buffer fill level; log warnings if > 80% full
- Use non-blocking audio capture with callback-based APIs

**Phase/component to address:**
ASR pipeline architecture.

---

### 🟡 P-17 — `uinput` Module Not Persistent Across Reboots

**What goes wrong:**
The user adds themselves to the `uinput` group and loads `modprobe uinput`, and everything works. After reboot, `/dev/uinput` is gone (module not loaded) and the app silently fails.

**Prevention:**
- Write a `/etc/modules-load.d/uinput.conf` file during installation containing just `uinput`
- Package post-install hook must do this — do not rely on users doing it manually
- Startup check: if `/dev/uinput` doesn't exist, print an actionable error that includes the exact fix command

**Phase/component to address:**
Packaging / install phase.

---

## Phase-Specific Warning Matrix

| Phase / Feature | Likely Pitfall | Severity | Mitigation |
|----------------|---------------|----------|------------|
| Input backend selection | Choosing ydotool over uinput direct | 🟡 | Use uinput directly (P-14) |
| PTT / wake word design | Global hotkey deaf in-game | 🔴 | evdev grab on physical device (P-01) |
| Audio capture subsystem | PipeWire/PA format mismatch, Bluetooth mic | 🟠 | CPAL abstraction (P-11) |
| ASR engine selection | Open vocabulary on gaming jargon | 🟠 | Constrained grammar (P-06) |
| ASR pipeline threading | Buffer overrun drops utterances | 🟡 | Dedicated capture thread (P-16) |
| VAD tuning | Game noise false triggers | 🟠 | Noise gate + configurable threshold (P-07) |
| Latency profiling | Pipeline compounds to >1s | 🔴 | Measure every stage, budget 300ms (P-05) |
| Macro execution engine | Too-fast injection drops inputs | 🟠 | Configurable dwell time (P-10) |
| uinput virtual device setup | EV_REP double-fire | 🟠 | Omit EV_REP capability (P-09) |
| uinput device naming | Anti-cheat flags virtual device | 🟠 | Neutral name, user-level only (P-08) |
| Game compatibility testing | nProtect update breaks injection | 🟠 | Monitor + maintain tested-versions list (P-08) |
| Distribution / packaging | uinput inaccessible in Flatpak | 🔴 | Use .deb/.rpm + AppImage instead (P-03) |
| Distribution / packaging | Module not persistent after reboot | 🟡 | modules-load.d in post-install (P-17) |
| First-run UX | Silent failure from missing udev/group | 🔴 | Startup check with actionable error (P-02) |
| Model download | 1.5GB surprise on first launch | 🟡 | Bundle tiny model or setup wizard (P-13) |
| Dependency licensing | ASR model with non-commercial clause | 🟠 | License audit per model (P-12) |
| Compositor compatibility | Focus race minimizes game | 🔴 | Headless daemon + gamescope (P-04) |
| Compositor compatibility | Behavior diverges across compositors | 🟡 | Supported matrix + CI testing (P-15) |

---

## Sources

- [Wayland input injection security model — ydotool issues](https://github.com/ReimuNotMoe/ydotool/issues/285) — HIGH confidence
- [Flatpak uinput access limitations](https://github.com/flatpak/flatpak/issues/4137) and [security discussion](https://github.com/flatpak/flatpak/issues/5459) — HIGH confidence
- [XWayland fullscreen focus race — COSMIC compositor](https://github.com/pop-os/cosmic-comp/issues/1874) — HIGH confidence
- [XWayland keyboard focus hijack — Sway](https://github.com/swaywm/sway/pull/8699) — HIGH confidence
- [EV_REP double repeat — evsieve issue](https://github.com/KarsMulder/evsieve/issues/36) — MEDIUM confidence
- [uinput permissions guide — Fedora Discussion](https://discussion.fedoraproject.org/t/implications-of-change-permissions-on-uinput/128865/5) — HIGH confidence
- [Vosk single-threaded, noise sensitivity, gaming study](https://www.preprints.org/manuscript/202505.0855/v1) — MEDIUM confidence (single study)
- [whisper.cpp streaming + VAD](https://github.com/ggml-org/whisper.cpp/blob/master/examples/stream/README.md) — HIGH confidence
- [Helldivers 2 — nProtect GameGuard Linux/Proton](https://bbs.archlinux.org/viewtopic.php?pid=2230490) — MEDIUM confidence (community reports)
- [PipeWire Flatpak Bluetooth mic portal requirement](https://huzi.pk/blog/tech/pipewire-flatpak-bluetooth-mic-permissions) — HIGH confidence
- [LinVAM (prior art: Linux voice macro)](https://github.com/stele95/LinVAM) — HIGH confidence (existing project with same pitfalls)
- [VoxInput (prior art: voice → uinput)](https://github.com/richiejp/VoxInput) — HIGH confidence
- [AntiMicroX uinput wiki (manual udev workaround)](https://github.com/AntiMicroX/antimicrox/wiki/Open-uinput-error) — HIGH confidence
- [uinput kernel documentation](https://www.kernel.org/doc/html/v4.16/input/uinput.html) — HIGH confidence