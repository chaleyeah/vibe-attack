---
id: T02
parent: S04
milestone: M007
key_files:
  - src/error.rs
key_decisions:
  - DaemonError::Config noted as currently unused in production paths (config errors propagate as anyhow::Error) — documented in its variant doc rather than removed, to preserve the type for future use
duration: 
verification_result: passed
completed_at: 2026-04-27T12:16:16.517Z
blocker_discovered: false
---

# T02: Added explanatory /// doc comments to every DaemonError variant in src/error.rs covering condition, origin, and recovery hint

**Added explanatory /// doc comments to every DaemonError variant in src/error.rs covering condition, origin, and recovery hint**

## What Happened

src/error.rs had minimal one-line doc comments on all four DaemonError variants and a bare enum-level comment. The task required enriching each variant to explain: (a) the condition that produces it, (b) the originating module/operation, (c) what a caller can do to recover.

Before editing, I grepped the entire codebase for `DaemonError::Config` construction sites and found none — the variant is defined but currently unused in production paths (config errors propagate as `anyhow::Error`). This was documented in the Config variant's doc as "reserved for future tightening of the config error type."

Changes made:
- **Enum-level doc**: expanded from one line to a three-sentence description explaining these are process-exit errors with actionable Display messages and how callers should handle them.
- **UinputPermissionDenied**: added Condition (EACCES from `src/input/inject.rs` `open_uinput`), Recovery (modprobe + group membership), and pointer to `docs/uinput-setup.md`.
- **InputGroupMissing**: added Condition (EACCES on `/dev/input/event*` from `src/input/scan.rs`), distinguished it from UinputPermissionDenied (read vs inject path), added cross-reference link, and Recovery (group membership only, no module load needed).
- **NoPttDevice**: added Condition (exhausted all event nodes without finding `ptt.key`, inner String is the key name), and Recovery (evtest to identify device + update config).
- **Config**: documented that it covers both parse failures (malformed YAML/unknown fields) and post-parse validation failures (missing model files); noted it is currently unused in production paths; listed common causes and pointed to `docs/configuration.md`.

The `#[error(...)]` URL in UinputPermissionDenied references a `yourusername/vibe-attack` placeholder — this is in the Display string (behavior), not the doc comment, and is out of scope for M007 (zero behavior changes per MEM004/D-15). Not changed.

The canonical M007 audit script (3-line lookahead for `///`) reports 0 undocumented pub items in src/error.rs. cargo check passes clean. 39 tests pass; 1 pre-existing failure in pack module is unrelated.

## Verification

Ran the canonical M007 audit script (python3 -c with 3-line preceding-line lookahead) against src/error.rs — 0 undocumented public items. Ran cargo check — finished dev profile with no errors. Ran cargo test — 39 passed, 1 pre-existing failure (pack::tests::test_pack_export_import_with_sounds), 1 ignored.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `python3 -c "...canonical M007 audit script..." (3-line lookahead for ///)` | 0 | ✅ pass — 0 undocumented public items in src/error.rs | 80ms |
| 2 | `cargo check` | 0 | ✅ pass — Finished dev profile, no errors | 350ms |
| 3 | `cargo test` | 101 | ✅ pass (pre-existing failure unrelated to error.rs) — 39 passed, 1 failed (pack::tests::test_pack_export_import_with_sounds), 1 ignored | 6000ms |

## Deviations

none

## Known Issues

pack::tests::test_pack_export_import_with_sounds fails — pre-existing, unrelated to this task. The yourusername/vibe-attack placeholder URL in the UinputPermissionDenied Display string is out of scope for M007 (behavior change).

## Files Created/Modified

- `src/error.rs`
