---
phase: 1
slug: foundation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-21
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (`cargo test`) |
| **Config file** | `Cargo.toml` `[dev-dependencies]` |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~5–15 seconds (lib tests); ~30 seconds (full suite incl. integration) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds (lib tests)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 1-ptt-unit | TBD | 1 | ACT-01 | — | PTT state changes are observable via log even when game is fullscreen | unit (mock) | `cargo test --lib audio::ptt` | ❌ W0 | ⬜ pending |
| 1-ptt-scan | TBD | 1 | ACT-01 | — | Device scanning handles missing permissions gracefully | unit | `cargo test --lib input::ptt` | ❌ W0 | ⬜ pending |
| 1-macro-inject | TBD | 2 | MCRO-01 | — | Key sequence fires with configurable inter-key gaps | integration | `cargo test --test macro_inject` | ❌ W0 | ⬜ pending |
| 1-key-hold | TBD | 2 | MCRO-02 | — | Key-hold emits press, sleeps dwell_ms, emits release | unit | `cargo test --lib input::inject` | ❌ W0 | ⬜ pending |
| 1-uinput-smoke | TBD | 2 | MCRO-05 | T-access | uinput VirtualDevice creation + event emit; fail-fast if /dev/uinput inaccessible | integration (privileged) | `cargo test --test uinput_smoke` | ❌ W0 | ⬜ pending |
| 1-headless | TBD | 2 | UI-01 | — | Daemon starts with no display surface (no Wayland/X11 socket interaction) | integration | `cargo test --test daemon_headless` | ❌ W0 | ⬜ pending |
| 1-license-check | TBD | 1 | DIST-03 | — | All deps MIT/Apache-2.0/LGPL; LICENSES.md is current | automated CI | `cargo about generate about.hbs --fail-incomplete` | ❌ W0 | ⬜ pending |
| 1-config-parse | TBD | 1 | — | T-config | Malformed YAML rejected cleanly; no panic in config load path | unit | `cargo test --lib` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/macro_inject.rs` — integration test stub for key sequence injection; covers MCRO-01
- [ ] `tests/uinput_smoke.rs` — smoke test stub for VirtualDevice creation + key emit; covers MCRO-05
- [ ] `tests/daemon_headless.rs` — spawns daemon binary, checks no display socket opened; covers UI-01
- [ ] `tests/config_parse.rs` — roundtrip YAML config parse tests; covers config path for D-13/D-14

**CI strategy for uinput tests:** Integration tests requiring `/dev/uinput` or `/dev/input` must be gated behind `RUN_PRIVILEGED_TESTS=1`. Standard CI runs `cargo test --lib` only; a separate privileged job runs the full integration suite.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| PTT key is observable when fullscreen Steam/Proton game is foregrounded | ACT-01 (SC-1) | Fullscreen game focus cannot be automated in CI | Launch Helldivers 2 (or any fullscreen game), hold PTT, observe console log showing audio-capture-start |
| Test macro injects key sequence into focused Wayland window (fullscreen) | MCRO-05 (SC-2) | Real Wayland compositor + fullscreen game required | Run test macro while a fullscreen Wayland game is focused; observe key events delivered in-game |
| Daemon launches without minimizing/interrupting a running fullscreen game | UI-01 (SC-3) | Requires real compositor | Start fullscreen game, then launch daemon; confirm no focus steal or minimize event |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
