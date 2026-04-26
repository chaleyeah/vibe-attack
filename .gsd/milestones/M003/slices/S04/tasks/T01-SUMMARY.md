---
id: T01
parent: S04
milestone: M003
key_files:
  - src/ui/config_app.rs
key_decisions:
  - (none)
duration: 
verification_result: passed
completed_at: 2026-04-26T00:21:56.911Z
blocker_discovered: false
---

# T01: Added load_profiles() to config_app.rs — reads *.yaml stems from XDG profiles dir, sorted, with tracing::info count log

**Added load_profiles() to config_app.rs — reads *.yaml stems from XDG profiles dir, sorted, with tracing::info count log**

## What Happened

Added pub fn load_profiles() -> Vec<String> to config_app.rs using xdg::BaseDirectories::with_prefix(\"vibe-attack\").get_config_home().join(\"profiles\"). Reads all .yaml files, extracts file stems, sorts, logs count. Returns empty vec gracefully when dir doesn't exist. Also added mic_no_device bool to ConfigApp struct for S04/T02 to use.

## Verification

cargo check --lib clean; function logic verified by inspection against XDG hermetic pattern

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check --lib` | 0 | pass | 700ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/ui/config_app.rs`
