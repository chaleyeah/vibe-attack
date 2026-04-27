# M007: Codebase Cleanup & Documentation

**Vision:** A first-time engineer reading this codebase can understand the architecture, follow data flow from microphone to keypress, and contribute confidently without asking anyone. M007 is a focused cleanup pass: dead code removed, abstractions collapsed where they add no value, modules trimmed to essential surface, and every public type and function carrying enough documentation to stand on its own. Behavior does not change. The test suite passes before and after every slice.

## Success Criteria

- cargo test passes (all non-hardware-gated tests) at the end of every slice
- cargo clippy -D warnings is clean throughout (both default and gui feature sets)
- Every public item in src/ has a doc comment (verified by research script counting pub fn/struct/enum/trait/type/const/mod with no preceding /// or //!)
- grep for hd.linux.voice|hd_linux_voice|hd2_linux|TODO|FIXME|HACK|dead_code|allow(unused) in src/ returns zero hits or each remaining hit is explicitly justified in milestone learnings
- README.md accurately describes vibe-attack, its architecture, and how to build/run/configure it
- A new engineer can read src/lib.rs module-level docs and understand the full system in under 10 minutes
- load_profiles UI bug fixed — the config UI lists profiles in the canonical {name}/pack.yaml subdirectory format that handle_switch_profile actually loads
- sha2 dead dependency removed from Cargo.toml

## Slices

- [x] **S01: S01** `risk:high` `depends:[]`
  > After this: cargo test passes; cargo clippy -D warnings clean; cargo check confirms no transitive breakage from sha2 removal; new integration test asserts load_profiles returns subdirectory-format profiles and ignores flat .yaml files; success-criteria grep returns zero hits in src/ except justified TODO in control/mod.rs

- [x] **S02: S02** `risk:low` `depends:[]`
  > After this: cargo test passes; cargo clippy -D warnings clean; grep for `unsafe impl` in src/pipeline/dispatcher.rs shows a // SAFETY: comment immediately above each impl; the SegCfg alias in coordinator.rs has an explanatory comment; the dual get_socket_path functions in control/mod.rs and control/client.rs have a comment in each describing the intentional split; the duplicate doc comment on default_config_path in config.rs is collapsed; the #[allow(clippy::too_many_arguments)] on jsonl.rs has a justification comment

- [ ] **S03: S03** `risk:medium` `depends:[]`
  > After this: cargo test passes; cargo clippy -D warnings clean; the Python undocumented-pub-item audit script from M007-RESEARCH.md reports 0 undocumented public items in src/; src/lib.rs has a //! crate-level doc comment describing the audio → VAD → wake → STT → pipeline → input architecture; spot-check of 10 random pub items shows doc comments explain why the item exists, not just restate the name

- [ ] **S04: Config and error type cleanup** `risk:low` `depends:[S03]`
  > After this: cargo test passes; cargo clippy -D warnings clean; src/config.rs and src/error.rs have full doc coverage on every public item; the duplicate default_config_path doc is gone (already done in S02 — verify); DaemonError variant docs explain what each variant represents and where it originates

- [ ] **S05: README, CONTRIBUTING, and docs/ accuracy pass** `risk:low` `depends:[S04]`
  > After this: README.md describes vibe-attack accurately, including the audio → keypress pipeline, build/run/configure steps, and the feature flags (default vs gui); CONTRIBUTING.md reflects the current dev setup; docs/configuration.md fields match the current Config struct; docs/troubleshooting.md references current binary names and uinput group conventions; docs/uinput-setup.md references the correct group name and udev rule; cargo test passes; cargo clippy -D warnings clean

## Boundary Map

## Boundary Map

M007 touches no external services, no network surfaces, no new dependencies, and no protocol changes.

### Internal boundaries touched

- **Cargo.toml** — remove `sha2` dependency (S01). No version bumps, no new dependencies.
- **src/ui/config_app.rs ↔ src/pack/mod.rs** — fix `load_profiles` to read the canonical pack subdirectory format (S01). Aligns the UI with the format `Pack::load_from_dir` and `handle_switch_profile` already use.
- **src/pipeline/dispatcher.rs** visibility — narrow `DispatcherState` from `pub` to `pub(crate)` (S01). Verified: zero external references in tests/ or other src/ modules.
- **src/lib.rs** — add crate-level `//!` doc comment describing pipeline architecture (S03). Public surface unchanged; only documentation added.

### Untouched (explicitly out of scope)

- Config file format (YAML schema stays stable — DIST/PACK requirement)
- JSONL log schema
- Control plane Unix socket protocol
- All public API signatures
- All packaging files (PKGBUILD, .spec, debian/, AppImage scripts)
- All CI workflows
