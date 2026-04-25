---
id: T02
parent: S01
milestone: M001
provides: []
requires: []
affects: []
key_files: []
key_decisions: []
patterns_established: []
observability_surfaces: []
drill_down_paths: []
duration: 
verification_result: passed
completed_at: 
blocker_discovered: false
---
# T02: 01-foundation 02

**# Phase 01 Plan 02: Config System + CLI Entry Point — Summary**

## What Happened

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
