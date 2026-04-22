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
