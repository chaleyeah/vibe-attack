# STACK

Technology stack for `hd-linux-voice` — a Linux-only voice-macro daemon for Helldivers 2.

## Language & Runtime

- **Language:** Rust (edition 2021)
- **Async runtime:** `tokio` 1.x (`features = ["full"]`) — used for signal handling and the UDS control listener; the audio/VAD/STT/inject hot paths run on dedicated `std::thread` OS threads, not the Tokio executor.
- **Target platform:** Linux only. Hard dependencies on `/dev/uinput`, `/dev/input/event*`, evdev, ALSA/PipeWire, `libonnxruntime.so`, and XDG base directories.

## Build & Toolchain

- **Build tool:** `cargo` (Rust stable toolchain).
- **Lockfile:** `Cargo.lock` is committed (see `packaging/PKGBUILD` building with `--locked`).
- **Manifest:** `Cargo.toml` (single-crate workspace; no virtual workspace).
- **Vendored crate:** `vendor/whisper-rs` is patched in via `[patch.crates-io]` so the project always builds against the local fork rather than the published `whisper-rs`.
- **Cargo features:**
  - default — no extras (CPU-only baseline).
  - `stt` — pulls in `whisper-rs` for speech-to-text.
  - `stt-vulkan` — `stt` plus `whisper-rs/vulkan` GPU acceleration.
  - `gui` — pulls in `eframe` and enables the second binary `hd-linux-voice-config`.

## Binaries

- `hd-linux-voice` — main daemon (`src/main.rs`).
- `hd-linux-voice-config` — egui-based first-run/config GUI, gated behind `gui` feature (`src/bin/hd-linux-voice-config.rs`).

## Key Frameworks & Libraries (versions from `Cargo.toml`)

### Audio / DSP
- `cpal = "0.15"` — cross-platform audio capture (used in ALSA/PipeWire mode).
- `silero-vad-rust = "6.2.1"` — Silero voice activity detection (ONNX).
- `ort = "=2.0.0-rc.10"` — pinned ONNX Runtime bindings (used by `silero-vad-rust`).
- `sherpa-onnx = "1.12.39"` (`default-features = false`, `features = ["shared"]`) — wake-word keyword spotting; the `shared` feature links against a shared `libonnxruntime.so` so both `ort` and `sherpa-onnx` use one ORT instance.
- `whisper-rs = "0.16.0"` (optional, vendored fork) — speech-to-text via `whisper.cpp`.
- `rodio = "0.17"` — audio playback for per-macro feedback sounds.

### Input / OS Integration
- `evdev = "0.13.2"` (with `serde`) — read PTT key events and emit synthetic key events via uinput.
- `xdg = "3.0.0"` — XDG base-directory paths (config, runtime socket, profiles).
- `tokio-util = "0.7"` — `CancellationToken` for cooperative shutdown.

### Concurrency
- `crossbeam-channel = "0.5.15"` — bounded MPSC channels between pipeline stages.
- `ringbuf = "0.4.8"` — lock-free SPSC ring buffer fed by the CPAL real-time callback.
- `std::sync::mpsc` — used for the macro injection command channel.

### Configuration & Serialization
- `serde = "1"` (with `derive`).
- `serde_yaml_ng = "0.10"` — YAML for `config.yaml` and pack files.
- `serde_json = "1.0.149"` — stdout JSONL transcripts and UDS control protocol.

### CLI / TUI / GUI
- `clap = "4"` (`derive`) — CLI parsing for the daemon and subcommands.
- `ratatui = "0.26"` + `crossterm = "0.27"` — interactive TUI editor (`hd-linux-voice edit`).
- `eframe = "0.31"` (optional, `default-features = false`, features `default_fonts` and `glow`) — GUI for the config binary.

### Logging / Errors
- `tracing = "0.1"` + `tracing-subscriber = "0.3.23"` (`env-filter`) — structured logs to **stderr only** (stdout is reserved for JSONL).
- `anyhow = "1"` — error context.
- `thiserror = "1"` — typed `DaemonError` variants (`src/error.rs`).

### Misc
- `strsim = "0.11"` — Levenshtein-based fuzzy phrase matching for the dispatcher.
- `zip = "0.6"` — read/write `.hdpack` profile archives.
- `sha2 = "0.10"` — hashing for pack content (used in tests/utilities).

## Dev Dependencies

- `tempfile = "3"` — scratch dirs in tests.

## Distribution / Packaging

- `cargo-about` configured via `about.toml` / `about.hbs` for license aggregation; full license bundle committed at `LICENSES.md`.
- Arch Linux: `packaging/PKGBUILD`.
- AppImage: `packaging/appimage/build.sh` + `hd-linux-voice.desktop`.
- License: AGPL-3.0-only.
