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
