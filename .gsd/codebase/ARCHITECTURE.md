# ARCHITECTURE

`hd-linux-voice` is a **single-binary Rust daemon** organized as a real-time audio pipeline with strict thread-topology discipline. The codebase is one Cargo crate; modules are internal boundaries, not separate packages.

## Architectural Style

- **Local daemon, single process.** No microservices, no IPC fanout. A second binary (`hd-linux-voice-config`, behind the `gui` feature) is a sibling configuration UI, not a service component.
- **Pipeline / staged-event-driven.** Audio flows through fixed stages connected by bounded queues. Each stage runs on its own dedicated thread, chosen for the workload (RT callback, blocking I/O, or compute).
- **Library + binary split.** `src/lib.rs` re-exports all modules so both `src/main.rs` (daemon) and `src/bin/hd-linux-voice-config.rs` (GUI) and the tests in `tests/` consume the same `hd_linux_voice::*` namespace.
- **Async only at the edges.** `tokio` is used for signal handling and the UDS control listener. Hot paths (audio capture, VAD, STT, key injection, PTT polling) run on **`std::thread`** OS threads — never on the Tokio executor.

## Core Data Flow

```
┌──────────────┐   f32 samples     ┌─────────────────┐  20-ms frames    ┌────────────────┐
│ CPAL RT cb   │ ─push (no alloc)─▶│ HeapRb<f32>     │ ─drain (poll)──▶ │ Pipeline thread│
│ (audio mod)  │   resample to     │ pre-allocated   │                  │ wake + VAD     │
└──────────────┘   16 kHz mono     │ 5 s capacity    │                  └───────┬────────┘
                                   └─────────────────┘                          │ UtteranceJob
                                                                                ▼
       ┌──────────────────┐  ptt_active (AtomicBool)         ┌────────────────────────┐
       │ PTT thread       │ ───────────────────────▶         │ STT thread (whisper)   │
       │ evdev poll       │                                  │ bounded queue (cap 4)  │
       └──────────────────┘                                  └───────────┬────────────┘
                                                                         │ SttResult
                                                                         ▼
                                                              ┌──────────────────────┐
                                                              │ Dispatcher thread    │
                                                              │ phrase match + flags │
                                                              │ + sound trigger      │
                                                              └─┬──────────┬─────────┘
                                                                │MacroCmd  │OutputMsg
                                                                ▼          ▼
                                                  ┌────────────────────┐  ┌────────────────────┐
                                                  │ Injection thread   │  │ Output thread      │
                                                  │ uinput VirtualDev  │  │ JSONL on stdout    │
                                                  └────────────────────┘  └────────────────────┘
```

Sources: `src/main.rs`, `src/audio/mod.rs`, `src/pipeline/coordinator.rs`, `src/input/{ptt,inject}.rs`, `src/stt/mod.rs`, `src/wake/mod.rs`, `src/vad/mod.rs`, `src/pipeline/{dispatcher,jsonl}.rs`.

### Thread topology (locked decisions, see code-doc references like `D-01`, `D-04`, `D-07`)

| Thread | Owner | Responsibility |
|---|---|---|
| Tokio main | `tokio::main` (`src/main.rs:69`) | startup orchestration, signal wait, control listener |
| Audio RT callback | `cpal::Stream` (kept on **main**) | push samples to `HeapRb` — no alloc, no logging, no blocking |
| Pipeline | `std::thread` (`coordinator::spawn_pipeline`) | drain ringbuf → wake spotter → VAD segmentation → enqueue STT jobs |
| STT | `std::thread` (`SttService::spawn`) | whisper inference, drop-oldest input queue (cap 4), output queue (cap 8) |
| Dispatcher | `std::thread` (inside `spawn_pipeline`) | phrase match transcripts → emit `MacroCmd::Execute` + `OutputMsg::Dispatched`/`NoMatch` |
| Output | `std::thread` (inside `spawn_pipeline`) | sole stdout writer; serializes `JsonlEvent`s |
| PTT | `std::thread` (`spawn_ptt_thread`) | blocking `evdev::Device::fetch_events`, updates `Arc<AtomicBool>` ptt_active |
| Injection | `std::thread` (`spawn_injection_thread`) | consumes `MacroCmd` from `mpsc`, emits keys via uinput with dwell/gap timing |
| Control listener | Tokio task | UDS accept loop, spawns per-connection task |

### Critical thread-topology constraints (from inline comments)

- **`cpal::Stream` MUST stay on the main thread.** Moving it into a worker that drops it later silently stops ALSA/PipeWire callbacks (`src/audio/mod.rs:24`, `src/main.rs:240`). The pipeline only receives the `HeapCons<f32>` consumer half.
- **No allocation, no logging, no blocking inside the CPAL callback.** First-callback notice is written via raw `writeln!` to stderr to bypass `tracing`'s mutex (`src/audio/mod.rs:163`).
- **`VirtualDevice::emit` auto-appends `SYN_REPORT`** — manual SYN sends garble events (`src/input/inject.rs:142`).
- **`ORT_DYLIB_PATH` must be set before pipeline threads start** — see "Process-wide setup" below.

## Pipeline Modes

The pipeline supports two activation paths, gated by config flags:

1. **PTT mode** — when `ptt_active` is true, the pipeline bypasses VAD speech-gating and accumulates raw audio into `ptt_audio`. On release, the entire buffer (≥ 100 ms) is shipped to STT (`coordinator.rs:351`). VAD state is reset after each PTT utterance to prevent bleed-over.
2. **Wake-word + VAD mode** — when idle, frames feed the sherpa-onnx keyword spotter and a 600 ms rolling pre-roll ring. Wake hits open a `LISTENING` window (default 5 s, configurable). During the window, a tuned VAD segmenter (shorter `end_silence_ms`, `min_speech_ms`, `max_utterance_secs`) runs; if it fails to close after `WAKE_FORCE_FLUSH_MS` (1200 ms), the segmenter is force-flushed.

Both paths emit identical `UtteranceJob`s downstream, so STT and dispatch logic is mode-agnostic.

## Bounded Queues & Backpressure

- `HeapRb<f32>` of 80 000 samples (5 s @ 16 kHz) between RT callback and pipeline (`src/audio/mod.rs:22`).
- `crossbeam-channel::bounded::<SttMsg>(4)` for VAD → STT.
- `crossbeam-channel::bounded::<SttResult>(8)` for STT → dispatcher.
- `crossbeam-channel::bounded::<SttResult>(8)` for dispatcher → output.
- `crossbeam-channel::bounded::<OutputMsg>(16)` for the multiplex of utterance/dispatch/no-match events to output.
- `std::sync::mpsc` (unbounded) for the macro injection command channel — small and fast.

**Drop-oldest semantics** are implemented in `vad::try_send_drop_oldest` (`src/vad/mod.rs:27`): on a full queue, one oldest item is consumed and the new item retried once. This is reused in both VAD→STT and STT→dispatcher paths to preserve responsiveness under load.

## Key Design Patterns

- **Fail-fast preflight.** All model paths, `/dev/input` readability, key parsing, and `/dev/uinput` opening happen on the main thread before any worker starts (`src/main.rs:178–207`). On any failure, the daemon exits with a copy-pasteable remedy.
- **Pre-allocated ring buffer.** RT-safety pattern — the audio callback must never allocate or block.
- **RAII resource ownership.** `StreamGuard` wraps `cpal::Stream`; dropping it stops capture. Held on main and dropped last (`src/audio/mod.rs:32`, `src/main.rs:315`).
- **Cancellation token (`tokio_util::sync::CancellationToken`).** Threads check `is_cancelled()` between event batches and exit cooperatively (`src/input/ptt.rs:120`, others). Best-effort 500 ms timed joins on shutdown for threads that may block on external I/O (`src/main.rs:299–311`).
- **`#[serde(deny_unknown_fields)]`** on every config struct — typos in YAML fail at parse time rather than silently no-op (`src/config.rs:8`, etc.).
- **Stable JSONL contract.** `JsonlEvent` is the public-facing wire format. The output thread is the *only* writer to stdout; logs go to stderr (`src/main.rs:56`).
- **Catch ORT panics as actionable errors.** `std::panic::catch_unwind` wraps Silero VAD load to convert ORT panics into a config-level "install onnxruntime" message (`src/pipeline/coordinator.rs:252`).
- **Profile = pack.** A "profile" is a directory with `pack.yaml` and optional `sounds/`. `.hdpack` is just a ZIP of that directory (`Stored` compression). `Pack::flatten` collapses categories into the flat `Vec<MacroConfig>` used by the dispatcher.
- **Levenshtein-based fuzzy matching.** `PhraseMatcher` normalizes (lowercase, alnum + whitespace, trim, single-space) then scores `1 - dist/max_len`, accepting only above the configured threshold (`src/pipeline/matcher.rs`).

## Process-wide Setup (one-shot, single-threaded)

Performed in `coordinator::spawn_pipeline` before pipeline workers start:
- Auto-set `ORT_DYLIB_PATH` to the sherpa-onnx-shipped `libonnxruntime.so` if unset, so both `ort` and `sherpa-onnx` share one ORT environment (`src/pipeline/coordinator.rs:231`). `unsafe { std::env::set_var(...) }` is documented as safe here because no pipeline threads exist yet.
- Load Silero VAD with `force_onnx_cpu: true`, wrapped in `catch_unwind`.

## Module / Package Boundaries

The crate is internally divided by responsibility, not by layer:

- **`audio`** — CPAL stream lifecycle, ringbuf, mono downmix, linear resampling.
- **`vad`** — Silero scoring (sliding 512-sample window over 320-sample frames) and segmentation (hysteresis, preroll, tail, max-utterance, force-flush).
- **`wake`** — sherpa-onnx keyword spotter wrapper.
- **`stt`** — Whisper service: bounded queue, dedicated thread, reusable state.
- **`pipeline`** — `coordinator` (orchestrates threads), `dispatcher` (phrase match + flags + macro emit + sound), `matcher` (Levenshtein fuzzy match), `jsonl` (stable wire format), `timing` (mono/wall clocks), `sound` (rodio playback).
- **`input`** — `ptt` (evdev key polling) and `inject` (uinput VirtualDevice + macro execution).
- **`pack`** — profile model: `Pack`/`Category`/serialization, `.hdpack` zip import/export, `manager` for active-profile persistence.
- **`control`** — UDS listener (Tokio), JSON request/response protocol, blocking client used by CLI subcommands.
- **`tui`** — ratatui/crossterm interactive editor (`hd-linux-voice edit`).
- **`ui`** — pure-logic state for the egui config GUI (`config_app.rs`, `first_run.rs`).
- **`config`** — `Config` struct hierarchy with `serde`-driven YAML loader and model-path validator.
- **`error`** — `thiserror`-derived `DaemonError` whose `Display` strings are user-facing remediation instructions.

The dispatcher holds an `Arc<RwLock<Vec<MacroConfig>>>`, allowing the control listener (Tokio task) to swap macros at runtime via `Dispatcher::update_macros` when `SwitchProfile` arrives (`src/pipeline/dispatcher.rs:79`, `src/control/mod.rs:89`).
