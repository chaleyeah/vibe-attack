# S06 Research: Documentation — Usage docs, troubleshooting, and contributor guides

**Slice goal:** Produce usage documentation, a troubleshooting guide, and a contributor guide; add `tests/documentation.rs` with structural tests that verify these files exist and contain required sections — proving "documentation works" the same way S05 proved packaging works.

---

## Summary

S06 is a **light-research, well-understood documentation slice**. The pattern is identical to the S05 packaging tests: create Markdown files under `docs/`, then write integration tests in `tests/documentation.rs` using `env!("CARGO_MANIFEST_DIR")` to assert each file exists and contains required headings/sections. No novel APIs, no new dependencies, no risky integrations.

---

## Implementation Landscape

### Existing docs/ structure (what already exists)

```
docs/
├── latency-baseline.md       # Phase 2 latency proof procedure (exists, thorough)
├── latency-proofs/           # Evidence archive for target-hardware runs
│   └── phase-02-target-hardware/
│       ├── README.md
│       ├── RESULTS.template.md
│       ├── results-ptt/RESULTS.md
│       └── results-wake/RESULTS.md
└── uinput-setup.md           # uinput permission setup guide (exists, thorough)
```

**Gaps to fill:**
- No `CONTRIBUTING.md` — contributor guide is missing
- `README.md` exists but uses the old project name ("vibe-attack"), references portaudio (old dep), and is out of date with actual CLI commands
- No `docs/troubleshooting.md` — troubleshooting is scattered across README and uinput-setup.md
- No `docs/configuration.md` — config reference document (README.md refers to a `CONFIG.md` that doesn't exist)
- No `docs/pack-format.md` — hdpack format spec for pack authors

### README.md state

The existing `README.md` (line 1: "# vibe-attack") is stale:
- Project name: "vibe-attack" (should be "hd-linux-voice")
- Installation: references portaudio (not used; actual dep is ALSA/CPAL)
- Running: uses `./target/release/hd-linux-voice run` — but `run` is not a subcommand in `src/main.rs`. The actual daemon starts with no subcommand.
- Commands listed include `edit` (TUI) which exists; `import` and `export` which exist; `ping`, `switch`, `test` which also exist (S03/S04)
- Missing: `--list-devices`, `-v`/`-vv` verbosity flags, config file location, JSONL stdout contract, PTT evdev setup

The README needs a significant rewrite to reflect the current state.

### CLI surface (from src/main.rs)

```
hd-linux-voice [OPTIONS] [COMMAND]

Options:
  -v, --verbose       DEBUG (-v) / TRACE (-vv)
  -c, --config FILE   Path to config file
  --list-devices      List CPAL audio input devices and exit
  -h, --help
  -V, --version

Commands:
  ping                Check daemon is alive (UDS control socket)
  switch <name>       Switch active macro pack/profile at runtime
  test <name>         Execute a macro by name immediately
  import <file>       Import a .hdpack file
  export <name> [output]  Export profile to .hdpack
  edit                Open TUI macro editor
```

Default (no subcommand): start daemon. Stdout = JSONL transcripts; stderr = logs.

### config.example.yaml structure (from file)

The annotated `config.example.yaml` is the de-facto config reference. A `docs/configuration.md` should be a prose companion to it, explaining each section: `ptt`, `audio`, `timing`, `pipeline`, `vad`, `stt`, `wake`, `macros`.

### Pack format (from tests/pack_hd2_bundle.rs + src/pack/)

The `.hdpack` format is a zip archive. Key fields: `name`, `phrases[]`, `key_sequence[]`, `dwell_ms`, `gap_ms`, `category`, `description`. Pack files contain a YAML manifest. A `docs/pack-format.md` would help community pack authors.

### Test pattern (from tests/ui_distribution.rs)

All structural doc tests follow this pattern:
```rust
let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
let file = root.join("path/to/file.md");
assert!(file.exists(), "file must exist");
let contents = std::fs::read_to_string(&file).expect("failed to read file");
assert!(contents.contains("## Installation"), "missing section");
```

Tests run without display server, network access, or model files — pure filesystem checks.

---

## Recommended Task Decomposition

### T01 — Rewrite README.md

Update `README.md` to reflect current state:
- Rename project header to "hd-linux-voice"
- Fix installation: ALSA deps, not portaudio
- Correct quick-start: no `run` subcommand; daemon starts with no subcommand
- Document all CLI commands (from `src/main.rs` Commands enum)
- Document stdout JSONL / stderr logs contract
- Add model download section (Whisper tiny.en, no auto-download)
- Add uinput setup (link to `docs/uinput-setup.md`)
- Add config file location (`~/.config/hd-linux-voice/config.yaml`)

**Files:** `README.md`

### T02 — Create docs/troubleshooting.md

Consolidate all known failure modes into a single reference:

Key sections:
- `## uinput / /dev/uinput` — permission denied, module not loaded, systemd v258+ group issue (from `docs/uinput-setup.md`)
- `## Audio / CPAL` — no input devices found, wrong device selected (use `--list-devices`)
- `## Models not found` — Whisper model path, ONNX Runtime dylib, sherpa-onnx artifacts
- `## STT accuracy` — low confidence, noisy environment, vocabulary mismatch
- `## Daemon unresponsive` — UDS socket missing, ping fails
- `## Build failures` — ALSA sys missing, whisper-rs cmake, feature flags

**Files:** `docs/troubleshooting.md`

### T03 — Create CONTRIBUTING.md

Standard contributor guide:
- Dev environment prerequisites (Rust stable, ALSA, evdev/uinput access)
- Build variants (`cargo build`, `cargo build --features stt`, `cargo build --features gui`)
- Running tests (`cargo test` — all pass without hardware; hardware tests are opt-in)
- Architecture overview (two-stage pipeline, channel contract, no cross-component direct calls)
- Coding conventions (no allocations in audio callback, STT always spawn_blocking, stdout = JSONL only)
- PR process (slice-based; tests must pass; new features need slice plan)
- Pack authoring (link to `docs/pack-format.md`)

**Files:** `CONTRIBUTING.md`

### T04 — Create docs/configuration.md

Prose companion to `config.example.yaml`:
- Each section explained (ptt, audio, timing, pipeline, vad, stt, wake, macros)
- XDG config path
- How to use `--config` override
- Model path conventions
- Per-key dwell override syntax

**Files:** `docs/configuration.md`

### T05 — Add tests/documentation.rs

Structural tests following the `env!("CARGO_MANIFEST_DIR")` pattern from S05:

Target: **8+ tests** (one per required doc file/section):

1. `readme_exists` — `README.md` exists
2. `readme_has_installation_section` — contains `## Installation`
3. `readme_has_usage_section` — contains `## Usage` or `## Running`
4. `readme_has_correct_project_name` — contains `hd-linux-voice` (not `vibe-attack`)
5. `troubleshooting_doc_exists` — `docs/troubleshooting.md` exists
6. `troubleshooting_has_uinput_section` — contains `uinput`
7. `contributing_exists` — `CONTRIBUTING.md` exists
8. `contributing_has_build_section` — contains `cargo build` or `## Building`
9. `configuration_doc_exists` — `docs/configuration.md` exists
10. `configuration_has_ptt_section` — contains `ptt`
11. `readme_does_not_reference_portaudio` — `portaudio` should not appear (stale dep reference)

**Files:** `tests/documentation.rs`

---

## What to Build First

1. **T05 tests** — write the tests first (they all fail); they define the contract
2. **T01 README rewrite** — highest-visibility; unblocks tests 1–4, 11
3. **T02 troubleshooting.md** — unblocks tests 5–6
4. **T03 CONTRIBUTING.md** — unblocks tests 7–8
5. **T04 configuration.md** — unblocks tests 9–10

All five tasks are independent after T05 establishes the contracts. No new Rust code needed; no new dependencies.

---

## Verification Commands

```bash
# All doc tests should pass without hardware or models
cargo test --test documentation

# Confirm README no longer references stale content
grep -n "vibe-attack\|portaudio" README.md  # should return nothing

# Count tests
grep -c '#\[test\]' tests/documentation.rs  # expect >= 8
```

---

## Constraints / Gotchas

- **README.md currently says "vibe-attack"** — test `readme_has_correct_project_name` will catch regressions
- **`run` is not a real subcommand** — README correction is critical; users trying `hd-linux-voice run` get a clap error
- **No CONTRIBUTING.md means no contributor guide at all** — this is a gap for an AGPL project that wants community packs
- **config.example.yaml is the authoritative config spec** — `docs/configuration.md` should reference/echo it, not diverge
- **docs/uinput-setup.md is already thorough** — `docs/troubleshooting.md` should cross-link it rather than duplicate it
- **No new Rust source changes needed** — all deliverables are Markdown + one new test file

---

## Forward Intelligence for Planner

- Test file name: `tests/documentation.rs` (not `docs.rs` — avoids conflict with any future rustdoc integration)
- All tests are `#[test]` functions using only `std::fs` and `env!("CARGO_MANIFEST_DIR")` — no use statements needed beyond the standard prelude
- The `readme_does_not_reference_portaudio` negative assertion test is useful as a regression guard; the project has historically had portaudio as a dep and it crept into docs
- `docs/configuration.md` scope: prose only — do NOT add a new TOML/YAML schema file; `config.example.yaml` is already the canonical spec
- The S05 summary notes `build.sh execute bit not set` as a deviation; consider a test `build_script_has_shebang` already exists in `ui_distribution.rs` — no need to re-test in S06
- Keep CONTRIBUTING.md focused on getting code running and contributing a pack; detailed architecture is already in `.planning/research/ARCHITECTURE.md` (cross-link, don't copy)
