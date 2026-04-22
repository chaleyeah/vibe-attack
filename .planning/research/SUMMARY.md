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
