# INTEGRATIONS

External dependencies and integrations for `hd-linux-voice`. Everything is **local-first**: no third-party APIs, no telemetry, no network calls at runtime.

## Third-Party APIs / Network Services

- **None at runtime.** The daemon performs no outbound network requests in normal operation.
- Model artifacts (Whisper, Silero VAD, sherpa-onnx KWS) are **never downloaded automatically**. The user obtains them out-of-band and points to local paths in `config.yaml`. `Config::validate_model_paths` (`src/config.rs:27`) fails fast at startup if any referenced model file is missing.

## Speech & Audio Stack

- **Audio capture (CPAL):** `cpal::default_host()` selects the platform host; on Linux this is ALSA, which is typically routed through PipeWire's ALSA bridge. The capture stream runs at the device's native rate; if non-16 kHz, the CPAL callback resamples linearly to 16 kHz mono before pushing into the ringbuf (`src/audio/mod.rs:106`).
- **VAD (Silero):** loaded via `silero-vad-rust` → `ort` → `libonnxruntime.so`. CPU-only path is forced (`force_onnx_cpu: true`) at `src/pipeline/coordinator.rs:252`.
- **Wake word (sherpa-onnx):** keyword spotter built from local `encoder.onnx`/`decoder.onnx`/`joiner.onnx`/`tokens.txt`/`keywords.txt`. BPE bundles auto-detect a sibling `bpe.model` (`src/wake/mod.rs:55`).
- **STT (whisper.cpp via `whisper-rs`):** uses a vendored fork at `vendor/whisper-rs`. Greedy decoding with `best_of: 1`. Optional `initial_prompt` from config is injected per-utterance to bias toward the active stratagem vocabulary. Whisper state is created once and reused across jobs to avoid heap fragmentation (`src/stt/mod.rs:151`).
- **Audio feedback (`rodio`):** per-macro WAV playback via `rodio::Decoder`/`Sink`. Failures are logged but never abort dispatch (`src/pipeline/sound.rs`).

### ONNX Runtime Coexistence

`sherpa-onnx` (with `features = ["shared"]`) and the `ort` crate used by `silero-vad-rust` must share **one** `libonnxruntime.so` to avoid the dual-ORT crash. The pipeline auto-discovers the sherpa-shipped `.so` and sets `ORT_DYLIB_PATH` before any pipeline thread is spawned (`src/pipeline/coordinator.rs:235`). See `MEMORY.md → project_dual_ort_conflict.md` for context on the existing wake-word disable-by-default decision.

## Operating-System Integrations (Linux)

- **`/dev/uinput`** — the daemon opens `/dev/uinput` via `evdev::uinput::VirtualDevice` to inject keystrokes (`src/input/inject.rs:111`). On `EACCES` it returns the actionable `DaemonError::UinputPermissionDenied` message instructing the user to add themselves to the `input` group.
- **`/dev/input/event*`** — scanned non-exclusively for the configured PTT key. Game still receives the key. A preflight check (`src/input/ptt.rs:35`) ensures readability before the daemon proceeds.
- **uinput kernel module** — required (`modprobe uinput`).
- **`input` Linux group** — required for `/dev/uinput` and `/dev/input/event*` access. Note: messages explicitly say `input` (not `uinput`) because systemd v258+ ignores non-system groups in udev rules (`src/error.rs:8`).
- **POSIX signals** — `tokio::signal::unix` listens for `SIGTERM` and `SIGINT` (`src/main.rs:271`) for graceful shutdown.

## Persistence Layer

- **No database.** All persistent state is plain files under XDG dirs.
- **YAML config:** `$XDG_CONFIG_HOME/hd-linux-voice/config.yaml` (default), or path provided via `--config` (`src/config.rs:280`).
- **Profile manager state:** `$XDG_CONFIG_HOME/hd-linux-voice/manager.yaml` records the active profile name (`src/pack/manager.rs`).
- **Profiles directory:** `$XDG_CONFIG_HOME/hd-linux-voice/profiles/<name>/` — each contains `pack.yaml` and an optional `sounds/` subdirectory.
- **`.hdpack` archives:** ZIP files (CompressionMethod::Stored) bundling `pack.yaml` and `sounds/` for sharing profiles. Path-traversal protection on import via `ZipFile::enclosed_name` (`src/pack/mod.rs:78`).

## Inter-Process Control

- **Unix Domain Socket:** the daemon binds a UDS at `$XDG_RUNTIME_DIR/hd-linux-voice/hd-linux-voice.sock` with mode `0600`. It accepts newline-delimited JSON requests (`Ping`, `SwitchProfile`, `TestMacro`, `Shutdown`) defined in `src/control/protocol.rs`. The CLI subcommands (`ping`, `switch`, `test`) connect via `src/control/client.rs` and emit `ControlResponse` JSON (`src/control/mod.rs:15`).

## Authentication / Identity Providers

- **None.** The daemon runs as a normal user; OS group membership (`input`) is the only authorization mechanism.

## Telemetry / Observability

- **Logs:** `tracing` → stderr only (stdout is reserved for JSONL transcripts).
- **Stdout JSONL contract:** `JsonlEvent` variants `utterance`, `stage`, `status`, `dispatch`, `no_match` (`src/pipeline/jsonl.rs`). Verbosity is `summary` or `stages` per `pipeline.verbosity` config knob.
- **No metrics export, no tracing exporter, no crash reporter.**

## Infrastructure / Deployment

- **No remote infrastructure.** Single-machine, single-user daemon.
- **Distribution targets:**
  - Cargo source build (primary).
  - Arch Linux: `packaging/PKGBUILD` (`makedepends = rust cargo`, `depends = alsa-lib`).
  - AppImage: `packaging/appimage/build.sh` + `hd-linux-voice.desktop` for desktop integration.

## Communication Services

- **None.** No email, messaging, push notifications, or webhooks.
