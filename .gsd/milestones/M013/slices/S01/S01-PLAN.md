# S01: Fix release versioning bugs

**Goal:** Eliminate all hardcoded version strings from the CI release pipeline
**Demo:** Show release.yml diff: RPM tarball prefix uses GITHUB_REF_NAME, deb changelog stamped with tag version, dh-cargo removed

## Must-Haves

- No literal 1.0.0 version strings in release workflow steps; RPM tarball prefix and deb changelog both use GITHUB_REF_NAME

## Proof Level

- This slice proves: code-review + diff inspection

## Integration Closure

Self-contained to .github/workflows/release.yml

## Verification

- None — workflow-only change

## Tasks

- [x] **T01: Fix RPM tarball prefix to use tag version** `est:15m`
  Replace hardcoded vibe-attack-1.0.0/ prefix with GITHUB_REF_NAME#v in the release.yml RPM source tarball step; rewrite spec Version field via sed
  - Files: `.github/workflows/release.yml`
  - Verify: git diff shows no 1.0.0 in the RPM tarball step

- [x] **T02: Fix deb changelog version stamping** `est:15m`
  Prepend a new changelog entry with tag version before dpkg-buildpackage runs; remove unused dh-cargo build-dep
  - Files: `.github/workflows/release.yml`
  - Verify: Stamp changelog step present in build-deb job; dh-cargo absent from apt-get install list

## Files Likely Touched

- .github/workflows/release.yml
