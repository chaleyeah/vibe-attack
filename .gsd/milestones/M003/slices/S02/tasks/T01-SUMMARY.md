---
id: T01
parent: S02
milestone: M003
key_files:
  - src/ui/probe.rs
  - src/ui/mod.rs
key_decisions:
  - xdg crate get_data_home() returns None when HOME is unset; fallback to $HOME/.local/share handled in xdg_data_model_path()
  - O_NONBLOCK not needed for /dev/uinput open — standard read+write flags are sufficient and avoid the libc dependency
duration: 
verification_result: passed
completed_at: 2026-04-26T00:15:16.359Z
blocker_discovered: false
---

# T01: Created src/ui/probe.rs with four check functions and probe::run() returning real FirstRunState

**Created src/ui/probe.rs with four check functions and probe::run() returning real FirstRunState**

## What Happened

Created probe.rs with check_config(), check_model(), check_uinput(), check_ptt() and pub fn run(). Uses xdg::BaseDirectories::with_prefix("vibe-attack") for config path; resolves XDG_DATA_HOME manually (xdg crate's with_prefix appends prefix to get_data_home()). uinput check opens /dev/uinput with read+write flags. PTT check scans config lines for 'key: KEY_*' pattern. Each failed check emits tracing::warn with check name and reason. probe::run() emits tracing::info with resolved config path on entry.

## Verification

cargo check --lib exits 0 with no errors in probe.rs source

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check --lib` | 0 | pass — no errors in probe module | 730ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/ui/probe.rs`
- `src/ui/mod.rs`
