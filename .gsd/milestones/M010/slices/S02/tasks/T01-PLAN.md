---
estimated_steps: 2
estimated_files: 2
skills_used: []
---

# T01: Fix .desktop Exec target and add ui_distribution.rs assertion

The packaging/appimage/vibe-attack.desktop file currently has `Exec=vibe-attack`, but the only built binary is `vibe-attack-config`. AppImage launch will fail on every distro until this is corrected. Also tighten the existing `desktop_file_exists_and_has_required_keys` test in tests/ui_distribution.rs to specifically assert `Exec=vibe-attack-config` so this regression cannot recur silently.

This is a quick, blocking fix — every later task in this slice (and S03/S05/S06) depends on the AppImage actually launching the right binary. No --skip-wizard logic yet; that comes in T02.

## Inputs

- ``packaging/appimage/vibe-attack.desktop` — current Exec line is incorrect (`Exec=vibe-attack`)`
- ``tests/ui_distribution.rs` — existing test only checks `Exec=` substring, not the target binary name`

## Expected Output

- ``packaging/appimage/vibe-attack.desktop` — `Exec=vibe-attack-config` (target binary name corrected)`
- ``tests/ui_distribution.rs` — `desktop_file_exists_and_has_required_keys` (or new sibling test) asserts `Exec=vibe-attack-config` exactly`

## Verification

cargo test --test ui_distribution -- --test-threads=1 desktop_file && grep -q '^Exec=vibe-attack-config$' packaging/appimage/vibe-attack.desktop
