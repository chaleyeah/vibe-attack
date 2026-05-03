# T02 Release Workflow Failure — v1.0.1-test

**Date:** 2026-05-03T21:38Z  
**Release run ID:** 25291491553  
**CI run ID:** 25291491559 (passed — all 3 jobs green)  
**Tag:** v1.0.1-test (still live on origin; NOT deleted per T02 protocol)

## Failures

### 1. Build RPM package — exit code 1

**Failing step:** `Build RPM package`  
**Error:**
```
error: line 2: Illegal char '-' (0x2d) in: Version:        1.0.1-test
```

**Root cause:** RPM spec `Version:` field does not permit hyphens. The `${GITHUB_REF_NAME#v}` substitution yields `1.0.1-test`, which is invalid for RPM. RPM reserves `-` as a separator between name, version, and release (`NAME-VERSION-RELEASE`). The `-test` suffix needs to move to the `Release:` field, or the tag version scheme must change to use `~` (tilde) or a period separator instead of a hyphen (e.g. `v1.0.1.test` or `v1.0.1rc1`).

**Fix needed:** Either:
- (A) In `packaging/vibe-attack.spec`, replace `Version: %{upstream_version}` with a sed transform that replaces `-` with `~` or `.` before inserting into the RPM Version field (RPM Release field allows `%{dist}` suffix, so `Release: 1.test%{?dist}` would also work if we split the pre-release suffix out), OR
- (B) Change the tag scheme away from `v1.0.1-test` to `v1.0.1.test` or `v1.0.1rc1` (but `-test` was chosen for Debian safety — this conflicts with RPM constraints), OR  
- (C) In `release.yml`, compute an RPM-safe version by stripping or replacing `-` in the version string before calling rpmbuild.

**Recommended fix:** In `release.yml`, extract an RPM-safe version variable: `RPM_VERSION=$(echo "${GITHUB_REF_NAME#v}" | tr '-' '~')` and pass it to the spec via `--define 'upstream_version ...'`. RPM supports `~` in version strings (it sorts lower than any character, appropriate for pre-release).

---

### 2. Build Debian package — exit code 3

**Failing step:** `Build .deb package`  
**Error:**
```
dpkg-checkbuilddeps: error: Unmet build dependencies: cargo rustc
dpkg-buildpackage: warning: build dependencies/conflicts unsatisfied; aborting
```

**Root cause:** `dpkg-buildpackage` enforces `Build-Depends` from `packaging/debian/control`. It lists `cargo` and `rustc` as build dependencies, but the CI workflow installs Rust via `rustup` (not the system package manager), so dpkg's dependency checker reports them as unmet.

**Fix needed:** Add `-d` flag to `dpkg-buildpackage` to skip build-dependency checking: `dpkg-buildpackage -uc -us -b -d`. The workflow already installs Rust via the `Install Rust toolchain` step and caches it — the `-d` flag tells dpkg to trust the caller rather than check apt-installed packages.

Alternatively, remove `cargo` and `rustc` from the `Build-Depends` field in `packaging/debian/control` (replace with just `debhelper-compat (= 13)` and other non-Rust deps), since the workflow manages Rust installation separately.

---

## Status

- CI workflow: **PASSED** (Test ✅, Clippy ✅, Validate AUR PKGBUILD ✅)  
- Release workflow: **FAILED** (RPM ❌, Debian ❌, AppImage still in progress when run failed)  
- Tag `v1.0.1-test`: **alive on origin** — required for T03 inspection; T02 protocol prohibits deletion  
- This slice needs replanning to fix the RPM version character constraint and Debian build-depends bypass  
