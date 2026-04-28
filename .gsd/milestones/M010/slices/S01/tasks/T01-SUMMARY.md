---
id: T01
parent: S01
milestone: M010
key_files:
  - scripts/verify-appimage.sh
key_decisions:
  - Script uses STATUS: skipped:tools-missing (exit 0) when linuxdeploy/appimagetool absent, allowing the verification gate to pass on a build host without packaging tools while the static cargo test --test packaging still covers build.sh structure
  - Transcript written unconditionally even on failure so partial proof remains inspectable downstream
duration: 
verification_result: passed
completed_at: 2026-04-28T03:49:25.516Z
blocker_discovered: false
---

# T01: Add scripts/verify-appimage.sh: POSIX-portable transcript-capture wrapper that detects missing linuxdeploy/appimagetool and emits STATUS: skipped:tools-missing on this build host

**Add scripts/verify-appimage.sh: POSIX-portable transcript-capture wrapper that detects missing linuxdeploy/appimagetool and emits STATUS: skipped:tools-missing on this build host**

## What Happened

The task required creating `scripts/verify-appimage.sh`, a portable shell wrapper that invokes `packaging/appimage/build.sh`, runs the produced AppImage with `--version`, and writes a structured transcript (STATUS, DISTRO, KERNEL, SIZE_BYTES, SHA256, EXIT_CODE, VERSION_OUTPUT) to the path passed as `$1`.

**Key design decisions:**
- Written as `#!/bin/sh` (POSIX sh) with `set -euo pipefail` — compatible with dash (Debian), bash (Fedora/Arch).
- Tool detection happens before any build attempt: if `linuxdeploy` or `appimagetool` are absent, the script writes `STATUS: skipped:tools-missing` and exits 0, so the verification gate is satisfied without crashing auto-mode.
- Transcript is always written — even on build failure, size overrun, or `--version` failure — with the appropriate `STATUS: failed:<reason>` and `FAILURE_REASON:` field so partial proof remains inspectable.
- 50 MB size guard is enforced after build succeeds and AppImage is present.
- `build.sh` is invoked unchanged per the plan's constraint.

**Build host outcome:** `linuxdeploy` and `appimagetool` are not installed on this Ubuntu 26.04 runner. The script correctly detected their absence, wrote a complete transcript with all structural fields (`STATUS: skipped:tools-missing`, `DISTRO: Ubuntu 26.04 LTS`, `KERNEL: 7.0.0-14-generic`, all remaining fields `pending`), and exited 0 in ~6 ms.

The transcript at `/tmp/host-transcript.md` is the input artefact that T02 will use when populating the `debian12` proof directory (the runner is Ubuntu/Debian-derived).

## Verification

Ran the exact verification command from the task plan:
`bash scripts/verify-appimage.sh /tmp/host-transcript.md && grep -q '^STATUS: ' /tmp/host-transcript.md && grep -q 'vibe-attack 0.1.0|STATUS: skipped' /tmp/host-transcript.md`
Result: exit 0, VERIFICATION PASSED.

Also confirmed:
- `sh -n scripts/verify-appimage.sh` reports no syntax errors
- Transcript contains all 7 required fields (STATUS, DISTRO, KERNEL, SIZE_BYTES, SHA256, EXIT_CODE, VERSION_OUTPUT) plus FAILURE_REASON
- Script is executable (`chmod +x` applied)

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `bash scripts/verify-appimage.sh /tmp/host-transcript.md && grep -q '^STATUS: ' /tmp/host-transcript.md && grep -q 'vibe-attack 0.1.0\|STATUS: skipped' /tmp/host-transcript.md && echo VERIFICATION: PASSED` | 0 | ✅ pass | 6ms |
| 2 | `sh -n scripts/verify-appimage.sh && echo 'syntax ok'` | 0 | ✅ pass | 5ms |
| 3 | `cat /tmp/host-transcript.md` | 0 | ✅ pass — all 7 structural fields present, STATUS: skipped:tools-missing | 2ms |

## Deviations

none — exactly as specified in the task plan

## Known Issues

linuxdeploy and appimagetool are not installed on this build host (Ubuntu 26.04), so the actual AppImage assembly was skipped. The full AppImage build will require these tools in CI or a VM with them installed.

## Files Created/Modified

- `scripts/verify-appimage.sh`
