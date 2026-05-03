---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Fix RPM tarball prefix to use tag version

Replace hardcoded vibe-attack-1.0.0/ prefix with GITHUB_REF_NAME#v in the release.yml RPM source tarball step; rewrite spec Version field via sed

## Inputs

- `.github/workflows/release.yml`

## Expected Output

- `TAG=${GITHUB_REF_NAME#v} used in tarball prefix and SOURCES filename`
- `sed rewrites spec Version field from the tag`

## Verification

git diff shows no 1.0.0 in the RPM tarball step
