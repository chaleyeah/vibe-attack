---
id: T01
parent: S03
milestone: M014
key_files:
  - src/pipeline/coordinator.rs
key_decisions:
  - (none)
duration: 
verification_result: passed
completed_at: 2026-06-07T21:12:15.698Z
blocker_discovered: false
---

# T01: Auto-build Whisper initial_prompt from active pack phrases when not explicitly configured

**Auto-build Whisper initial_prompt from active pack phrases when not explicitly configured**

## What Happened

Added effective_initial_prompt construction in coordinator.rs before SttService::new. Uses config.stt.initial_prompt if explicitly set (allows power-user override), otherwise joins all non-empty macro phrase strings as a comma-separated list. The active pack's macros are already loaded into config.macros at startup via ProfileManager, so all 75 HD2 stratagem phrases are included. Logs phrase_count at INFO level so the user can confirm prompting is working. All tests pass.

## Verification

cargo test — all tests pass. Checked that config.macros is populated from active pack before spawn_pipeline is called in main.rs.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test` | 0 | pass | 8000ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/pipeline/coordinator.rs`
