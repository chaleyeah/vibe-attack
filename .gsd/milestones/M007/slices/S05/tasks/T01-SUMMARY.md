---
id: T01
parent: S05
milestone: M007
key_files:
  - README.md
key_decisions:
  - Removed 'double-tap detection' claim (not implemented in codebase)
  - Added Feature Flags section before Installation to surface the stt/gui opt-in requirement upfront
duration: 
verification_result: passed
completed_at: 2026-04-27T12:24:27.599Z
blocker_discovered: false
---

# T01: Updated README.md to accurately reflect feature flags, build commands, and CLI flags — removing false "double-tap detection" claim and adding missing stt/gui feature documentation

**Updated README.md to accurately reflect feature flags, build commands, and CLI flags — removing false "double-tap detection" claim and adding missing stt/gui feature documentation**

## What Happened

Read README.md line-by-line and cross-checked it against src/lib.rs, src/config.rs, Cargo.toml, and src/main.rs.

**Findings and fixes:**

1. **Project name (a)**: "vibe-attack" used correctly throughout. No drift.

2. **Architecture description (b)**: README matches lib.rs crate docs at a high level. No drift in the narrative description of the audio→keypress pipeline.

3. **Build/run instructions (c)**: The default `cargo build --release` command was accurate but incomplete — it did not communicate that the default feature set has no STT. Added a "Clone and Build" section with four build variants: default (no STT), `--features stt`, `--features stt-vulkan`, and `--features stt,gui`. Also clarified that `vibe-attack-config` binary is produced by the `gui` feature.

4. **Configuration section (d)**: README's Configuration section correctly defers to docs/configuration.md with no field names inline — no drift possible.

5. **Feature flags (e)**: README had NO feature flags section at all. Added a "Feature Flags" table before Installation explaining default (no STT), stt, stt-vulkan, and gui with what each adds.

6. **Whisper Model section**: Updated to state that STT requires BOTH the `stt` feature flag and a model file — the original implied it was always available.

7. **False feature claim**: "Fail-Safe Design: Built-in delays and double-tap detection to prevent accidental activations" — grep found zero references to double-tap/debounce in the codebase. Replaced with accurate "Configurable Timing" bullet referencing dwell_ms and gap_ms from TimingConfig.

8. **CLI flags**: Updated flags table to show the short forms `-v`/`--verbose` and `-c`/`--config` as confirmed by running the binary with `--help`.

9. **Build verification**: `cargo build` compiled cleanly. Binary confirmed to have correct name (`vibe-attack`), flags (`-v`, `-c`, `--list-devices`), and subcommands (`ping`, `switch`, `test`, `import`, `export`, `edit`) via `--help` output.

## Verification

Ran `cargo build` — compiled cleanly in 2.30s. Ran `./target/debug/vibe-attack --help` (with LD_LIBRARY_PATH set for sherpa shared lib) — confirmed binary name, all flags, and all subcommands match the updated README exactly. Cross-checked every README claim against Cargo.toml [features], src/config.rs Config struct fields, and src/main.rs Cli/Commands definitions.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build` | 0 | ✅ pass | 2300ms |
| 2 | `LD_LIBRARY_PATH=./target/debug ./target/debug/vibe-attack --help` | 0 | ✅ pass | 80ms |

## Deviations

None — all changes were corrections of drift, not scope additions.

## Known Issues

none

## Files Created/Modified

- `README.md`
