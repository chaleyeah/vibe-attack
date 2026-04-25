# T01: 01-foundation 01

**Slice:** S01 — **Milestone:** M001

## Description

Install the Rust toolchain and create a compilable project skeleton: Cargo.toml with all 13
Phase 1 dependencies, a module stub hierarchy (config, audio, input, error), four integration
test stubs, and cargo-about configuration files.

Purpose: Every subsequent plan depends on `cargo check` passing. This plan clears the
compilation baseline so Wave 2+ plans can focus purely on implementation.

Output: A compilable Rust crate with all deps resolved, stub modules in place, and
test infrastructure ready for Wave 1–4 implementations to fill in.

## Must-Haves

- [ ] "rustup, cargo, and rustc are available on PATH after plan execution"
- [ ] "cargo check exits 0 — all 13 deps resolved, module skeleton compiles"
- [ ] "All four integration test stub files exist in tests/ and cargo test compiles"
- [ ] "about.toml and about.hbs are present for cargo-about generation in Wave 5"
- [ ] "No GUI/display-server crate (winit, xcb, wayland-client, gtk) appears in Cargo.toml"

## Files

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
