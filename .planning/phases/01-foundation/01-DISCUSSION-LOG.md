# Phase 1: Foundation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-21
**Phase:** 01-foundation
**Areas discussed:** Audio Capture Stack, Input Injection Backend, PTT Behavior, Daemon + Config

---

## Audio Capture Stack

| Option | Description | Selected |
|--------|-------------|----------|
| PipeWire-native | Direct PipeWire API, Linux-modern but ties to PW | |
| CPAL | Cross-driver Rust crate (ALSA/Pulse/PipeWire) | ✓ |
| ALSA-direct | Lowest-level, no abstraction | |

**Device selection:**

| Option | Selected |
|--------|----------|
| Default system input | ✓ |
| Explicit device id/name in config | |

**Capture format:**

| Option | Selected |
|--------|----------|
| Mono 16 kHz f32 | ✓ |
| Mono 16 kHz i16 | |
| Capture native + resample later | |

**PTT capture semantics:**

| Option | Description | Selected |
|--------|-------------|----------|
| a — start stream on PTT press | Stream lifecycle tied to PTT | |
| b — keep stream warm, gate samples | Stream always running; samples gated to pipeline | ✓ |
| c — hybrid warm + gate | | |

**Notes:** CPAL chosen for cross-driver portability. Mono 16 kHz f32 matches whisper.cpp's expected format, avoiding a Phase 2 resample stage.

---

## Input Injection Backend

| Option | Description | Selected |
|--------|-------------|----------|
| Keyboard-only uinput device | Simpler; mouse added later if needed | ✓ |
| Combined keyboard + mouse device | Future-proofs mouse macros | |

**Timing model:**

| Option | Selected |
|--------|----------|
| Per-key only | |
| Global defaults + per-key override | ✓ |
| Global-only | |

**Injection thread:**

| Option | Selected |
|--------|----------|
| Dedicated OS thread (blocking) | ✓ |
| Tokio async task | |

**Log verbosity:**

| Option | Selected |
|--------|----------|
| Always silent unless --verbose | ✓ |
| DEBUG level by default | |

---

## PTT Behavior

**Device targeting:**

| Option | Selected |
|--------|----------|
| Scan all /dev/input/event* for configured key | ✓ |
| Explicit device path in config | |

**Grab exclusivity:**

| Option | Description | Selected |
|--------|-------------|----------|
| Non-exclusive (PTT passes through) | Game still receives PTT key | ✓ |
| Exclusive EVIOCGRAB | Other apps lose PTT key | |

**Conflict handling:**

| Option | Selected |
|--------|----------|
| Fail with error | ✓ |
| Warn + fall back to non-grab | |
| Retry N times then fail | |

**Console feedback:**

| Option | Selected |
|--------|----------|
| Suppressed unless --verbose | ✓ |
| Always print to stdout | |

---

## Daemon + Config

**Config format:**

| Option | Selected |
|--------|----------|
| YAML | ✓ |
| TOML | |
| JSON | |

**Config location:**

| Option | Selected |
|--------|----------|
| XDG ($XDG_CONFIG_HOME/hd-linux-voice/config.yaml) | ✓ |
| Fixed ~/.config/hd-linux-voice/config.yaml | |
| Next to binary | |

**/dev/uinput error string:** Claude's discretion (template provided in CONTEXT.md D-15)

**LICENSES.md:**

| Option | Selected |
|--------|----------|
| cargo-about auto-generated (AGPL skipped) | ✓ |
| Manually curated | |
| Both | |

**User's notes:** AGPL v3.0 license already in repo root — skip it in the cargo-about inventory.

---

## Claude's Discretion

- Exact wording/URL in the /dev/uinput error message
- Internal Rust crate/workspace layout
- Tokio runtime configuration
- Logging framework (tracing vs log crate)

## Deferred Ideas

None.
