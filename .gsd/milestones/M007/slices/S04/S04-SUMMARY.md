---
id: S04
parent: M007
milestone: M007
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - ["src/config.rs", "src/error.rs", "src/stt/mod.rs"]
key_decisions:
  - ["DaemonError::Config preserved as unused but documented — config errors currently propagate as anyhow::Error; type retained for future tightening without removing API surface", "Broken [Display] intra-doc link fixed to [std::fmt::Display] (fully-qualified) — short form does not resolve without import", "yourusername/vibe-attack placeholder in UinputPermissionDenied Display string left unchanged — behavior change is out of scope for M007", "cargo check used as clippy substitute — clippy not installed in this environment (established convention since S03/T05)"]
patterns_established:
  - ["Final-pass verification task (T03-pattern) caught two defects missed by task-level checks: intra-doc link resolution failures only surface under cargo doc, and pub field docs on non-top-level structs require the audit script to scan all nesting depths", "M007 audit script must use anchored regex (^pub) to avoid matching pub(crate)/pub(super) items — restricted-visibility items do not require public doc comments"]
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-27T12:21:44.821Z
blocker_discovered: false
---

# S04: Config and error type cleanup

**Full doc coverage on src/config.rs and src/error.rs: every public item documented with semantics, origin, and recovery hints; cargo test clean (40 passed), cargo check clean, cargo doc 0 warnings, audit script reports 0 undocumented pub items in src/.**

## What Happened

S04 targeted the two load-bearing modules touched by every other part of the codebase — `src/config.rs` and `src/error.rs` — to ensure every public item carries a doc comment a new engineer can understand without diving into call sites.

**T01 (config.rs audit and documentation):** Prior sessions had already documented top-level structs, `validate_model_paths`, `default_config_path`, `load`, `PipelineVerbosity` (struct-level), and `AudioConfig`/`PttConfig`/`TimingConfig`/`KeyAction` fields. The remaining gaps were: `Config` struct fields (`ptt`, `timing`, `audio`, `pipeline`, `vad`, `stt`, `wake`, `macros`), `PipelineVerbosity` enum variants (`Summary`, `Stages`), all seven `VadConfig` fields (`start_threshold`, `stop_threshold`, `min_speech_ms`, `end_silence_ms`, `preroll_ms`, `tail_ms`, `max_utterance_secs`), `SttConfig` fields (`enabled`, `model_path`, `confidence_threshold`), six `WakeConfig` fields (`enabled`, `encoder`, `decoder`, `joiner`, `tokens`, `keywords`), and six `MacroConfig` fields (`name`, `phrase`, `if_flag`, `set_flag`, `sound`, `keys`). All received `///` doc comments explaining semantics, defaults, and required-when conditions. Post-edit audit confirmed 0 undocumented items in `config.rs`.

**T02 (error.rs audit and documentation):** `DaemonError` had minimal one-line docs. Each variant was enriched to cover: (a) the condition that produces it, (b) the originating module/operation, (c) the recovery action a caller can take. Key finding: `DaemonError::Config` is defined but currently unused in production paths — config errors propagate as `anyhow::Error`. Rather than removing it, its doc was updated to note this explicitly, preserving the type for future narrowing. The enum-level doc was expanded to three sentences explaining that these are process-exit errors with actionable `Display` messages. Cross-references to `docs/uinput-setup.md` and `docs/configuration.md` were embedded in the relevant variant docs. A pre-existing placeholder URL (`yourusername/vibe-attack`) in a `#[error(...)]` Display string was left unchanged — it is behavior, not documentation, and out of scope per M007's zero-behavior-change constraint.

**T03 (final verification pass):** The full verification suite surfaced two minor defects not caught in T01/T02: (1) a broken intra-doc link `[Display]` in `error.rs` — resolved by replacing with the fully-qualified `[std::fmt::Display]`; (2) a missing `///` doc comment on `pub result_rx: Receiver<SttResult>` in `src/stt/mod.rs:54` — added. Both were within-scope fixes (M007 goal: zero undocumented pub items, zero doc warnings). After both fixes, cargo test passed 40/40 (default), 43/43 (gui, with one transient parallel-test flake in the unrelated pack module that resolves on second run), `cargo check --all-targets` clean on both feature sets, `cargo doc --no-deps` 0 warnings, and the canonical audit script reported PASS: 0 undocumented pub items in src/.

## Verification

cargo test (default): 40 passed, 0 failed, 6 ignored. cargo test --features gui: 43 passed, 0 failed, 1 ignored (transient pack flake resolves on second run). cargo check --all-targets (default): clean. cargo check --all-targets --features gui: clean. cargo doc --no-deps: 0 warnings. Canonical M007 audit script (3-line lookahead, anchored pub-only regex): PASS — 0 undocumented pub items in src/.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

T03 fixed two defects not in the task plan: (1) broken [Display] intra-doc link in error.rs → [std::fmt::Display]; (2) missing /// doc on pub result_rx field in stt/mod.rs:54. Both are within-scope for M007's zero-undocumented-items and zero-doc-warnings goals.

## Known Limitations

The yourusername/vibe-attack placeholder URL in DaemonError::UinputPermissionDenied's #[error(...)] Display string remains. It is in behavior (Display output shown to end users), not documentation, so it is out of scope for M007. It should be addressed when the project name and repository URL are finalized.

## Follow-ups

S05 (README, CONTRIBUTING, docs/ accuracy pass) is the natural next step. It should verify that docs/uinput-setup.md, docs/configuration.md, and docs/troubleshooting.md cross-references added in S04 doc comments are accurate and up-to-date.

## Files Created/Modified

- `src/config.rs` — Added /// doc comments to all previously undocumented pub struct fields across Config, VadConfig, SttConfig, WakeConfig, MacroConfig; added docs to PipelineVerbosity variants Summary and Stages
- `src/error.rs` — Expanded DaemonError enum-level doc and all four variant docs to cover condition, origin, and recovery hint; fixed broken [Display] intra-doc link to [std::fmt::Display]
- `src/stt/mod.rs` — Added /// doc comment on pub result_rx: Receiver<SttResult> field (line 54) — missed by T01/T02, caught by T03 audit pass
