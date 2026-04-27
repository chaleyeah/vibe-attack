---
id: T03
parent: S05
milestone: M007
key_files:
  - docs/configuration.md
key_decisions:
  - Corrected macro.name description from 'phrase Whisper must recognise' to 'unique identifier used in logs and as the flag namespace' — the doc conflated name with phrase
  - Documented stt.confidence_threshold with its actual default (0.80) from the default_stt_confidence_threshold() function
  - Added sound field with rodio audio file note matching the MacroConfig doc comment
duration: 
verification_result: passed
completed_at: 2026-04-27T12:28:09.781Z
blocker_discovered: false
---

# T03: Updated docs/configuration.md to add three previously undocumented fields: stt.confidence_threshold, macro phrase/if_flag/set_flag/sound, and keys[].gap_ms per-key override

**Updated docs/configuration.md to add three previously undocumented fields: stt.confidence_threshold, macro phrase/if_flag/set_flag/sound, and keys[].gap_ms per-key override**

## What Happened

A field-by-field audit compared every `pub` field in `src/config.rs` against `docs/configuration.md`. Three gaps were found:

1. **`stt.confidence_threshold`** (default `0.80`) — present in `SttConfig` since the whisper integration but absent from the stt section. Added to both the YAML example block and the field table.

2. **`MacroConfig` optional fields** (`phrase`, `if_flag`, `set_flag`, `sound`) — the doc only showed `name` and `keys`. All four optional fields were documented in the YAML example (with comments) and in the table. The table description for `name` was also corrected from the misleading "phrase Whisper must recognise" to the accurate "unique identifier used in logs and as the flag namespace", since `name` is the identifier and `phrase` is the trigger.

3. **`KeyAction.gap_ms`** — the per-key gap override was in the code alongside `dwell_ms` but only `dwell_ms` appeared in the table. Added `keys[].gap_ms` row.

All 35 `pub` fields across all config structs (Config, AudioConfig, PttConfig, TimingConfig, PipelineConfig, VadConfig, SttConfig, WakeConfig, MacroConfig, KeyAction) are now documented with accurate names, types, and defaults. YAML example snippets were updated to match the actual Config schema.

## Verification

1. Field-by-field diff: extracted all `pub` fields from config.rs with grep and confirmed every one appears in the updated docs/configuration.md.
2. `cargo check` — exit 0, no errors.
3. `cargo test --lib -- --skip pack::tests::test_pack_export_import_with_sounds` — 39 passed, 0 failed. (The skipped test is a pre-existing fixture issue unrelated to this task.)

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `grep -n '^\s*pub ' src/config.rs | grep -v 'pub fn\|pub struct\|pub enum' | wc -l` | 0 | ✅ pass — 35 pub fields in config.rs, all now documented | 30ms |
| 2 | `cargo check` | 0 | ✅ pass | 380ms |
| 3 | `cargo test --lib -- --skip pack::tests::test_pack_export_import_with_sounds` | 0 | ✅ pass — 39 passed, 0 failed | 50ms |

## Deviations

none — task plan matched local reality exactly

## Known Issues

pack::tests::test_pack_export_import_with_sounds fails due to a missing pack.yaml fixture — pre-existing, unrelated to this task

## Files Created/Modified

- `docs/configuration.md`
