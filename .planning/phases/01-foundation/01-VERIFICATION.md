---
phase: 01-foundation
verified: 2026-04-21T00:00:00Z
status: human_needed
score: 5/5 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Hold the configured PTT key while the daemon runs with --verbose -vv and a fullscreen Proton/Steam game is in the foreground. Release the key."
    expected: "TRACE log lines 'PTT state changed' appear in the daemon terminal (not the game window) — first with pressed=true, then pressed=false. The game remains focused throughout."
    why_human: "Requires live hardware: a physical PTT key, a running CPAL-compatible audio device, and a fullscreen game session. Cannot mock evdev→AtomicBool→CPAL data-flow end-to-end in CI."
  - test: "Run the daemon with a config containing a test macro (e.g., KEY_UP/KEY_DOWN/KEY_LEFT/KEY_RIGHT sequence). Launch a text editor or game that responds to arrow keys and ensure it has focus. Observe the daemon startup behavior."
    expected: "The arrow key sequence appears in the focused window immediately after daemon start (Phase 1 fires first macro on startup). Key presses are discrete with visible dwell gaps, no double-SYN artifacts, no corrupted input."
    why_human: "Requires /dev/uinput access (user in 'input' group) and a running Wayland session with a focused target window. Integration tests (macro_inject.rs) verify timing but are gated behind RUN_PRIVILEGED_TESTS=1."
  - test: "Run the daemon with a fullscreen Steam/Proton game active. Start the daemon from a separate terminal."
    expected: "The fullscreen game does not minimize, lose focus, or flicker. The daemon starts without any display surface interaction."
    why_human: "The ldd check and absence of GUI crates confirm no GUI linkage, but actual game focus behavior requires a live fullscreen session."
---

# Phase 1: Foundation Verification Report

**Phase Goal:** A CLI daemon can capture microphone audio and inject arbitrary key sequences into a Wayland session via uinput, with PTT working correctly even when a fullscreen game holds exclusive compositor focus
**Verified:** 2026-04-21
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (ROADMAP Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Holding PTT key causes audio capture to begin; releasing stops it — observable via console log even when fullscreen game foregrounded | ✓ VERIFIED | `ptt_active.store(pressed, Relaxed)` in `process_event()` (ptt.rs:96); `ptt_active.load(Relaxed)` gates HeapRb push in CPAL callback (audio/mod.rs:110); `tracing::trace!` at ptt.rs:98; evdev reads from `/dev/input/event*` at kernel level — bypasses compositor focus |
| 2 | A test macro fires configurable key sequence (with dwell + gap) into a focused Wayland window via uinput, including key-hold | ✓ VERIFIED | `emit_key_action()` (inject.rs:140-167): KEY_DOWN(value=1) → `sleep(dwell_ms)` → KEY_UP(value=0) → `sleep(gap_ms)`; per-key overrides: `step.dwell_ms.unwrap_or(default_dwell_ms)` (inject.rs:196-197); VirtualDevice via `/dev/uinput` named "hd-linux-voice"; macro fires on daemon startup (main.rs:111-127) |
| 3 | Daemon starts headless with no mapped window; does not minimize a running fullscreen game | ✓ VERIFIED | No GUI crates in Cargo.toml (winit/xcb/wayland-client/gtk absent); `ldd target/debug/hd-linux-voice` shows no libwayland-client/libX11/libxcb/libgtk; `daemon_binary_does_not_link_gui_libraries` passes (3/3 headless tests green) |
| 4 | Startup emits actionable error and exits immediately if `/dev/uinput` cannot be opened — exact fix command + docs link | ✓ VERIFIED | `DaemonError::UinputPermissionDenied` Display (error.rs:9-13): "cannot open /dev/uinput", "modprobe uinput", "usermod -aG input $USER", "newgrp input", "docs/uinput-setup.md" link; `uinput_permission_denied_error_contains_required_strings` test passes; preflight runs before any thread spawn (main.rs:75) |
| 5 | All bundled Rust deps carry AGPL-3.0-compatible licenses; LICENSES.md exists | ✓ VERIFIED | LICENSES.md exists (617 lines); contains evdev (Apache-2.0 OR MIT), cpal (Apache-2.0); self-excluded (`hd-linux-voice` not present in LICENSES.md); about.toml accepted list: MIT, Apache-2.0, LGPL-2.1-only, LGPL-3.0-only, ISC, Unicode-3.0 |

**Score:** 5/5 truths verified by code analysis

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/main.rs` | Full daemon loop: config load, preflight, thread spawn, signal handling, shutdown | ✓ VERIFIED | `#[tokio::main]`; preflight (check_input_readable, open_uinput_device, find_ptt_device) before any `std::thread::spawn`; `CancellationToken` + `tokio::select!` for SIGTERM/SIGINT; `MacroCmd::Shutdown` on exit |
| `src/config.rs` | Config, PttConfig, TimingConfig, MacroConfig, KeyAction structs + load() | ✓ VERIFIED | All 5 structs with `#[serde(deny_unknown_fields)]` (5 matches); `serde_yaml_ng::from_reader()`; `xdg::BaseDirectories`; no `unwrap()` in load path |
| `src/audio/mod.rs` | CPAL stream init + PTT gate; AudioHandle with HeapRb | ✓ VERIFIED | `HeapRb::<f32>::new(RING_BUFFER_SAMPLES)` (16000×5); `AtomicBool` PTT gate; `Ordering::Relaxed`; no allocation in callback closure; 3 unit tests green |
| `src/input/ptt.rs` | evdev enumeration, preflight, PTT thread | ✓ VERIFIED | `check_input_readable()` with "input"+"usermod" in error; `find_ptt_device()`; `spawn_ptt_thread()` using `std::thread::spawn`; no EVIOCGRAB; 7 unit tests green |
| `src/input/inject.rs` | open_uinput_device(), spawn_injection_thread(), MacroCmd enum | ✓ VERIFIED | `VirtualDevice::builder()` with `VIRTUAL_KEYBOARD_KEYS`; `MacroCmd::Execute`/`Shutdown`; `std::thread::spawn`; no SYN_REPORT in emit calls; no `spawn_blocking` |
| `src/error.rs` | DaemonError::UinputPermissionDenied with actionable message | ✓ VERIFIED | All 4 required strings present (cannot open, modprobe, usermod, newgrp); "uinput-setup.md" link; 'input' group (not 'uinput' — systemd v258+ correct) |
| `config.example.yaml` | Canonical config with dwell_ms | ✓ VERIFIED | Exists; contains `dwell_ms`, `gap_ms`, `ptt.key`, per-key override example |
| `LICENSES.md` | Third-party dep license inventory (DIST-03) | ✓ VERIFIED | 617 lines; evdev + cpal entries present; self-excluded |
| `docs/uinput-setup.md` | uinput setup guide linked from D-15 error | ✓ VERIFIED | Exists; contains "usermod -aG input" (2×), "modprobe uinput" (2×), systemd v258+ note |
| `about.toml` + `about.hbs` | cargo-about config with self-exclusion | ✓ VERIFIED | `{{#unless (eq package.name "hd-linux-voice")}}` self-exclusion in about.hbs; accepted licenses list in about.toml |
| `tests/config_parse.rs` | 6 config round-trip tests | ✓ VERIFIED | 6 passed, 0 failed |
| `tests/macro_inject.rs` | Privileged injection integration tests | ✓ VERIFIED | 3 tests exist, all `#[ignore]` (gated by `RUN_PRIVILEGED_TESTS=1`), compile and show as `ignored` not failed |
| `tests/uinput_smoke.rs` | VirtualDevice smoke test | ✓ VERIFIED | 1 privileged test (`#[ignore]`) + 1 non-privileged test passes |
| `tests/daemon_headless.rs` | Binary headless check via ldd + config error path | ✓ VERIFIED | 3 passed, 0 failed: `daemon_binary_does_not_link_gui_libraries`, `daemon_exits_with_error_on_missing_config`, `uinput_permission_denied_message_links_to_docs` |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/main.rs` | `input::inject::open_uinput_device()` | Called before any thread spawn | ✓ WIRED | main.rs:75 calls `open_uinput_device()` before `spawn_injection_thread()` (main.rs:90) |
| `src/main.rs` | `CancellationToken` | `token.cancel()` on SIGTERM/SIGINT; passed to PTT thread | ✓ WIRED | main.rs:89 creates token; main.rs:148 cancels on signal; main.rs:107 passes clone to spawn_ptt_thread |
| `src/main.rs` | `audio::start_audio_stream()` | AudioHandle kept alive in main scope | ✓ WIRED | main.rs:96; `_audio_handle` kept in scope; drop on function exit stops CPAL stream |
| `src/audio/mod.rs` | `ptt_active: Arc<AtomicBool>` | `load(Ordering::Relaxed)` inside CPAL callback | ✓ WIRED | audio/mod.rs:110: `if ptt_active.load(Ordering::Relaxed)` |
| `src/input/ptt.rs` | `ptt_active: Arc<AtomicBool>` | `store(pressed, Ordering::Relaxed)` | ✓ WIRED | ptt.rs:96: `ptt_active.store(pressed, Ordering::Relaxed)` |
| `spawn_injection_thread()` | `std::sync::mpsc::Receiver<MacroCmd>` | blocking `recv()` loop on OS thread | ✓ WIRED | inject.rs:182: `std::thread::spawn`; inject.rs:185: `rx.recv()` |
| `emit_key_action()` | `evdev::uinput::VirtualDevice` | `device.emit(&[KEY_DOWN_event])` | ✓ WIRED | inject.rs:154: `device.emit(&[InputEvent::new(EventType::KEY.0, key.0, 1)])`; inject.rs:161: KEY_UP emit |
| `src/config.rs` | `xdg::BaseDirectories` | XDG path resolution | ✓ WIRED | config.rs:68: `xdg::BaseDirectories::with_prefix("hd-linux-voice")` |
| `src/config.rs` | `serde_yaml_ng` | `serde_yaml_ng::from_reader()` | ✓ WIRED | config.rs:89: `serde_yaml_ng::from_reader(file)` |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|-------------------|--------|
| `audio/mod.rs` CPAL callback | `ptt_active` (AtomicBool) | `Arc<AtomicBool>` shared with PTT thread; PTT thread writes on evdev key events | Yes — written by `spawn_ptt_thread` on real keypress events | ✓ FLOWING |
| `audio/mod.rs` HeapRb `consumer` | f32 samples from CPAL | CPAL input stream from `default_input_device()` | Yes — real audio device samples | ✓ FLOWING |
| `input/inject.rs` injection thread | `MacroCmd::Execute { keys }` | `mpsc::Receiver<MacroCmd>` from `macro_tx.send()` in main.rs | Yes — config macros mapped to KeySteps and sent on startup | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| `--help` prints --verbose and --config flags | `cargo run -- --help` | Output shows `--verbose...` and `--config <FILE>` | ✓ PASS |
| Missing config exits non-zero, no panic | `cargo run -- --config /tmp/nonexistent.yaml` | Exit 1; stderr: "Config file not found: ..."; no backtrace | ✓ PASS |
| `cargo test --lib` passes 13 tests | `cargo test --lib` | 13 passed, 1 ignored, 0 failed | ✓ PASS |
| Config round-trip tests | `cargo test --test config_parse` | 6 passed, 0 failed | ✓ PASS |
| Headless binary tests | `cargo test --test daemon_headless` | 3 passed, 0 failed | ✓ PASS |
| No GUI libs linked | `ldd target/debug/hd-linux-voice \| grep -iE "wayland\|X11\|gtk\|xcb"` | No matches | ✓ PASS |
| No `spawn_blocking` in source code (comments only) | `grep -rn "spawn_blocking" src/` | Only appears in doc/regular comments; no actual call sites | ✓ PASS |
| No `EVIOCGRAB` or `device.grab()` | `grep -rn "EVIOCGRAB\|device\.grab()" src/` | Only in doc comment in ptt.rs:5 | ✓ PASS |
| No `unwrap()` in config load path | `grep -n "unwrap()" src/config.rs` | No matches | ✓ PASS |
| D-15 error contains all required strings | `cargo test uinput_permission_denied_error_contains_required_strings` | Pass | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| ACT-01 | 01-02, 01-03 | Configurable PTT via evdev (works on Wayland) | ✓ SATISFIED | `parse_key_code()`, `find_ptt_device()`, `spawn_ptt_thread()` fully implemented; non-exclusive read (no EVIOCGRAB); 7 PTT unit tests green |
| MCRO-01 | 01-04 | Key sequences with configurable inter-key delays | ✓ SATISFIED | `emit_key_action()` with `dwell_ms`+`gap_ms` timing; `MacroCmd::Execute` channel; per-key override via `KeyStep`; integration tests written |
| MCRO-02 | 01-04 | Key/button holds (press-and-hold for duration) | ✓ SATISFIED | `emit_key_action()` emits KEY_DOWN → `sleep(dwell_ms)` → KEY_UP — dwell_ms is the hold duration; per-key dwell override tested |
| MCRO-05 | 01-04 | Key events via uinput/evdev for Wayland sessions | ✓ SATISFIED | `open_uinput_device()` creates keyboard-only VirtualDevice via `/dev/uinput`; uinput bypasses Wayland compositor focus |
| UI-01 | 01-01, 01-05 | Headless daemon (no window required) | ✓ SATISFIED | No GUI crates in Cargo.toml; ldd clean; `daemon_binary_does_not_link_gui_libraries` test passes |
| DIST-03 | 01-01, 01-05 | AGPL-3.0 licensed; all deps AGPL-compatible; LICENSES.md exists | ✓ SATISFIED | LICENSES.md with 617 lines; evdev (Apache-2.0 OR MIT) + cpal (Apache-2.0) listed; self-excluded; about.toml accepted list covers all bundled license IDs |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src/error.rs` | 13 | URL contains `yourusername` placeholder: `https://github.com/yourusername/hd-linux-voice/...` | ℹ️ Info | The docs/uinput-setup.md file exists and the link structure is correct; placeholder will need updating before public release. Does not affect runtime behavior. |

No blockers or warnings found. The `yourusername` placeholder is cosmetic only.

### Human Verification Required

#### 1. PTT Gating With Fullscreen Game Active

**Test:** Run daemon with `--verbose -vv` and a valid config. Hold the configured PTT key while a fullscreen Steam/Proton game is in the foreground. Observe the daemon's terminal (not the game).
**Expected:** TRACE log lines appear in daemon terminal: `PTT state changed key=KEY_F13 pressed=true` on press, `pressed=false` on release. Game remains focused throughout.
**Why human:** Requires a physical PTT key + `/dev/input/event*` access (input group) + running audio device. The code path is fully wired (evdev→AtomicBool→CPAL), but end-to-end flow cannot be asserted without live hardware.

#### 2. Key Sequence Injection Into Focused Wayland Window

**Test:** With user in `input` group, run daemon with a config containing a test macro (e.g., KEY_UP/KEY_DOWN sequence). Focus a text editor or terminal that responds to arrow keys. Observe arrow key presses.
**Expected:** Arrow keys register in the focused Wayland window immediately after daemon starts. The sequence is discrete (not held), consistent with 50ms dwell + 30ms gap defaults. No double-key artifacts or stuck keys.
**Why human:** Requires `/dev/uinput` access. The privileged integration tests (`RUN_PRIVILEGED_TESTS=1 cargo test --test macro_inject -- --include-ignored`) can verify timing but not actual Wayland session injection.

#### 3. No Focus Disruption With Live Fullscreen Game

**Test:** Start a fullscreen Steam/Proton game. From a second terminal, start the daemon with a valid config.
**Expected:** Fullscreen game does not minimize, flash, or lose focus. The daemon startup log ("Daemon running") appears in the second terminal while the game remains in the foreground.
**Why human:** The ldd headless check and GUI-crate absence prove no display surface is created, but actual Wayland compositor behavior with a running game requires a live session to confirm zero focus disruption.

### Gaps Summary

No gaps found. All five ROADMAP Success Criteria are satisfied by the implemented codebase:

- **SC-1 (PTT gates audio):** Full code path verified — evdev→AtomicBool→CPAL callback; TRACE logging in place; non-exclusive evdev read (no EVIOCGRAB) confirmed.
- **SC-2 (key injection with timing):** `emit_key_action()` implements KEY_DOWN→dwell→KEY_UP→gap; per-key override wired through `KeyStep`; VirtualDevice via uinput created and named "hd-linux-voice".
- **SC-3 (headless, no focus disruption):** No GUI crates, no GUI libs, ldd clean, headless integration tests pass.
- **SC-4 (actionable uinput error):** All four required error strings verified; docs/uinput-setup.md linked from error message.
- **SC-5 (license compliance):** LICENSES.md exists with full dep inventory; self-excluded; all accepted SPDX IDs cover bundled deps.

Three human verification items remain for E2E validation with live hardware. These are gating for the "PTT works with fullscreen game" claim which cannot be asserted from code analysis alone.

---

_Verified: 2026-04-21_
_Verifier: Claude (gsd-verifier)_
