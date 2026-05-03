---
id: T02
parent: S03
milestone: M013
key_files:
  - .gsd/milestones/M013/slices/S03/T02-FAILURE.md
key_decisions:
  - Tag v1.0.1-test is NOT deleted — left live on origin per T02 failure protocol so T03/next planner can inspect the run without needing to re-push
  - blocker_discovered: true — RPM hyphen prohibition and Debian dpkg-checkbuilddeps bypass are structural issues requiring release.yml changes before any re-run can succeed
duration: 
verification_result: mixed
completed_at: 2026-05-03T21:47:39.505Z
blocker_discovered: true
---

# T02: Pushed v1.0.1-test tag and triggered CI+Release workflows; CI passed but Release failed on RPM (hyphen in Version field) and Debian (dpkg build-dep check blocks rustup-installed Rust)

**Pushed v1.0.1-test tag and triggered CI+Release workflows; CI passed but Release failed on RPM (hyphen in Version field) and Debian (dpkg build-dep check blocks rustup-installed Rust)**

## What Happened

The annotated tag `v1.0.1-test` was created locally and pushed to `origin` (chaleyeah/vibe-attack). Both GitHub Actions workflows triggered as expected (CI run 25291491559, Release run 25291491553).

**CI workflow (PASSED):** All three jobs finished green — Test (2m24s), Clippy (1m16s), Validate AUR PKGBUILD (12s). The T01 fix (//! inner doc comments) caused no regressions; clippy passed on both default and gui feature sets.

**Release workflow (FAILED — 2 jobs):**

1. **Build RPM package** (exit code 1): `rpmbuild` rejected the spec with `error: line 2: Illegal char '-' (0x2d) in: Version: 1.0.1-test`. RPM's `Version:` field does not permit hyphens — the character is reserved as the name-version-release separator. The `${GITHUB_REF_NAME#v}` substitution yields `1.0.1-test`, which is structurally invalid for RPM. The tag scheme (`v1.0.1-test`) was chosen because `-test` is safe for Debian; this conflicts with RPM's stricter character set. Fix: In `release.yml`, compute `RPM_VERSION=$(echo "${GITHUB_REF_NAME#v}" | tr '-' '~')` and pass it via `--define 'upstream_version ...'` to rpmbuild. RPM supports `~` as a pre-release comparator.

2. **Build Debian package** (exit code 3): `dpkg-buildpackage -uc -us -b` calls `dpkg-checkbuilddeps` before building, which checks that `Build-Depends` packages are installed as system packages. The workflow installs Rust via `rustup` (not apt), so `cargo` and `rustc` appear absent to dpkg. Fix: Add `-d` flag to `dpkg-buildpackage` call in `release.yml` to skip the build-dep check (the workflow already manages Rust installation separately).

These are not minor deviations — they are structural incompatibilities that require changes to `release.yml` (and possibly `packaging/vibe-attack.spec`) before the Release workflow can succeed. The slice plan assumed `-test` was a universally safe version suffix, which is false for RPM.

The tag `v1.0.1-test` has NOT been deleted per the task protocol. Failure details written to `.gsd/milestones/M013/slices/S03/T02-FAILURE.md`.

## Verification

CI run verified green via `gh run watch 25291491559 --exit-status` — all 3 jobs completed successfully. Release run verified failed via `gh run watch 25291491553 --exit-status` (exit code 1) and `gh run view 25291491553 --log-failed` which surfaced the exact error messages for both failing jobs. Tag existence on origin confirmed via `git push origin v1.0.1-test` output and initial `git ls-remote --tags origin v1.0.1-test` (empty before push).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `git push origin v1.0.1-test` | 0 | ✅ pass — tag created on origin | 3200ms |
| 2 | `gh run watch 25291491559 --repo chaleyeah/vibe-attack --exit-status (CI)` | 0 | ✅ pass — all 3 CI jobs green (Test, Clippy, Validate AUR PKGBUILD) | 165000ms |
| 3 | `gh run watch 25291491553 --repo chaleyeah/vibe-attack --exit-status (Release)` | 1 | ❌ fail — Build RPM package and Build Debian package failed | 180000ms |
| 4 | `gh run view 25291491553 --repo chaleyeah/vibe-attack --log-failed` | 0 | ❌ RPM: 'Illegal char '-' in Version: 1.0.1-test'; Debian: 'Unmet build dependencies: cargo rustc' | 4000ms |

## Deviations

Task plan step 6 (failure path) executed instead of step 5 (success verification). Tag intentionally left alive per protocol. T02-FAILURE.md written instead of T02-RESULT.md.

## Known Issues

RPM fix: compute RPM_VERSION via `tr '-' '~'` in release.yml and pass via --define. Debian fix: add `-d` flag to dpkg-buildpackage call. Both changes are in release.yml; the spec Version field itself may not need to change if the workflow computes the RPM-safe version before calling rpmbuild.

## Files Created/Modified

- `.gsd/milestones/M013/slices/S03/T02-FAILURE.md`
