---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Release build, test suite, screenshot gallery

cargo build --release --features gui, run full test suite with --test-threads=1, capture screenshots to ui/screenshots/

## Inputs

- `All prior slices`

## Expected Output

- `vibe-attack and vibe-attack-config release binaries`
- `0 test failures`
- `ui/screenshots/ contains reference screenshots`

## Verification

Release build succeeds, 0 test failures, screenshots present
