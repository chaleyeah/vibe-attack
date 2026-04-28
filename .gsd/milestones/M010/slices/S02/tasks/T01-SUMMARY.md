---
id: T01
parent: S02
milestone: M010
key_files:
  - packaging/appimage/vibe-attack.desktop
  - tests/ui_distribution.rs
key_decisions:
  - Assert Exec=vibe-attack-config exactly (not just Exec= substring) to prevent silent regression
duration: 
verification_result: passed
completed_at: 2026-04-28T03:58:12.070Z
blocker_discovered: false
---

# T01: Corrected AppImage .desktop Exec target to vibe-attack-config and tightened the regression test to assert the exact binary name

**Corrected AppImage .desktop Exec target to vibe-attack-config and tightened the regression test to assert the exact binary name**

## What Happened

The .desktop file at `packaging/appimage/vibe-attack.desktop` had `Exec=vibe-attack` but the only built binary is `vibe-attack-config`. This would cause AppImage launch to fail on every distro. The fix was a one-line change to `Exec=vibe-attack-config`.

The existing `desktop_file_exists_and_has_required_keys` test in `tests/ui_distribution.rs` only asserted `contents.contains("Exec=")`, which passes even with the wrong binary name. The assertion was replaced with an exact-value check for `Exec=vibe-attack-config`, including a diagnostic message that prints the actual Exec line found when the assertion fails, so future regressions produce immediately actionable output.

## Verification

Ran `cargo test --test ui_distribution -- --test-threads=1 desktop_file` — 1 test passed (desktop_file_exists_and_has_required_keys). Confirmed `grep -q '^Exec=vibe-attack-config$' packaging/appimage/vibe-attack.desktop` exits 0.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test ui_distribution -- --test-threads=1 desktop_file` | 0 | ✅ pass | 760ms |
| 2 | `grep -q '^Exec=vibe-attack-config$' packaging/appimage/vibe-attack.desktop` | 0 | ✅ pass | 5ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `packaging/appimage/vibe-attack.desktop`
- `tests/ui_distribution.rs`
