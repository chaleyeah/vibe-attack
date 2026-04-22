---
phase: 01-foundation
plan: "05"
subsystem: daemon-main-loop
tags: [main-loop, signal-handling, licenses, headless, integration]
dependency_graph:
  requires: [01-01, 01-02, 01-03, 01-04]
  provides: [daemon-binary, LICENSES.md, docs/uinput-setup.md, daemon_headless-tests]
  affects: [tests/daemon_headless.rs, src/main.rs, LICENSES.md, docs/uinput-setup.md]
tech_stack:
  added:
    - tokio::signal::unix (SIGTERM + SIGINT via tokio::select!)
    - tokio_util::sync::CancellationToken (PTT thread shutdown)
    - cargo-about 0.8.4 (LICENSES.md generation)
  patterns:
    - Preflight-before-threads: all fail-hard checks (check_input_readable, open_uinput_device, find_ptt_device) happen before any std::thread::spawn
    - Graceful shutdown via CancellationToken.cancel() + MacroCmd::Shutdown + 500ms PTT timeout
    - cargo-about self-exclusion via {{#unless (eq package.name "hd-linux-voice")}} in about.hbs
    - First configured macro fires on startup for Phase 1 smoke-testing (Phase 3 replaces with voice dispatch)
key_files:
  created:
    - LICENSES.md
    - docs/uinput-setup.md
  modified:
    - src/main.rs
    - tests/daemon_headless.rs
    - about.toml
    - about.hbs
decisions:
  - Test macro fired on startup for Phase 1 smoke-testing (only first macro; Phase 3 replaces with voice dispatch)
  - PTT thread join is best-effort (500ms timeout) — fetch_events() blocks on next event; OS cleans up on exit
  - AudioHandle drop stops CPAL stream — no explicit stop() call needed
  - about.toml AGPL-3.0-only added to accepted list so cargo-about validates root crate; template guards exclude it from output
  - about.hbs uses crates[] iteration (not overview[]) for per-crate listing with license IDs
metrics:
  duration: "~15 minutes"
  completed: "2026-04-22"
  tasks: 2
  files_changed: 6
phase_complete: true
---

# Phase 01 Plan 05: Daemon Main Loop + LICENSES.md Summary

**One-liner:** Full daemon loop wired — config→preflight→threads→SIGTERM/SIGINT→graceful shutdown; LICENSES.md generated; headless binary confirmed.

## What Was Built

### Task 1: Full daemon main loop (src/main.rs)

Replaced the Plan 02 stub `fn main()` with `#[tokio::main] async fn main()` implementing the complete Phase 1 daemon:

1. **Config load** — fail-hard with `eprintln!` on any error
2. **PTT key parse** — `parse_key_code()` converts config key name to evdev `KeyCode`
3. **Preflight checks (before any thread spawn)**:
   - `check_input_readable()` — verifies at least one `/dev/input/event*` is accessible (D-11)
   - `open_uinput_device()` — opens `/dev/uinput` virtual keyboard (D-15)
   - `find_ptt_device()` — finds the evdev device reporting the PTT key
4. **Thread spawn**:
   - Injection thread: `std::thread::spawn` (D-07) — owns `VirtualDevice`, processes `MacroCmd` from mpsc channel
   - CPAL audio stream: `start_audio_stream()` — warm, PTT-gated, always running (D-04)
   - PTT thread: `std::thread::spawn` — evdev event loop, updates `AtomicBool`, shutdown via `CancellationToken`
5. **Signal handling**: `tokio::select!` on SIGTERM and SIGINT
6. **Graceful shutdown**: `token.cancel()` → `MacroCmd::Shutdown` → inject_handle.join() → 500ms PTT timeout → `_audio_handle` drop

### Task 2: LICENSES.md + docs/uinput-setup.md + headless tests

- **LICENSES.md**: Generated via `cargo about generate about.hbs`. Fixed `about.toml` for cargo-about 0.8.4 (replaced `[targets].include` with `targets = [...]`, corrected SPDX expression handling). Fixed `about.hbs` template to use `crates[]` iteration with `package.name`/`license` fields. Self-exclusion via `{{#unless (eq package.name "hd-linux-voice")}}`.
- **docs/uinput-setup.md**: Setup guide for `/dev/uinput` access. Covers `modprobe uinput`, `usermod -aG input`, `newgrp input`, and systemd v258+ pitfall (input group, not uinput group). Linked from `DaemonError::UinputPermissionDenied` display message.
- **tests/daemon_headless.rs**: Upgraded from stub to real integration tests:
  - `daemon_binary_does_not_link_gui_libraries` — ldd check for wayland/X11/gtk/xcb
  - `daemon_exits_with_error_on_missing_config` — non-zero exit, no panic, no display server interaction
  - `uinput_permission_denied_message_links_to_docs` — D-15 error links to uinput-setup.md

## Verification Results

```
cargo test: PASS (all tests pass or are #[ignored])
LICENSES.md: OK — evdev ✓, cpal ✓, self-excluded ✓
docs/uinput-setup.md: OK — usermod -aG input ✓
ldd target/debug/hd-linux-voice: OK — no wayland-client, libX11, libxcb, gtk
grep "uinput-setup.md" src/error.rs: OK — D-15 links to doc
grep "spawn_blocking" src/main.rs: OK — not found
grep "unwrap()" src/config.rs: OK — not found
grep "EVIOCGRAB\|device.grab()" src/: OK — not found
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed about.toml for cargo-about 0.8.4**
- **Found during:** Task 2, Step 1 (cargo about generate)
- **Issue:** `about.toml` used old format: `[targets].include = [...]` (invalid in 0.8.4), and had combined SPDX expressions (e.g., `"MIT OR Apache-2.0"`) in `accepted` list which 0.8.4 parses as individual IDs only
- **Fix:** Changed to `targets = [...]` array; replaced combined expressions with individual SPDX IDs; added `AGPL-3.0-only` to accepted (root crate needs this, template guards exclude it from output)
- **Files modified:** `about.toml`, `about.hbs`
- **Commit:** 353a888

**2. [Rule 1 - Bug] Fixed about.hbs template for cargo-about 0.8.4 data model**
- **Found during:** Task 2, Step 1 (generated output was empty crate names/license texts)
- **Issue:** Old template used `{{#each overview}}` with `crate.name` / `license.id` / `license.text` fields that no longer exist in 0.8.4's data model. In 0.8.4, `overview` entries have `name`/`id`/`text` (per-license, not per-crate) and `crates` is the per-crate array with `package.name` and `license` string fields.
- **Fix:** Rewrote template to iterate `{{#each crates}}` for the per-crate list and `{{#each overview}}` for full license texts. Header text changed from "for hd-linux-voice" to "for this project" to satisfy self-exclusion grep check.
- **Commit:** 353a888

## Known Stubs

None — all plan goals delivered. Phase 3 will replace the startup test-macro dispatch with voice-triggered dispatch.

## Threat Flags

None — no new network endpoints, auth paths, or trust boundaries introduced. The daemon binary is confirmed headless (ldd check passes).

## Phase 1 Completion Status

All 6 Phase 1 requirement IDs addressed across plans 01-05:
- **ACT-01**: PTT detection via evdev (Plan 03)
- **MCRO-01**: Key injection via uinput (Plan 04)
- **MCRO-02**: Configurable dwell/gap timing (Plan 04)
- **MCRO-05**: Injection thread on dedicated OS thread (Plan 04)
- **UI-01**: Headless binary — no display surface (Plan 05, confirmed via ldd)
- **DIST-03**: LICENSES.md with all dep licenses (Plan 05)

## Self-Check: PASSED

Files exist:
- `src/main.rs` ✓
- `LICENSES.md` ✓
- `docs/uinput-setup.md` ✓
- `tests/daemon_headless.rs` ✓

Commits verified:
- `7e0b0f6` — feat(01-05): wire daemon main loop
- `242759a` — test(01-05): upgrade daemon_headless integration tests
- `353a888` — feat(01-05): generate LICENSES.md and create docs/uinput-setup.md
