# M007: Codebase Cleanup & Documentation — Research

**Date:** 2026-04-27

## Summary

M007 is a focused cleanup and documentation pass over a well-structured but lightly documented ~6,000-line Rust codebase. The pipeline architecture is sound and layered cleanly. The primary work breaks into three real categories: (1) doc comments on ~100 undocumented public items, (2) a handful of genuine inconsistencies to fix, and (3) one dead dependency to remove. There are no major architectural surprises and no behavioral changes required.

The codebase is already free of legacy `hd-linux-voice` / `hd_linux_voice` references in `src/` (the one hit from the success-criteria grep is a `TODO` comment in `control/mod.rs` about a future `CancellationToken` wire-up, which is valid and intentional). The README and CONTRIBUTING.md already use `vibe-attack` throughout.

One real bug was found during research: `src/ui/config_app.rs::load_profiles()` scans for `profiles/*.yaml` flat files, but the pack system stores profiles as `profiles/{name}/pack.yaml` subdirectories. The two formats coexist on disk (evidence: `profiles/hd2.yaml` and `profiles/ImportTest/pack.yaml` both exist). `handle_switch_profile` in `control/mod.rs` uses `Pack::load_from_dir` (subdirectory format only), so the UI list can name a profile that the switch command cannot load. This should be fixed in S01 or S02 — it is a latent bug, not a doc issue.

## Recommendation

Execute the five slices as proposed in the milestone brief. Reorder S01 to include fixing the `load_profiles` inconsistency alongside dead code removal. S04 (config + error) can be brief — both files are already clean; the main gap is a duplicate doc comment on `default_config_path` in `config.rs`. S05 (README/docs) is also brief since the public docs are already accurate. The bulk of work is S03 (public API docs).

## Implementation Landscape

### Key Files

- `src/lib.rs` — 12 lines, no crate-level `//!` doc comment; must add pipeline architecture overview
- `src/config.rs` — 289 lines; `validate_model_paths`, `PipelineVerbosity`, and `default_config_path` are undocumented; `default_config_path` has a **duplicate doc comment** (lines 258 and 260 both start "Return the XDG config file path")
- `src/error.rs` — 32 lines; well-documented; `DaemonError::Config(String)` variant could clarify what "config" means
- `src/pipeline/dispatcher.rs` — `DispatcherState` is `pub` but only used internally; can be made `pub(crate)`; has `unsafe impl Send for Dispatcher {}` / `unsafe impl Sync for Dispatcher {}` that should carry a safety comment explaining why the `SoundPlayer` (which wraps non-`Send` `OutputStream`) is safe
- `src/pipeline/matcher.rs` — `PhraseMatcher`, `normalize`, `find_best_match` all lack doc comments
- `src/pipeline/sound.rs` — `SoundPlayer` struct and both methods lack doc comments
- `src/pipeline/timing.rs` — `MonoClock::start_now/elapsed/elapsed_ms` and several `UtteranceTimings` methods lack doc comments
- `src/pipeline/jsonl.rs` — `StageName`, `StageStatus`, `JsonlWriter::new/verbosity/write_*` methods undocumented; has one `#[allow(clippy::too_many_arguments)]` annotation that should carry an explanatory comment
- `src/pipeline/coordinator.rs` — `PipelineHandles` struct fields are undocumented; `spawn_pipeline` function is the most complex function in the codebase (~250 lines) and would benefit from a doc comment summarizing the thread topology
- `src/control/mod.rs` — `DaemonHandle::new/state/status` undocumented; one `TODO` comment about future `CancellationToken` wiring
- `src/control/client.rs` — `send_command/query_status/is_daemon_running` undocumented
- `src/control/protocol.rs` — all well-documented already
- `src/input/inject.rs` — `KeyStep::from_config`, `MacroCmd` enum undocumented; `DispatcherState` visibility concern noted above
- `src/pack/mod.rs` — `get_profiles_dir` undocumented; `load_profiles` inconsistency (see below)
- `src/tui/app.rs` — `App`, `AppMode`, `App::new/draw/handle_key` all public, none documented
- `src/tui/editor.rs` — `MacroEditor` and its methods undocumented
- `src/ui/config_app.rs` — `MAX_LOG_LINES`, `ConfigApp` fields undocumented; **`load_profiles` bug**: scans for `profiles/*.yaml` flat files but the canonical format is `profiles/{name}/pack.yaml` subdirectories
- `src/ui/probe.rs` — private helpers; `run()` is well-documented
- `src/ui/first_run.rs` — `SetupStep` variants undocumented; otherwise clean
- `src/ui/wizard.rs` — heavily feature-gated; inner types undocumented but acceptable given GUI scaffold status

### Specific Issues Found

| Category | File | Issue |
|----------|------|-------|
| Dead dependency | `Cargo.toml` | `sha2 = "0.10"` is listed but **never referenced** in any `src/` or `tests/` file |
| Duplicate doc | `src/config.rs:258–260` | `default_config_path` has two `///` doc lines saying the same thing |
| Latent bug | `src/ui/config_app.rs:load_profiles` | Scans for `*.yaml` flat files; pack system uses `{name}/pack.yaml` subdirs |
| Unnecessary pub | `src/pipeline/dispatcher.rs:DispatcherState` | Never used outside `dispatcher.rs`; should be `pub(crate)` or private |
| Missing safety comment | `src/pipeline/dispatcher.rs:55–56` | `unsafe impl Send/Sync` lacks `// SAFETY:` comment explaining why |
| Allow annotation | `src/pipeline/jsonl.rs:105` | `#[allow(clippy::too_many_arguments)]` has no explanation |
| VadConfig alias | `src/pipeline/coordinator.rs:24` | `use crate::vad::{VadConfig as SegCfg, ...}` — the alias `SegCfg` is inconsistent with the type's actual name; no note explaining why it was aliased |
| Control duplication | `src/control/mod.rs` + `src/control/client.rs` | Both define a private `fn get_socket_path()` with slightly different behavior (`place_runtime_file` vs `find_runtime_file`); this is intentional but undocumented |
| Missing crate doc | `src/lib.rs` | No `//!` crate-level doc comment; a new engineer reading the entry point gets no orientation |
| TODO | `src/control/mod.rs:129` | `// TODO: wire to CancellationToken in a future slice` for `ControlRequest::Shutdown` — valid outstanding work, not dead code; should be noted in M007 learnings |

### Undocumented Public Item Count (by module)

Approximately 100 public items lack doc comments, concentrated in:
- `src/pipeline/` (timing, jsonl, matcher, sound, dispatcher) — ~40 items
- `src/tui/` (app, editor) — ~10 items  
- `src/control/` (mod, client) — ~8 items
- `src/config.rs` — ~4 items (mostly method-level)
- `src/ui/` (config_app, first_run) — ~6 items
- `src/lib.rs` module declarations — all `pub mod` lines (~12 items)

### Build Order

1. **S01: Dead code audit + bug fix** — remove `sha2` dep, fix `load_profiles` bug, make `DispatcherState` `pub(crate)`. Run `cargo test` to confirm no regressions.
2. **S02: Internal consistency** — add `// SAFETY:` comment on `unsafe impl`, explain `SegCfg` alias, add note on dual `get_socket_path`, fix duplicate doc on `default_config_path`, explain `#[allow(clippy::too_many_arguments)]`.
3. **S03: Public API docs** — systematic pass adding `///` to all ~100 undocumented pub items. Heaviest slice; most time here.
4. **S04: Config + error cleanup** — these are already clean; primary task is to add any missing method-level docs and ensure `error.rs` error messages reference the correct docs URL.
5. **S05: README + docs/** — already accurate; verify `configuration.md` reflects current config struct fields, ensure `uinput-setup.md` references correct group names.

### Verification Approach

After each slice:
```bash
cargo test                              # all non-hardware-gated tests must pass
cargo check                             # no compile errors
grep -rn "hd.linux.voice\|hd_linux_voice\|hd2_linux\|TODO\|FIXME\|HACK\|dead_code\|allow(unused" src/
```

Final milestone verification:
```bash
# Every public item in src/ has a doc comment
python3 -c "
import re, pathlib
src = pathlib.Path('src')
undocumented = []
for f in sorted(src.rglob('*.rs')):
    lines = f.read_text().splitlines()
    for i, line in enumerate(lines):
        stripped = line.strip()
        if re.match(r'^pub\s+(fn|struct|enum|trait|type|const|mod)\s', stripped):
            preceding = lines[max(0,i-3):i]
            has_doc = any(l.strip().startswith('///') or l.strip().startswith('//!') for l in preceding)
            if not has_doc:
                undocumented.append(f'{f}:{i+1}')
print(len(undocumented), 'undocumented public items')
"
```

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Counting undocumented pub items | Script above (Python one-liner) | Already validated; same approach for before/after comparison |
| Checking dead dependencies | `cargo +nightly udeps` (if available) or manual grep | `sha2` grep confirms no usage; verify manually |

## Constraints

- Behavioral changes are out of scope — doc comments and visibility changes only (plus the `load_profiles` bug fix which changes UI behavior but not pipeline behavior)
- `cargo test` must pass at end of every slice — the test suite has 80+ tests spanning all modules
- `cargo clippy -D warnings` (once clippy is available) must be clean — current `#[allow(clippy::too_many_arguments)]` should become justified rather than removed
- `pub` visibility of `DispatcherState` can be reduced to `pub(crate)` without breaking the external API surface since it is never re-exported via `lib.rs`

## Common Pitfalls

- **`load_profiles` fix scope** — fixing this in S01 requires deciding the canonical format. The subdirectory format (`{name}/pack.yaml`) is what `Pack::load_from_dir` and `handle_switch_profile` use; the flat format is from a test run. Fix `load_profiles` to scan for subdirectories containing `pack.yaml`, not `*.yaml` flat files.
- **`unsafe impl Send/Sync` removal** — do NOT remove these without confirming `rodio::OutputStream` is now `Send`. As of rodio 0.17 it is not `Send`. The `unsafe impl` is correct; it needs a safety comment, not removal.
- **Doc comment length** — the goal is "explains what and why", not exhaustive parameter docs. One-liners are fine for trivial getters.

## Open Risks

- `sha2` removal: confirm it is not pulled in transitively in a way that requires re-exporting; `cargo check` after removal will catch any breakage
- `DispatcherState` visibility change: verify no integration tests reference it directly (grep shows zero external references — safe to narrow)
- `load_profiles` fix: the flat `hd2.yaml` on the developer's machine will stop appearing in the UI after the fix; this is correct behavior, but warrants noting in the slice summary

## Sources

- Codebase inspection: `src/` (all 33 files), `tests/` (15 files), `Cargo.toml`, `docs/`, `README.md`, `CONTRIBUTING.md`
- Developer environment: `~/.config/vibe-attack/profiles/` reveals mixed flat + subdirectory profile format
