---
phase: 01-foundation
plan: "01"
subsystem: build-toolchain
tags: [rust, cargo, toolchain, skeleton, test-stubs]
dependency_graph:
  requires: []
  provides:
    - "compilable Rust crate skeleton"
    - "all 13 Phase 1 crate dependencies pinned in Cargo.lock"
    - "integration test infrastructure (4 stub files)"
    - "cargo-about config for LICENSES.md generation"
  affects:
    - "all subsequent plans (cargo check baseline)"
tech_stack:
  added:
    - "Rust stable 1.95.0 (rustup)"
    - "cargo 1.95.0"
    - "cargo-about 0.8.4"
    - "cpal 0.17.3 (Apache-2.0)"
    - "evdev 0.13.2 with serde feature (Apache-2.0 OR MIT)"
    - "ringbuf 0.4.8 (MIT OR Apache-2.0)"
    - "tokio 1.52.1 with full feature (MIT)"
    - "serde 1.0.228 with derive feature (MIT OR Apache-2.0)"
    - "serde_yaml_ng 0.10.0 (MIT)"
    - "xdg 3.0.0 (Apache-2.0 OR MIT)"
    - "tracing 0.1.44 (MIT)"
    - "tracing-subscriber 0.3.23 with env-filter (MIT)"
    - "clap 4.6.1 with derive feature (MIT OR Apache-2.0)"
    - "tokio-util 0.7.18 (MIT)"
    - "anyhow 1.0.102 (MIT OR Apache-2.0)"
    - "thiserror 1.0.69 (MIT OR Apache-2.0)"
  patterns:
    - "single-crate layout (no workspace; workspace deferred to Phase 3+)"
    - "module hierarchy: audio/, input/ptt.rs, input/inject.rs"
    - "dedicated OS thread pattern for PTT and injection (std::thread, not spawn_blocking)"
    - "cargo-about with custom about.hbs to exclude root crate from license inventory"
key_files:
  created:
    - Cargo.toml
    - Cargo.lock
    - about.toml
    - about.hbs
    - src/main.rs
    - src/config.rs
    - src/error.rs
    - src/audio/mod.rs
    - src/input/mod.rs
    - src/input/ptt.rs
    - src/input/inject.rs
    - tests/config_parse.rs
    - tests/macro_inject.rs
    - tests/uinput_smoke.rs
    - tests/daemon_headless.rs
  modified: []
decisions:
  - "evdev serde feature enabled for future PTT config serialization"
  - "tokio-util used without a named feature (CancellationToken in sync module is always available; plan had incorrect 'sync' feature name)"
  - "about.hbs Handlebars template excludes root crate by name rather than via publish=false (preserves crates.io publishability)"
  - "serde_yaml_ng used instead of serde_yaml (deprecated March 2024, unresolved libyaml CVE)"
metrics:
  duration: "~3 minutes"
  completed_date: "2026-04-22"
  tasks_completed: 2
  tasks_total: 2
  files_created: 15
  files_modified: 0
---

# Phase 01 Plan 01: Rust Toolchain + Compilable Skeleton Summary

**One-liner:** Rust stable 1.95.0 installed; single-crate skeleton with 13 AGPL-compatible deps and 4 integration test stubs compiles clean under `cargo check`.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Install Rust toolchain and create Cargo.toml with all Phase 1 deps | ead0999 | Cargo.toml, about.toml, about.hbs |
| 2 | Create module skeleton and integration test stubs — cargo check passes | 409e7cd | src/main.rs, src/config.rs, src/error.rs, src/audio/mod.rs, src/input/mod.rs, src/input/ptt.rs, src/input/inject.rs, tests/config_parse.rs, tests/macro_inject.rs, tests/uinput_smoke.rs, tests/daemon_headless.rs, Cargo.lock |

## Verification Results

```
rustc 1.95.0 (59807616e 2026-04-14)
cargo 1.95.0 (f2d3ce0bd 2026-03-21)
cargo check: Finished `dev` profile [unoptimized + debuginfo] — 0 errors, 6 dead_code warnings (expected for stubs)
cargo test: 3 passed, 3 ignored (uinput-gated), 0 failed
```

- `serde_yaml ` (old crate): NOT present — OK
- GUI crates (winit/xcb/wayland-client/gtk/x11): NOT present — OK
- `about.toml` contains `"Apache-2.0"` — OK
- `about.hbs` contains `hd-linux-voice` exclusion guard — OK

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] tokio-util `sync` feature does not exist**
- **Found during:** Task 2 (first `cargo check` run)
- **Issue:** Plan specified `tokio-util = { version = "0.7", features = ["sync"] }` but tokio-util has no named `sync` feature. `CancellationToken` lives in `tokio_util::sync` and is always available.
- **Fix:** Removed `features = ["sync"]` — `tokio-util = { version = "0.7" }` resolves cleanly.
- **Files modified:** Cargo.toml
- **Commit:** 409e7cd (bundled with Task 2 commit)

## Decisions Made

1. **tokio-util without features:** `CancellationToken` in `tokio_util::sync` is unconditionally compiled; no feature flag needed.
2. **about.hbs exclusion by name:** Root crate excluded from LICENSES.md via `{{#unless (eq crate.name "hd-linux-voice")}}` template guard rather than `publish = false` (which would block crates.io publishing).
3. **serde_yaml_ng over serde_yaml:** Enforced per RESEARCH.md — serde_yaml deprecated March 2024 with unresolved libyaml CVE.

## Known Stubs

All source files are intentional stubs — no business logic yet. Subsequent plans implement:
- `src/config.rs` → Plan 02
- `src/audio/mod.rs` → Plan 03
- `src/input/ptt.rs` → Plan 03
- `src/input/inject.rs` → Plan 04
- `tests/config_parse.rs` → Plan 02 (real round-trip tests)
- `tests/macro_inject.rs` → Plan 04 (real injection tests)
- `tests/uinput_smoke.rs` → Plan 04 (VirtualDevice smoke test)
- `tests/daemon_headless.rs` → Plan 05 (binary spawn test)

These stubs are intentional — Plan 01 goal is compilation baseline, not implementation.

## Threat Surface Scan

No new network endpoints, auth paths, or file access patterns introduced beyond what the plan's threat model covers. Cargo.lock pins all 182 resolved crate hashes (T-01-01-02 mitigated). `serde_yaml_ng` confirmed present; `serde_yaml` confirmed absent (T-01-01-03 mitigated). No GUI crates in Cargo.toml (T-01-01-04 mitigated).

## Self-Check: PASSED

- [x] Cargo.toml exists: FOUND
- [x] about.toml exists: FOUND
- [x] about.hbs exists: FOUND
- [x] src/main.rs exists: FOUND
- [x] src/config.rs exists: FOUND
- [x] src/error.rs exists: FOUND
- [x] src/audio/mod.rs exists: FOUND
- [x] src/input/mod.rs exists: FOUND
- [x] tests/config_parse.rs exists: FOUND
- [x] tests/macro_inject.rs exists: FOUND
- [x] tests/uinput_smoke.rs exists: FOUND
- [x] tests/daemon_headless.rs exists: FOUND
- [x] Task 1 commit ead0999: FOUND
- [x] Task 2 commit 409e7cd: FOUND
- [x] cargo check exits 0: VERIFIED
- [x] cargo test exits 0 with 3 ignored tests: VERIFIED
