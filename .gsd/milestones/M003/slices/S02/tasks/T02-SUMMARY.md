---
id: T02
parent: S02
milestone: M003
key_files:
  - src/ui/probe.rs
key_decisions:
  - serial_test already in dev-dependencies from prior work — no new dep needed
  - with_xdg helper documents the single-threaded invariant in a comment per codebase convention
duration: 
verification_result: passed
completed_at: 2026-04-26T00:15:22.887Z
blocker_discovered: false
---

# T02: Added 8 hermetic unit tests with #[serial] annotation for env-var isolation; all 8 pass

**Added 8 hermetic unit tests with #[serial] annotation for env-var isolation; all 8 pass**

## What Happened

Wrote 8 tests in probe.rs #[cfg(test)]: config_missing/present, model_missing/empty/present, ptt_missing_config/no_key/key_present. All use with_xdg() helper that sets XDG_CONFIG_HOME and XDG_DATA_HOME to tempdirs and removes them on exit. Added #[serial] from serial_test (already in dev-dependencies) to prevent env-var races. First run without serial had 3 failures due to parallel test pollution — adding serial fixed all 8.

## Verification

cargo test --lib ui::probe exits 0 with 8/8 passing

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --lib ui::probe` | 0 | pass — 8 passed, 0 failed | 3600ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/ui/probe.rs`
