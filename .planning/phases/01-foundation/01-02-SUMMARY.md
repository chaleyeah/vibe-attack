---
phase: 01-foundation
plan: "02"
subsystem: config
tags: [config, yaml, xdg, clap, tracing, tdd]
one_liner: "Typed YAML config with XDG resolution via serde_yaml_ng + clap CLI with tracing init"

dependency_graph:
  requires: [01-01]
  provides: [config-structs, config-load, cli-entrypoint, tracing-init]
  affects: [01-03, 01-04, 01-05]

tech_stack:
  added: [serde_yaml_ng, xdg, clap, tracing-subscriber]
  patterns:
    - "Config struct hierarchy with deny_unknown_fields on all 4 structs"
    - "XDG path resolution: xdg::BaseDirectories::with_prefix + place_config_file"
    - "Tracing init: EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level))"
    - "TDD RED/GREEN: integration tests written before implementation"
    - "anyhow::Context wraps all I/O and parse errors — no unwrap() in load path"

key_files:
  created:
    - src/config.rs
    - src/lib.rs
    - config.example.yaml
    - tests/config_parse.rs
  modified:
    - src/main.rs

decisions:
  - "src/lib.rs created to expose config module for integration tests (crate-level pub mod)"
  - "macros field uses #[serde(default)] making it optional in config YAML"
  - "default_config_path() uses place_config_file (not get_config_file) — returns path unconditionally and creates config dir; get_config_file returns Option<PathBuf> (only Some if file exists)"
  - "init_logging uses unwrap_or_else not unwrap — RUST_LOG parse failure falls back to level string"

metrics:
  duration_seconds: 153
  completed_date: "2026-04-22T02:27:11Z"
  tasks_completed: 2
  files_changed: 5
---

# Phase 01 Plan 02: Config System + CLI Entry Point — Summary

Typed YAML config with XDG resolution via serde_yaml_ng + clap CLI with tracing init.

## Tasks Completed

| # | Task | Commit | Files |
|---|------|--------|-------|
| 1 | Config structs + XDG YAML load (TDD RED+GREEN) | 715f492, dbcefa2 | src/config.rs, src/lib.rs, config.example.yaml, tests/config_parse.rs |
| 2 | clap CLI + tracing init in main.rs | 43ac324 | src/main.rs |

## What Was Built

### src/config.rs

Four structs (`Config`, `PttConfig`, `TimingConfig`, `MacroConfig`, `KeyAction`) all carrying `#[serde(deny_unknown_fields)]`. `load()` accepts an optional path override (or falls back to XDG default), opens the file, and deserializes via `serde_yaml_ng::from_reader`. All errors are wrapped with `anyhow::Context` — no `unwrap()` in the load path.

### src/lib.rs

Created to expose `config`, `audio`, `error`, and `input` modules as a public library target so integration tests can import `hd_linux_voice::config::load`.

### config.example.yaml

Canonical example in repo root showing `ptt`, `timing`, and `macros` sections including a per-key `dwell_ms` override.

### src/main.rs

clap `Cli` struct: `--verbose` (count flag, DEBUG at `-v`, TRACE at `-vv`) and `--config FILE`. `init_logging` initializes a compact `tracing_subscriber` with `EnvFilter`, respecting `RUST_LOG` env var. `main()` calls `Config::load()`, printing actionable errors via `eprintln` before propagating.

## Verification Results

```
cargo test --test config_parse   →  6 passed, 0 failed
cargo build                      →  exit 0
cargo run -- --help              →  --verbose and --config visible
cargo run -- --config /tmp/nonexistent.yaml  →  non-zero exit, actionable error message
grep "deny_unknown_fields" src/config.rs     →  5 matches (one per struct + top-level)
grep "serde_yaml_ng" src/config.rs           →  match found
grep "unwrap()" src/config.rs               →  no matches
grep "BaseDirectories" src/config.rs        →  match found
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] xdg::BaseDirectories::with_prefix returns BaseDirectories directly (not Result)**
- **Found during:** Task 1 GREEN phase (compilation error)
- **Issue:** Plan code called `.context(...)` on `BaseDirectories`, but the type is not `Result`. The `xdg` v3 crate returns `BaseDirectories` infallibly from `with_prefix`.
- **Fix:** Removed `.context()` wrapper. Used `place_config_file("config.yaml")` (returns `Result<PathBuf>`, creating the config dir) instead of `get_config_file` (returns `Option<PathBuf>`, only Some if file exists). Changed `default_config_path` return type back to `Result<PathBuf>`.
- **Files modified:** src/config.rs
- **Commit:** dbcefa2 (incorporated in GREEN commit)

## Known Stubs

None — all fields are wired. The daemon loop in `main()` is intentionally deferred (comment documents Plan 05 as the implementation target). Config loading is fully functional.

## Threat Flags

No new threat surface beyond what the plan's threat model covers. `deny_unknown_fields`, typed deserialization, and no-unwrap load path implement all four threat mitigations (T-01-02-01 through T-01-02-04).

## Self-Check: PASSED

- [x] src/config.rs — exists
- [x] src/lib.rs — exists
- [x] config.example.yaml — exists
- [x] tests/config_parse.rs — exists, 6 tests green
- [x] src/main.rs — exists, --help works
- [x] Commits 715f492, dbcefa2, 43ac324 — all present in git log
