# S02: Add AUR PKGBUILD validation to CI

**Goal:** Gate every tag push on a PKGBUILD syntax and field-presence check
**Demo:** ci.yml validate-pkgbuild job runs and passes, showing all required fields and function definitions present

## Must-Haves

- validate-pkgbuild job in ci.yml exits 0 on current PKGBUILD; breaking the PKGBUILD causes exit non-zero

## Proof Level

- This slice proves: CI green + manual break-test

## Integration Closure

Standalone CI job, no coupling to build jobs

## Verification

- CI job result visible in GitHub Actions tab

## Tasks

- [x] **T01: Add validate-pkgbuild job to ci.yml** `est:20m`
  Add a new job that bash -n checks PKGBUILD syntax and sources it to verify required fields (pkgname, pkgver, pkgrel, arch, license) and function definitions (build, package) are present
  - Files: `.github/workflows/ci.yml`
  - Verify: ci.yml contains validate-pkgbuild job; bash -n packaging/PKGBUILD exits 0 locally

## Files Likely Touched

- .github/workflows/ci.yml
