# T05: 01-foundation 05

**Slice:** S01 — **Milestone:** M001

## Description

Wire all Phase 1 subsystems into the daemon main loop: config load, uinput preflight,
PTT thread, CPAL audio stream, injection thread, and Tokio signal handling for graceful
shutdown. Generate LICENSES.md via cargo-about. Create docs/uinput-setup.md.

Purpose: This plan delivers the COMPLETE Phase 1 deliverable — a working headless daemon
that captures PTT-gated audio and can inject key sequences, with all error handling and
license compliance in place.

Output: Full `src/main.rs` daemon loop, `LICENSES.md`, `docs/uinput-setup.md`, and
passing `tests/daemon_headless.rs` integration test.

## Must-Haves

- [ ] "Running hd-linux-voice with a valid config starts all three threads (PTT, CPAL, injection) and logs 'Daemon running'"
- [ ] "SIGTERM and SIGINT cause a clean shutdown: injection thread joins, CPAL stream stops, PTT thread exits"
- [ ] "If /dev/uinput is permission-denied, daemon exits with the D-15 error message before spawning any threads"
- [ ] "If PTT device cannot be found, daemon exits with an actionable error before spawning any threads"
- [ ] "Daemon starts with no WAYLAND_DISPLAY or DISPLAY socket interaction — no display surface created"
- [ ] "LICENSES.md exists and lists all Phase 1 deps (serde_yaml_ng, evdev, cpal, ringbuf, tokio, etc.)"
- [ ] "docs/uinput-setup.md exists and is referenced by the D-15 error message"
- [ ] "cargo test exits 0 (lib + integration stubs all green)"

## Files

- `src/main.rs`
- `docs/uinput-setup.md`
- `LICENSES.md`
- `tests/daemon_headless.rs`
