---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Create .github/workflows/release.yml

Workflow triggers on v* tags, builds AppImage via build.sh, uploads via softprops/action-gh-release

## Inputs

- `packaging/appimage/build.sh`

## Expected Output

- `.github/workflows/release.yml`

## Verification

YAML parses without error; workflow has correct trigger, rust-cache, and upload steps
