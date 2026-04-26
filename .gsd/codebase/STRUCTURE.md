# STRUCTURE

Directory and code organization for `hd-linux-voice`. Single Cargo crate, Rust 2021.

## Top-Level Layout

```
hd-linux-voice/
├── Cargo.toml                # crate manifest, features, vendored whisper-rs patch
├── Cargo.lock                # committed; built with --locked in PKGBUILD
├── README.md                 # user-facing install + quickstart
├── CONTRIBUTING.md
├── LICENSE                   # AGPL-3.0-only
├── LICENSES.md               # aggregated dependency licenses (cargo-about output)
├── about.toml / about.hbs    # cargo-about config + handlebars template
├── config.example.yaml       # canonical reference config (copy to XDG dir)
├── config.yaml               # local dev config
├── demo_hd2.yaml             # demo / sample profile config
├── src/                      # all Rust source
├── tests/                    # integration tests (each .rs file = one test binary)
├── examples/                 # cargo example binaries (audio_probe.rs)
├── docs/                     # user-facing docs (configuration, troubleshooting, uinput, latency)
├── models/                   # local model directories: whisper/, sherpa/{kws,vad,…}
├── profiles/                 # bundled macro packs (e.g. profiles/hd2/pack.yaml)
├── packaging/                # PKGBUILD (Arch) and AppImage build script + .desktop file
├── vendor/whisper-rs/        # vendored fork of whisper-rs, patched in via [patch.crates-io]
├── target/                   # cargo build output (gitignored)
├── .gsd/                     # GSD workflow state (codebase/, milestones/, journal/, gsd.db, …)
├── .planning/                # planning notes (out-of-source)
├── .bg-shell/                # session shell state
└── .claude/ / .mcp.json      # agent / MCP integration metadata
```

## Source Code Organization (`src/`)

```
src/
├── main.rs                  # daemon entry point: CLI parsing, preflight, thread spawn, signal wait, shutdown
├── lib.rs                   # crate root re-exporting all modules (used by main, tests, and the GUI binary)
├── config.rs                # Config + sub-structs (Ptt/Timing/Audio/Pipeline/Vad/Stt/Wake/Macro), YAML loader, validator
├── error.rs                 # DaemonError enum (thiserror) — Display messages are the user-facing remedies
├── audio/
│   └── mod.rs               # CPAL stream, HeapRb, mono downmix, linear resampling, StreamGuard RAII
├── vad/
│   └── mod.rs               # VadConfig, VadSegmenter, Silero scoring window, drop-oldest helper
├── wake/
│   └── mod.rs               # sherpa-onnx KeywordSpotter wrapper (reads encoder/decoder/joiner/tokens/keywords)
├── stt/
│   └── mod.rs               # SttService, SttSubmitter, SttResult, dedicated whisper-rs thread + bounded queue
├── pipeline/
│   ├── mod.rs               # re-exports submodules
│   ├── coordinator.rs       # spawn_pipeline: pipeline + dispatcher + output threads, ORT_DYLIB_PATH bootstrap
│   ├── dispatcher.rs        # Dispatcher: phrase match, conditional flags, sound trigger, macro emit
│   ├── matcher.rs           # PhraseMatcher (normalize + Levenshtein-based fuzzy match)
│   ├── jsonl.rs             # JsonlWriter, JsonlEvent enum (utterance/stage/status/dispatch/no_match)
│   ├── timing.rs            # MonoClock, UtteranceTimings, wall_time_ms helper
│   └── sound.rs             # rodio SoundPlayer (per-macro WAV playback)
├── input/
│   ├── mod.rs               # re-exports
│   ├── ptt.rs               # evdev PTT detection: parse_key_code, find_ptt_device, spawn_ptt_thread
│   └── inject.rs            # uinput VirtualDevice + injection thread (MacroCmd::Execute / Shutdown)
├── control/
│   ├── mod.rs               # spawn_control_listener (Tokio UDS), socket placement, switch-profile handler
│   ├── client.rs            # blocking UDS client for CLI subcommands
│   └── protocol.rs          # ControlRequest / ControlResponse JSON enums (#[serde(tag, content)])
├── pack/
│   ├── mod.rs               # Pack, Category, .hdpack import/export (zip), profile dir helpers
│   └── manager.rs           # ProfileManager: persists active_profile to manager.yaml, loads active pack
├── tui/
│   ├── mod.rs               # ratatui + crossterm setup/teardown, key event loop
│   ├── app.rs               # TUI App state machine
│   └── editor.rs            # macro editor view
├── ui/
│   ├── mod.rs
│   ├── config_app.rs        # pure-logic ConfigApp state for the GUI (profile list, log lines, mic level)
│   └── first_run.rs         # FirstRunState: tracks setup completion checks
└── bin/
    └── hd-linux-voice-config.rs  # eframe (egui) GUI binary, gated behind the `gui` feature
```

### Library / binary split

- `src/lib.rs` exports every top-level module as part of the `hd_linux_voice` library crate. This is what tests, the GUI binary, and `main.rs` import.
- `src/main.rs` is the daemon `[[bin]]`.
- `src/bin/hd-linux-voice-config.rs` is the second `[[bin]]` (`required-features = ["gui"]`).

### Module conventions

- Each module under `src/` is either a single `.rs` file or a directory with `mod.rs` + sub-files. No `lib.rs`/`mod.rs` re-exports beyond the immediate submodule list.
- Submodules are made public via `pub mod`; types within are individually `pub` as needed.
- Threads are spawned by free functions named `spawn_*` (e.g. `spawn_ptt_thread`, `spawn_injection_thread`, `spawn_pipeline`, `spawn_control_listener`).

## Test Organization (`tests/`)

Each file under `tests/` is a standalone integration test binary (Rust convention). Privileged tests use `#[ignore]` plus an env-var gate. Files present:

- `concurrency_stress.rs` — pipeline/queue stress.
- `config_parse.rs` — YAML deserialization, `deny_unknown_fields`.
- `daemon_headless.rs` — daemon startup paths without a display.
- `dispatcher_logic.rs` — phrase matching, flag conditions.
- `documentation.rs` — assertions over docs (e.g. README/configuration examples).
- `drop_oldest_queue.rs` — `try_send_drop_oldest` semantics.
- `jsonl_schema.rs` — JSONL event shape stability.
- `macro_inject.rs` — privileged uinput integration test.
- `pack_hd2_bundle.rs` — `.hdpack` import/export round-trip.
- `stt_smoke.rs` — STT path (feature-gated).
- `ui_distribution.rs` — UI logic / distribution layout.
- `uinput_smoke.rs` — uinput device open smoke test.
- `wake_word.rs` — sherpa-onnx KWS integration.

Unit tests live alongside source via `#[cfg(test)] mod tests`. Examples: `src/audio/mod.rs:218`, `src/vad/mod.rs:321`, `src/input/inject.rs:222`, `src/input/ptt.rs:142`, `src/pipeline/matcher.rs:61`, `src/pack/mod.rs:161`.

## Configuration File Locations

| Purpose | Path |
|---|---|
| Daemon config (runtime) | `$XDG_CONFIG_HOME/hd-linux-voice/config.yaml` (default `~/.config/hd-linux-voice/config.yaml`) |
| Override flag | `--config <FILE>` |
| Active-profile pointer | `$XDG_CONFIG_HOME/hd-linux-voice/manager.yaml` |
| Profiles directory | `$XDG_CONFIG_HOME/hd-linux-voice/profiles/<name>/` (each: `pack.yaml` + optional `sounds/`) |
| Control socket | `$XDG_RUNTIME_DIR/hd-linux-voice/hd-linux-voice.sock` (mode 0600) |
| Reference config | `config.example.yaml` (repo root) |
| Local dev configs | `config.yaml`, `demo_hd2.yaml` (repo root) |
| Bundled profile | `profiles/hd2/pack.yaml` |
| Recommended model dir | `~/.local/share/hd-linux-voice/models/{whisper,sherpa}/` (per `config.example.yaml`) |
| Cargo build config | `Cargo.toml`, `Cargo.lock` |
| License aggregator config | `about.toml`, `about.hbs` |
| Packaging | `packaging/PKGBUILD`, `packaging/appimage/build.sh`, `packaging/appimage/hd-linux-voice.desktop` |

## Documentation (`docs/`)

- `configuration.md` — full config reference for users.
- `troubleshooting.md` — uinput permissions, audio device problems, Whisper model setup.
- `uinput-setup.md` — group/udev steps for `/dev/uinput`.
- `latency-baseline.md` + `latency-proofs/` — measured latency floors with supporting JSONL.

## GSD Workflow Artifacts (`.gsd/`)

Project-management state managed by the GSD workflow tooling — not part of the runtime:

- `gsd.db` — SQLite (Phase 1) memory/journal store.
- `journal/`, `activity/`, `event-log.jsonl` — auto-mode and tool-call audit trails.
- `milestones/` — per-milestone planning, summaries, validation.
- `codebase/` — this scan's output (STACK, INTEGRATIONS, ARCHITECTURE, STRUCTURE, …).
- `STATE.md`, `last-snapshot.md`, `state-manifest.json` — current/working state pointers.
- `runtime/` — write-gate and other runtime control state.
- `notifications.jsonl` — workflow notifications.
