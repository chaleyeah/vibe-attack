---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T02: Fix deb changelog version stamping

Prepend a new changelog entry with tag version before dpkg-buildpackage runs; remove unused dh-cargo build-dep

## Inputs

- `.github/workflows/release.yml`
- `packaging/debian/changelog`

## Expected Output

- `Changelog prepend step using TAG=${GITHUB_REF_NAME#v}`
- `dh-cargo removed from Install system dependencies step`

## Verification

Stamp changelog step present in build-deb job; dh-cargo absent from apt-get install list
