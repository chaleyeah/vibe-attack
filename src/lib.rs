//! **vibe-attack** — a voice-macro daemon for Helldivers 2 on Linux.
//!
//! vibe-attack listens to a microphone, detects speech, and translates spoken
//! phrases into keypress sequences injected through `/dev/uinput`. A optional
//! wake-word gate can require you to say a trigger phrase before any macro fires.
//! All configuration lives in `$XDG_CONFIG_HOME/vibe-attack/config.yaml`; macro
//! packs live in subdirectories of `$XDG_CONFIG_HOME/vibe-attack/` as
//! `{name}/pack.yaml`.
//!
//! # Audio → Keypress Pipeline
//!
//! ```text
//!  Microphone (CPAL)
//!       │  f32 samples @ 16 kHz
//!       ▼
//!  ┌─────────┐   ring buffer    ┌─────────┐  UtteranceJob   ┌─────────┐
//!  │  audio  │ ──────────────▶ │   vad   │ ─────────────▶ │  wake   │
//!  │ (CPAL   │                 │ (Silero │                 │(sherpa- │
//!  │  RT cb) │                 │  VAD)   │                 │  onnx)  │
//!  └─────────┘                 └─────────┘                 └────┬────┘
//!                                                               │ keyword hit
//!                                                               ▼
//!  ┌─────────┐  keypress seqs  ┌──────────────────┐  text  ┌─────────┐
//!  │  input  │ ◀────────────── │    pipeline      │ ◀───── │   stt   │
//!  │(uinput) │                 │ (match → macro)  │        │(whisper)│
//!  └─────────┘                 └──────────────────┘        └─────────┘
//! ```
//!
//! Each stage runs on its own OS thread and communicates via bounded crossbeam
//! channels with drop-oldest semantics so a slow downstream never blocks audio
//! capture.
//!
//! # Module Guide
//!
//! | Module | Responsibility |
//! |--------|---------------|
//! | [`audio`] | CPAL input stream; writes f32 samples into a pre-allocated ring buffer without allocating in the RT callback. |
//! | [`vad`] | Silero VAD; segments the sample stream into speech utterances with hysteresis, preroll, and tail audio. |
//! | [`wake`] | sherpa-onnx keyword spotter; gates STT so only post-wake audio is transcribed. |
//! | [`stt`] | whisper.cpp transcription; runs on a dedicated blocking thread, never on the Tokio runtime. |
//! | [`pipeline`] | Coordinator, phrase matcher, dispatcher, and JSONL/timing helpers; routes transcribed text to macro actions. |
//! | [`input`] | PTT detection (`input::ptt`) and uinput keypress injection (`input::inject`). |
//! | [`control`] | Unix-socket control plane; lets CLI subcommands (ping, switch, test, import, export) talk to the running daemon. |
//! | [`config`] | Top-level `Config` struct (audio, VAD, wake, STT, pipeline, input settings) loaded from YAML. |
//! | [`error`] | `DaemonError` — typed fatal errors (uinput access, model load, socket bind) with actionable stderr messages. |
//! | [`pack`] | `Pack` / `MacroPack` — loads and manages `{name}/pack.yaml` macro-pack files; `pack::manager` handles hot-swap. |
//! | [`tui`] | Ratatui-based interactive TUI editor (`vibe-attack edit`) for browsing and editing macro packs. |
//! | [`ui`] | GUI helpers: first-run wizard, config app, audio-device probe, and optional system-tray (`feature = "gui"`). |
//!
//! # Where to Start
//!
//! | Goal | Where to look |
//! |------|--------------|
//! | Add a new spoken phrase / macro | [`pack`] — `pack.yaml` schema; [`pipeline::matcher`] — matching logic |
//! | Change how keypresses are injected | [`input::inject`] |
//! | Tune VAD sensitivity or timing | [`vad`] — thresholds and segment constants |
//! | Debug why a phrase doesn't dispatch | [`pipeline::coordinator`] → [`pipeline::dispatcher`] → tracing logs (`RUST_LOG=debug`) |
//! | Add a new control-plane command | [`control::protocol`] → [`control`] server handler |
//! | Understand the config file format | [`config`] — `Config` and its nested structs |

pub mod audio;
pub mod config;
pub mod error;
pub mod input;
pub mod pipeline;
pub mod stt;
pub mod vad;
pub mod wake;
pub mod control;
pub mod pack;
pub mod tui;
pub mod ui;
