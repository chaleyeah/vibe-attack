# S05: README, CONTRIBUTING, and docs/ accuracy pass — UAT

**Milestone:** M007
**Written:** 2026-04-27T12:35:37.144Z

# S05 UAT — Documentation Accuracy Pass

## Preconditions

- Fresh clone of the repository at the post-S05 commit
- Rust toolchain installed with cargo
- libclang-dev (Debian/Ubuntu) or clang (Arch) installed

---

## Test Cases

### TC-01: README Feature Flags section exists and is accurate

1. Open `README.md` and search for "Feature Flags"
2. **Expected**: A table or section exists before the Installation section listing at minimum: default (no STT), stt, stt-vulkan, gui
3. Run `grep '\[features\]' Cargo.toml` to confirm these feature names exist in Cargo.toml
4. **Expected**: All feature names in the README table match entries in Cargo.toml [features]

### TC-02: README build commands work

1. Run `cargo build` (default, no features)
2. **Expected**: Compiles cleanly, produces `target/debug/vibe-attack` binary
3. Run `./target/debug/vibe-attack --help`
4. **Expected**: Shows flags `-v`/`--verbose`, `-c`/`--config`, `--list-devices`; subcommands include `ping`, `switch`, `test`, `import`, `export`, `edit`
5. Verify no `daemon` subcommand appears in --help output
6. **Expected**: No `daemon` subcommand listed

### TC-03: README contains no false feature claims

1. Search README.md for "double-tap"
2. **Expected**: Zero matches — the false "double-tap detection" claim is absent
3. Search README.md for "dwell_ms" or "gap_ms" or "Configurable Timing"
4. **Expected**: At least one match — the accurate timing description replaces the removed claim

### TC-04: CONTRIBUTING.md prerequisites match CI

1. Open `.github/workflows/ci.yml` and note the apt-get install packages
2. Open `CONTRIBUTING.md` and find the Prerequisites section
3. **Expected**: Both `libasound2-dev` and `libclang-dev` are listed for Debian/Ubuntu
4. Check the PR Process section for clippy invocation
5. **Expected**: Shows `cargo clippy --all-targets -- -D warnings` (not bare `cargo clippy`)

### TC-05: CONTRIBUTING.md module list is complete

1. Run `ls src/` in the repository root
2. Open `CONTRIBUTING.md` Coding Conventions section
3. **Expected**: All 11 modules are listed: audio, vad, wake, stt, input, pipeline, pack, control, config, ui, tui

### TC-06: docs/configuration.md covers all config fields

1. Run `grep -n '^\s*pub ' src/config.rs | grep -v 'pub fn\|pub struct\|pub enum'`
2. Note the field names (35 total)
3. For each field, search `docs/configuration.md` for its name
4. **Expected**: Every pub field appears in docs/configuration.md
5. Specifically verify: `stt.confidence_threshold`, `macro.phrase`, `macro.if_flag`, `macro.set_flag`, `macro.sound`, `keys[].gap_ms` are all present
6. **Expected**: All six fields are documented with accurate types and defaults

### TC-07: MacroConfig.name vs phrase distinction is clear

1. Open `docs/configuration.md` and find the macro configuration section
2. Find the `name` field description
3. **Expected**: Description says something like "unique identifier used in logs and as the flag namespace" — NOT "phrase Whisper must recognise"
4. Find the `phrase` field description
5. **Expected**: phrase is documented as the optional spoken trigger string

### TC-08: docs/troubleshooting.md daemon restart command is correct

1. Open `docs/troubleshooting.md` and search for the daemon restart example
2. **Expected**: No `vibe-attack daemon &` command appears
3. **Expected**: The restart example uses `vibe-attack > /dev/null 2>&1 &` or equivalent direct invocation
4. Run `./target/debug/vibe-attack --help` and confirm no `daemon` subcommand exists
5. **Expected**: Matches — troubleshooting example does not reference a non-existent subcommand

### TC-09: Ping response casing matches binary output

1. Run the daemon in background: `./target/debug/vibe-attack > /dev/null 2>&1 &`
2. Run `./target/debug/vibe-attack ping`
3. **Expected**: Output contains `Pong` (capital P, matching Debug repr of ControlResponse::Pong)
4. Open `docs/troubleshooting.md` and find the ping section
5. **Expected**: Doc shows `Pong` (not `pong`)

### TC-10: docs/uinput-setup.md uses correct group name

1. Open `docs/uinput-setup.md`
2. Search for the group name used in usermod/udev instructions
3. **Expected**: Group is `input` (not `uinput`)
4. Search for "udev rule" or ".rules"
5. **Expected**: No udev rule section exists — only group membership and module-load steps are documented

### TC-11: Final M007 grep returns only the justified TODO

1. Run `grep -rn 'hd.linux.voice\|hd_linux_voice\|hd2_linux\|TODO\|FIXME\|HACK\|dead_code\|allow(unused' src/`
2. **Expected**: Exactly one hit — `src/control/mod.rs:135` containing the CancellationToken TODO
3. Zero other hits

### TC-12: cargo test passes

1. Run `cargo test`
2. **Expected**: All non-hardware-gated tests pass (exit 0); hardware-gated tests are ignored, not failed
