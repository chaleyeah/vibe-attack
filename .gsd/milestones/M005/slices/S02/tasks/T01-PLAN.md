---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Complete build.sh

Both binaries, SVG→PNG, libsherpa-onnx-c-api.so bundling, AppRun LD_LIBRARY_PATH

## Inputs

- `assets/vibe-attack.svg`

## Expected Output

- `Updated packaging/appimage/build.sh`

## Verification

build.sh exits 0 on machine with cargo + rsvg-convert
