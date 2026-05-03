---
id: T02
parent: S01
milestone: M013
key_files:
  - .github/workflows/release.yml
key_decisions:
  - Prepend approach (not replace) keeps existing changelog history intact for local dpkg-buildpackage users
duration: 
verification_result: mixed
completed_at: 2026-05-03T20:46:05.197Z
blocker_discovered: false
---

# T02: Fixed deb changelog version stamping and removed unused dh-cargo build-dep

**Fixed deb changelog version stamping and removed unused dh-cargo build-dep**

## What Happened

dpkg-buildpackage derives the .deb version from debian/changelog, which was hardcoded at 1.0.0. Added a Stamp changelog step that prepends a new entry using TAG=${GITHUB_REF_NAME#v} before the build runs, ensuring the output .deb filename and package metadata match the git tag. Also removed dh-cargo from the apt-get install list — the debian/rules file overrides all dh steps with custom cargo invocations, so dh-cargo was never used.

## Verification

Stamp changelog step present in build-deb job; dh-cargo absent from dependency list in release.yml.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `grep 'dh-cargo' .github/workflows/release.yml` | 1 | dh-cargo removed | 50ms |
| 2 | `grep -A5 'Stamp changelog' .github/workflows/release.yml` | 0 | Changelog stamp step present using GITHUB_REF_NAME | 50ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `.github/workflows/release.yml`
