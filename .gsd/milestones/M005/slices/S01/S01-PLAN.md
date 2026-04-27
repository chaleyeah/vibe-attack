# S01: Icon and shared assets

**Goal:** Create the vibe-attack SVG icon and wire it into .desktop and build.sh
**Demo:** assets/vibe-attack.svg exists; build.sh copies it into AppDir without errors

## Must-Haves

- Not provided.

## Proof Level

- This slice proves: Not provided.

## Integration Closure

Not provided.

## Verification

- Not provided.

## Tasks

- [x] **T01: Create assets/vibe-attack.svg** `est:15m`
  Design and write the SVG icon file
  - Files: `assets/vibe-attack.svg`
  - Verify: File exists with valid SVG markup

- [x] **T02: Update build.sh for SVG→PNG conversion and both binaries** `est:15m`
  Add SVG-to-PNG conversion step and vibe-attack-config binary copy
  - Files: `packaging/appimage/build.sh`
  - Verify: build.sh references assets/vibe-attack.svg and copies vibe-attack-config

- [x] **T03: Update .desktop file** `est:5m`
  Add StartupWMClass to .desktop file
  - Files: `packaging/appimage/vibe-attack.desktop`
  - Verify: .desktop file has StartupWMClass=vibe-attack

## Files Likely Touched

- assets/vibe-attack.svg
- packaging/appimage/build.sh
- packaging/appimage/vibe-attack.desktop
