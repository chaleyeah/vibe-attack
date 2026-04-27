# Contributing to vibe-attack

Thank you for your interest in contributing. This guide covers how to build, test, and submit changes.

## Prerequisites

- **Rust stable** toolchain (`rustup toolchain install stable`)
- **libasound2-dev** and **libclang-dev** (Debian/Ubuntu) or **alsa-lib** and **clang** (Arch)
- **evdev/uinput access** — your user must be in the `input` group or have appropriate udev rules (see [docs/uinput-setup.md](docs/uinput-setup.md))

## Building

Default build (no optional features):
```bash
cargo build
```

With egui config window (requires a display server):
```bash
cargo build --features gui
```

With Whisper speech-to-text:
```bash
cargo build --features stt
```

Release build:
```bash
cargo build --release
```

## Running Tests

```bash
cargo test
```

All tests pass without hardware present — hardware-dependent paths are gated behind feature flags or skipped when devices are absent.

## Architecture

The daemon is a two-stage pipeline:

```
Microphone → VAD → (audio chunks) → STT → (transcript) → Dispatch → Key injection
```

- **VAD** (Voice Activity Detection) gates audio capture around the push-to-talk key.
- **STT** runs Whisper inference inside `tokio::task::spawn_blocking` to avoid blocking the async runtime.
- **Dispatch** matches transcripts against the loaded macro pack and sends keystrokes via uinput.
- **stdout** is reserved for machine-readable JSONL transcripts. Never write anything else to stdout.

## Coding Conventions

- **No allocations in the audio callback.** The CPAL callback runs on a real-time thread; heap allocation can cause xruns. Pre-allocate all buffers before the stream starts.
- **STT must use `spawn_blocking`.** Whisper inference is CPU-bound and must not run on a tokio worker thread.
- **stdout is reserved for JSONL.** All diagnostics go to stderr via `tracing`. Nothing should `println!()` except the JSONL output path.
- Use `anyhow` for error propagation in application code; `thiserror` for library error types.
- Follow existing module structure: `audio`, `vad`, `wake`, `stt`, `input`, `pipeline`, `pack`, `control`, `config`, `ui`, `tui`.

## Pull Request Process

1. Fork the repo and create a branch from `main`.
2. Make your changes with appropriate tests.
3. Run `cargo test` and `cargo clippy --all-targets -- -D warnings` — both must pass.
4. Open a PR with a clear description of what changed and why.

For pack/macro format authoring, see [docs/pack-format.md](docs/pack-format.md) (coming soon).
