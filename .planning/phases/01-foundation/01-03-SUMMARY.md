---
phase: "01-foundation"
plan: "03"
subsystem: "audio-capture, ptt-input"
tags: ["cpal", "evdev", "atomicbool", "ringbuf", "ptt", "audio"]
dependency_graph:
  requires: ["01-02"]
  provides: ["AudioHandle", "start_audio_stream", "spawn_ptt_thread", "parse_key_code", "check_input_readable", "find_ptt_device", "process_event"]
  affects: ["src/audio/mod.rs", "src/input/ptt.rs"]
tech_stack:
  added: []
  patterns:
    - "AtomicBool PTT gate: ptt_active.load(Relaxed) in CPAL callback guards HeapRb push"
    - "HeapRb RING_BUFFER_SAMPLES constant: 16000*5 pre-allocated; no alloc in RT callback"
    - "build_audio_config() queries supported_input_configs() before constructing manually (Pitfall 1 mitigation)"
    - "evdev::enumerate() for device scan; supported_keys().contains(target_key) for PTT device detection"
    - "process_event() extracted as pure function for unit testing without real hardware"
    - "Manual /dev/input/event{0..63} scan instead of glob crate (no new dependency)"
key_files:
  created: []
  modified:
    - "src/audio/mod.rs"
    - "src/input/ptt.rs"
decisions:
  - "Manual event node loop (event{0..63}) over glob crate — avoids adding glob dependency"
  - "Consumer trait imported in test module directly (not re-exported by super::*)"
  - "device.description().name() used instead of deprecated device.name()"
  - "channels extracted as u16 copy before move closure to avoid borrow-then-move conflict on config"
metrics:
  duration: "~12 minutes"
  completed: "2026-04-22"
  tasks: 2
  files_modified: 2
---

# Phase 01 Plan 03: CPAL Audio Capture + evdev PTT Scanner Summary

CPAL warm-stream audio capture with AtomicBool PTT gate and HeapRb sample queue, plus evdev device enumeration, preflight check, pure `process_event` function, and blocking `spawn_ptt_thread` using std::thread.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 (RED) | Audio PTT gate tests | 04b1332 | src/audio/mod.rs |
| 1 (GREEN) | CPAL audio capture with PTT gate | b3273e7 | src/audio/mod.rs |
| 2 (RED) | evdev PTT scanner tests | 9474a9e | src/input/ptt.rs |
| 2 (GREEN) | evdev PTT scanner and thread | a3fa70a | src/input/ptt.rs |

## Test Results

```
cargo test --lib → 10 passed, 0 failed

audio::tests::ptt_gate_off_discards_samples     ok
audio::tests::ptt_gate_on_pushes_samples        ok
audio::tests::ring_buffer_overflow_does_not_panic ok
input::ptt::tests::parse_valid_key_code         ok
input::ptt::tests::parse_key_f13               ok
input::ptt::tests::parse_invalid_key_returns_err ok
input::ptt::tests::check_input_readable_actionable_error_contains_group ok
input::ptt::tests::process_event_press_sets_ptt_active ok
input::ptt::tests::process_event_release_clears_ptt_active ok
input::ptt::tests::process_event_different_key_does_not_change_ptt ok
```

## Verification Checklist

- [x] `cargo test --lib` exits 0 — 10 tests passing
- [x] No `device.grab()` or `EVIOCGRAB` call in ptt.rs (D-10 non-exclusive)
- [x] No `tokio::task::spawn_blocking` call in ptt.rs (doc comment only; actual code uses `std::thread::spawn`)
- [x] `ptt_active` uses `Ordering::Relaxed` for both load (audio callback) and store (PTT thread)
- [x] `check_input_readable()` error message contains "input" group and "usermod" command
- [x] `parse_key_code("INVALID")` returns Err with the bad key name in the message
- [x] No heap allocation (`Vec::new`, `Box::new`) inside the `build_input_stream` closure

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed config borrow-then-move conflict in start_audio_stream**
- **Found during:** Task 1 GREEN implementation
- **Issue:** Plan code passed `&config` to `build_input_stream` AND captured `config` by value in the move closure (references `config.channels` inside)
- **Fix:** Extracted `let channels = config.channels` (u16 Copy) before the closure; cloned config as `actual_config` before `build_input_stream` call
- **Files modified:** src/audio/mod.rs
- **Commit:** b3273e7

**2. [Rule 1 - API] CPAL 0.17 SampleRate is a u32 type alias, not a struct**
- **Found during:** Task 1 GREEN compile
- **Issue:** Plan used `SampleRate(16_000)` (tuple struct constructor) but CPAL 0.17 defines `type SampleRate = u32`
- **Fix:** Used `16_000` directly where sample rate value is needed; used `c.with_sample_rate(16_000)` for range configuration
- **Files modified:** src/audio/mod.rs
- **Commit:** b3273e7

**3. [Rule 1 - API] device.name() deprecated in CPAL 0.17; replaced with description().name()**
- **Found during:** Task 1 GREEN compile
- **Issue:** `Device::name()` is deprecated in CPAL 0.17; recommends `description()` instead
- **Fix:** Used `device.description().map(|d| d.name().to_string()).unwrap_or_else(|_| "unknown".to_string())`
- **Files modified:** src/audio/mod.rs
- **Commit:** b3273e7

**4. [Rule 1 - API] ringbuf 0.4.8 requires explicit trait imports for Consumer/Producer/Split**
- **Found during:** Task 1 GREEN compile
- **Issue:** Plan did not import `ringbuf::traits::{Producer, Split}` — methods unavailable without traits in scope
- **Fix:** Added `use ringbuf::traits::{Producer, Split};` to main module; added `use ringbuf::traits::Consumer;` directly in test module (Consumer not re-exported via `super::*`)
- **Files modified:** src/audio/mod.rs
- **Commit:** b3273e7

**5. [Rule 1 - API] producer.push() renamed to try_push() in ringbuf 0.4.8**
- **Found during:** Task 1 GREEN compile
- **Issue:** Plan used `producer.push(sample)` but the method is `try_push` in ringbuf 0.4
- **Fix:** Changed to `producer.try_push(sample)` with `let _ = ...` to discard the Result
- **Files modified:** src/audio/mod.rs
- **Commit:** b3273e7

**6. [Rule 2 - Missing functionality] Manual event node scan instead of glob crate**
- **Found during:** Task 2 GREEN planning
- **Issue:** Plan suggested `glob` crate OR manual loop; adding `glob` adds a dependency
- **Fix:** Used manual `(0..64).map(|i| PathBuf::from(format!("/dev/input/event{i}")))` — covers all practical event nodes without adding a dependency
- **Files modified:** src/input/ptt.rs
- **Commit:** a3fa70a

## Key Decisions

- **Manual event node scan**: Scanning `/dev/input/event0..63` manually avoids adding the `glob` dependency. Covers all practical event nodes on Linux systems.
- **channels extracted before closure**: `let channels = config.channels` (u16 is Copy) resolves the borrow-then-move conflict when config is referenced both in `build_input_stream(&config, ...)` and inside the move closure.
- **Consumer trait in test module**: `ringbuf::traits::Consumer` must be imported directly in the `#[cfg(test)] mod tests` block; it's not re-exported through `use super::*` from the parent module.

## Known Stubs

None — both subsystems are fully implemented with real logic.

## Threat Surface Scan

No new network endpoints, auth paths, or trust boundary crossings introduced beyond those documented in the plan's threat model (T-01-03-01 through T-01-03-05).

T-01-03-02 (CPAL callback alloc DoS): Mitigated — `push_slice` and `try_push` with pre-allocated HeapRb; no Vec/Box/String in callback closure.

T-01-03-03 (fetch_events on Tokio): Mitigated — `std::thread::spawn` confirmed (no `spawn_blocking`).

T-01-03-04 (/dev/input unreadable): Mitigated — `check_input_readable()` preflight with actionable error.

## Self-Check: PASSED

- src/audio/mod.rs: FOUND ✓
- src/input/ptt.rs: FOUND ✓
- commit 04b1332 (RED audio tests): FOUND ✓
- commit b3273e7 (GREEN audio impl): FOUND ✓
- commit 9474a9e (RED PTT tests): FOUND ✓
- commit a3fa70a (GREEN PTT impl): FOUND ✓
