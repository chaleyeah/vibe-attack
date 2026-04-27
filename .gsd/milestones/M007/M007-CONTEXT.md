# M007: Codebase Cleanup & Documentation

## Vision

A first-time engineer reading this codebase should be able to understand the architecture, follow data flow from microphone to keypress, and contribute confidently — without needing to ask anyone. This milestone is a focused cleanup pass: dead code removed, abstractions collapsed where they add no value, modules trimmed to their essential surface, and every public type and function carrying enough documentation to stand on its own.

This is not a rewrite. The behavior does not change. The test suite passes before and after.

## Goals

1. **Remove dead code and legacy artifacts** — anything that was scaffolding, was replaced, or was left from the hd-linux-voice → vibe-attack migration and never cleaned up.
2. **Reduce line count where it costs nothing** — collapse trivial wrapper functions, consolidate related constants, remove redundant re-exports, flatten one-item modules.
3. **Improve internal consistency** — naming conventions, error handling patterns, and module organization should follow a single style across the whole codebase.
4. **Document the codebase for a new engineer** — every public module, struct, enum, trait, and function should have a doc comment explaining *what it does and why it exists*. Private implementation details get comments only where the behavior would surprise a reader.
5. **Audit `config.rs` and `error.rs`** — these are the load-bearing types every other module touches; they should be especially clean and well-documented.

## Scope

**In scope:**
- All files under `src/`
- `tests/` — remove obsolete test stubs, ensure test names and comments are accurate
- `Cargo.toml` — remove unused dependencies or features after cleanup
- `docs/` — verify that `configuration.md`, `troubleshooting.md`, and `uinput-setup.md` accurately reflect the current codebase
- Top-level `README.md` and `CONTRIBUTING.md` — ensure they describe vibe-attack (not hd-linux-voice) and reflect the current architecture

**Out of scope:**
- Behavioral changes
- New features
- Packaging changes (PKGBUILD, .spec, debian/)
- CI workflow changes

## Constraints

- All existing tests must pass at the end of every slice
- `cargo clippy -D warnings` must be clean throughout
- No public API surface changes that would break external callers (config file format stays stable)

## Key Architectural Areas to Review

The pipeline has a clear layered structure worth preserving and documenting:

```
Audio capture (src/audio/)
  → VAD (src/vad/)        — silence gating
  → Wake (src/wake/)      — keyword detection
  → STT (src/stt/)        — speech-to-text
  → Pipeline (src/pipeline/) — dispatch, matching, timing, sound, JSONL logging
  → Input (src/input/)    — uinput injection + PTT
```

Control plane: `src/control/` (Unix socket client/server protocol)  
Config: `src/config.rs` (single YAML-parsed config struct)  
Error: `src/error.rs` (unified error type)  
UI: `src/ui/` (GTK tray, wizard, first-run, probe), `src/tui/` (terminal UI), `src/bin/` (config binary)  
Pack: `src/pack/` (HD2 profile/bundle management)

## Success Criteria

- `cargo test` passes (all non-hardware-gated tests)
- `cargo clippy -D warnings` is clean
- Every public item in `src/` has a doc comment
- `grep -rn "hd.linux.voice\|hd_linux_voice\|hd2_linux\|TODO\|FIXME\|HACK\|dead_code\|allow(unused" src/` returns zero hits (or each remaining hit is explicitly justified in this milestone's learnings)
- The `README.md` accurately describes vibe-attack, its architecture, and how to build/run/configure it
- A new engineer can read `src/lib.rs` module-level docs and understand the full system in under 10 minutes

## Suggested Slice Decomposition

- **S01:** Dead code audit and removal (src/ + tests/)
- **S02:** Internal consistency pass — naming, error patterns, module organization
- **S03:** Public API documentation — all pub items in src/ get doc comments
- **S04:** Config + error type cleanup and documentation
- **S05:** README, CONTRIBUTING, and docs/ accuracy pass
