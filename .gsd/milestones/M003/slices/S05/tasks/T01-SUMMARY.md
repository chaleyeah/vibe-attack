---
id: T01
parent: S05
milestone: M003
key_files:
  - src/ui/first_run.rs
  - src/ui/config_app.rs
key_decisions:
  - (none)
duration: 
verification_result: passed
completed_at: 2026-04-26T00:23:50.501Z
blocker_discovered: false
---

# T01: Added 5 FirstRunState tests and 4 ConfigApp tests (load_profiles + log cap); all 17 ui:: tests pass

**Added 5 FirstRunState tests and 4 ConfigApp tests (load_profiles + log cap); all 17 ui:: tests pass**

## What Happened

Added #[cfg(test)] to first_run.rs: 5 tests covering all-false step ordering, all-true completion, partial step filter, first_incomplete_step dispatch, and only-ptt-missing case. Added #[cfg(test)] to config_app.rs: 4 tests covering load_profiles absent dir, sorted yaml stems, non-yaml file ignore, and add_log_line MAX_LOG_LINES cap. All use tempdir + XDG isolation with #[serial] where env vars are touched.

## Verification

cargo test --lib ui:: exits 0 with 17 tests passing

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --lib ui::` | 0 | pass — 17 passed, 0 failed | 1250ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/ui/first_run.rs`
- `src/ui/config_app.rs`
