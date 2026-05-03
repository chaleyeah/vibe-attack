---
id: T01
parent: S01
milestone: M013
key_files:
  - .github/workflows/release.yml
key_decisions:
  - Used sed rewrite of spec into ~/rpmbuild/SPECS/ rather than modifying the source file, keeping packaging/vibe-attack.spec clean for local use
duration: 
verification_result: mixed
completed_at: 2026-05-03T20:45:58.535Z
blocker_discovered: false
---

# T01: Fixed RPM tarball prefix to derive version from git tag via GITHUB_REF_NAME

**Fixed RPM tarball prefix to derive version from git tag via GITHUB_REF_NAME**

## What Happened

The create source tarball step in build-rpm used a hardcoded vibe-attack-1.0.0/ prefix and SOURCES filename. Changed to TAG=${GITHUB_REF_NAME#v} so both the tarball prefix and the SOURCES path match the pushed tag. Also replaced the cp packaging/vibe-attack.spec step with a sed rewrite that stamps the spec's Version: field with the tag before passing it to rpmbuild.

## Verification

Diff inspection confirms no literal 1.0.0 in the RPM tarball step; sed pattern anchored with ^ to avoid false replacements on URLs in the spec.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `grep '1\.0\.0' .github/workflows/release.yml` | 1 | No hardcoded 1.0.0 in release.yml | 50ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `.github/workflows/release.yml`
