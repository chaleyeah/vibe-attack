---
id: T01
parent: S01
milestone: M003
key_files:
  - scripts/setup.sh
key_decisions:
  - --step flag added for wizard integration (S03 panels can shell out per step)
  - --dry-run implemented via run_cmd wrapper that prints instead of executing
duration: 
verification_result: passed
completed_at: 2026-04-26T00:10:34.629Z
blocker_discovered: false
---

# T01: Scaffolded scripts/setup.sh with bash strict mode, --help/--yes/--dry-run/--step flags, step registry, run_step/run_cmd helpers, and colour output

**Scaffolded scripts/setup.sh with bash strict mode, --help/--yes/--dry-run/--step flags, step registry, run_step/run_cmd helpers, and colour output**

## What Happened

Created scripts/setup.sh from scratch. Bash strict mode (set -euo pipefail). CLI parsing handles --help, -y/--yes, --dry-run, --step=NAME, and unknown-option guard. Colour helpers degrade gracefully when stdout is not a TTY. Step registry is an ordered array; main() iterates it or dispatches a single step via --step.

## Verification

bash -n scripts/setup.sh exits 0; --help prints usage and step list

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `bash -n scripts/setup.sh && echo ok` | 0 | pass | 20ms |
| 2 | `bash scripts/setup.sh --help` | 0 | pass | 30ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `scripts/setup.sh`
