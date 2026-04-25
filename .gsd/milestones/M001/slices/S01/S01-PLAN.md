# S01: Foundation

**Goal:** Install the Rust toolchain and create a compilable project skeleton: Cargo.
**Demo:** Install the Rust toolchain and create a compilable project skeleton: Cargo.

## Must-Haves


## Tasks

- [x] **T01: 01-foundation 01**
  - Install the Rust toolchain and create a compilable project skeleton: Cargo.toml with all 13
Phase 1 dependencies, a module stub hierarchy (config, audio, input, error), four integration
test stubs, and cargo-about configuration files.

Purpose: Every subsequent plan depends on `cargo check` passing. This plan clears the
compilation baseline so Wave 2+ plans can focus purely on implementation.

Output: A compilable Rust crate with all deps resolved, stub modules in place, and
test infrastructure ready for Wave 1–4 implementations to fill in.
- [x] **T02: 01-foundation 02**
  - Implement the configuration system (typed serde structs, XDG path resolution, YAML
deserialization) and the daemon entry point (clap CLI, tracing init). After this plan,
`hd-linux-voice --help` works and config loading is fully tested.

Purpose: Config is the source of truth for every runtime decision (PTT key, timing
defaults, macro definitions). Getting it right with full validation prevents silent
misconfiguration bugs in all downstream plans.

Output: `src/config.rs` (full implementation), `src/main.rs` (CLI + logging + config load
call), `config.example.yaml`, and passing unit tests in `tests/config_parse.rs`.
- [x] **T03: 01-foundation 03**
  - Implement the two real-time subsystems that form the PTT pipeline: CPAL audio capture
(with warm stream + AtomicBool gate) and evdev PTT scanner (device enumeration + blocking
event loop). Both subsystems share a single `Arc<AtomicBool>` (ptt_active) — the PTT
thread writes it; the CPAL callback reads it.

Purpose: These two systems form the input side of the entire daemon. Subsequent plans
wire them together with the injection thread and daemon loop.

Output: `src/audio/mod.rs` (CPAL stream + HeapRb gate), `src/input/ptt.rs` (evdev
enumerate + PTT thread). Both compile and have unit tests.
- [x] **T04: 01-foundation 04**
  - Implement the uinput injection subsystem: a keyboard-only `VirtualDevice` (MCRO-05),
a `MacroCmd` channel, and a dedicated OS injection thread that executes key sequences
with dwell + gap timing (MCRO-01, MCRO-02). D-15 actionable error on permission denied.

Purpose: This is the output side of the daemon — where voice commands will ultimately
become keystrokes in the game. Correctness of the key-hold timing and the error message
on /dev/uinput failure are critical UX invariants.

Output: `src/input/inject.rs` (full implementation), updated `src/error.rs` (DaemonError
with actionable uinput message). Integration test stubs in `tests/macro_inject.rs` and
`tests/uinput_smoke.rs` are upgraded to real tests (privileged, #[ignore] guarded).
- [x] **T05: 01-foundation 05**
  - Wire all Phase 1 subsystems into the daemon main loop: config load, uinput preflight,
PTT thread, CPAL audio stream, injection thread, and Tokio signal handling for graceful
shutdown. Generate LICENSES.md via cargo-about. Create docs/uinput-setup.md.

Purpose: This plan delivers the COMPLETE Phase 1 deliverable — a working headless daemon
that captures PTT-gated audio and can inject key sequences, with all error handling and
license compliance in place.

Output: Full `src/main.rs` daemon loop, `LICENSES.md`, `docs/uinput-setup.md`, and
passing `tests/daemon_headless.rs` integration test.

## Files Likely Touched

- `Cargo.toml`
- `about.toml`
- `about.hbs`
- `src/main.rs`
- `src/config.rs`
- `src/error.rs`
- `src/audio/mod.rs`
- `src/input/mod.rs`
- `src/input/ptt.rs`
- `src/input/inject.rs`
- `tests/config_parse.rs`
- `tests/macro_inject.rs`
- `tests/uinput_smoke.rs`
- `tests/daemon_headless.rs`
- `src/config.rs`
- `src/lib.rs`
- `src/main.rs`
- `config.example.yaml`
- `Cargo.toml`
- `src/audio/mod.rs`
- `src/input/ptt.rs`
- `src/input/inject.rs`
- `src/error.rs`
- `tests/macro_inject.rs`
- `tests/uinput_smoke.rs`
- `src/main.rs`
- `docs/uinput-setup.md`
- `LICENSES.md`
- `tests/daemon_headless.rs`
