---
id: T04
parent: S01
milestone: M003
key_files:
  - scripts/setup.sh
key_decisions:
  - Used 'input' group (not 'uinput') per docs/uinput-setup.md which documents the systemd v258+ breakage
duration: 
verification_result: passed
completed_at: 2026-04-26T00:10:55.748Z
blocker_discovered: false
---

# T04: Implemented step_setup_uinput: modprobe check, persist-on-boot option, input group check with systemd v258+ note, dry-run support

**Implemented step_setup_uinput: modprobe check, persist-on-boot option, input group check with systemd v258+ note, dry-run support**

## What Happened

Checks /dev/uinput existence before modprobe; offers to persist via /etc/modules-load.d/uinput.conf. Checks group membership via getent. Includes warning about 'input' vs 'uinput' for systemd v258+/CachyOS. Sets needs_relogin flag and prints logout reminder when group was just added. All sudo commands route through run_cmd for dry-run support and ask_confirm for interactive mode.

## Verification

On dev machine where /dev/uinput exists and user is in input group: both checks report already-satisfied, no sudo commands run

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `bash scripts/setup.sh --step=setup_uinput (dev machine with uinput+group)` | 0 | pass — both checks skipped as already satisfied | 40ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `scripts/setup.sh`
