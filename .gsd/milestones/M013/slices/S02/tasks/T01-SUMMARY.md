---
id: T01
parent: S02
milestone: M013
key_files:
  - .github/workflows/ci.yml
key_decisions:
  - Avoided --verifysource to prevent network fetches for unreleased commits
  - No Arch container needed — bash field/function checks are distro-agnostic
duration: 
verification_result: passed
completed_at: 2026-05-03T20:46:33.217Z
blocker_discovered: false
---

# T01: Added validate-pkgbuild job to ci.yml; checks syntax and required PKGBUILD fields on every tag push

**Added validate-pkgbuild job to ci.yml; checks syntax and required PKGBUILD fields on every tag push**

## What Happened

Added a validate-pkgbuild job to ci.yml running on ubuntu-22.04 (no Arch container needed). The job bash -n checks syntax, then sources the PKGBUILD in a subshell to verify pkgname, pkgver, pkgrel, arch, and license are non-empty, and that build() and package() functions are defined. Skipped --verifysource since that would attempt to download the release tarball from GitHub, which won't exist for unreleased commits.

## Verification

bash -n packaging/PKGBUILD exits 0 locally; job added to ci.yml without Arch container dependency.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `bash -n packaging/PKGBUILD && echo OK` | 0 | PKGBUILD syntax valid | 80ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `.github/workflows/ci.yml`
