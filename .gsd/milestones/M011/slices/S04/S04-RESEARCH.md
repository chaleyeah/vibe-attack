# S04 — Version bump + release CI (.deb / .rpm jobs) — Research

**Date:** 2026-04-28

## Summary

The project has well-structured packaging definitions for Debian (.deb), Fedora/RPM (.rpm), and Arch (PKGBUILD), each already specifying version `0.1.0` across `Cargo.toml`, `packaging/vibe-attack.spec`, and `packaging/PKGBUILD`. The current release.yml workflow builds and uploads AppImage + source tarball, but lacks .deb and .rpm build jobs. Slice S04 requires:

1. **Version bump**: Update `Cargo.toml`, `vibe-attack.spec`, `PKGBUILD`, and `CHANGELOG.md` to version `1.0.0`.
2. **Release CI jobs**: Add `.deb` and `.rpm` build jobs to `.github/workflows/release.yml` that run on ubuntu-22.04 with cross-distro tooling (dpkg-deb for .deb, rpmbuild for .rpm).
3. **Artifact upload**: Extend the release artifact upload block to include .deb and .rpm files.

The blocking challenge is that ubuntu-22.04 does not pre-install rpmbuild; we must install `rpm` package or use a specialized action. Debian packaging via dpkg-deb or cargo-deb is more straightforward.

## Recommendation

**Build order:** (1) Version bump all files → (2) Proof .deb build locally or in CI → (3) Proof .rpm build (likely via action or rpm toolchain install) → (4) Integrate both into release.yml → (5) Tag and test on real tag push.

**Approach:** Use cargo-deb crate for .deb build (simpler, fewer CI dependencies) and rpm crate or rpmbuild for .rpm (more complex; rpmbuild requires the rpm package which is ~200MB but readily available via apt). Reference MEM086 (explicit newline-separated globs + fail_on_unmatched_files) and MEM089 (sherpa-onnx cache parity between ci.yml and release.yml).

## Implementation Landscape

### Key Files

- **`Cargo.toml`** (line 3) — Currently `version = "0.1.0"`. Bump to `1.0.0`.
- **`packaging/vibe-attack.spec`** (line 2) — Currently `Version: 0.1.0`. Bump to `1.0.0`. Also update Release field and %changelog entry date.
- **`packaging/PKGBUILD`** (line 3) — Currently `pkgver=0.1.0`. Bump to `1.0.0`. Per MEM093, sha256sums remain `'SKIP'` until release time (pinned via docs/distribution-proofs/aur/README.md workflow).
- **`packaging/debian/control`** (line 1) — Source stanza does not embed version; version lives in Debian changelog only.
- **`packaging/debian/changelog`** (line 1) — Currently `0.1.0-1`. Bump to `1.0.0-1` with today's timestamp.
- **`CHANGELOG.md`** (lines 8–46) — Unreleased section exists. Create a `[1.0.0]` block with today's date and move "Unreleased" content or create dated summary.
- **`.github/workflows/release.yml`** (lines 12–87) — Currently has build-appimage job only. Add build-deb and build-rpm jobs (parallel, same ubuntu-22.04 runner).

### Build Order

1. **Version string updates (all four files)** — Atomic, low risk. Verify via grep/cat.
2. **CHANGELOG.md dated entry** — Format check; ensure [1.0.0] block is parseable.
3. **Proof .deb build locally or in mock CI** — Validate debian/rules, debian/control, and cargo build flow. Cargo-deb is simpler; dpkg-deb requires manual control file + binary layout.
4. **Proof .rpm build locally or in mock CI** — More complex: rpmbuild requires `rpm` package (~200MB on ubuntu-22.04). Validate vibe-attack.spec, cargo build, and install steps.
5. **Integrate both into release.yml** — Add two new jobs (build-deb, build-rpm) with sherpa-onnx cache parity (MEM089). Use explicit globs + fail_on_unmatched_files in upload block (MEM086).
6. **Tag push test** — Create a real v1.0.0 tag, verify all three artifacts (AppImage, .deb, .rpm) upload to GitHub Releases.

### Verification Approach

**Local/pre-release:**
- Grep version strings: `grep -r "0\.1\.0" Cargo.toml packaging/ CHANGELOG.md` → all must be empty after bump.
- Cargo build `cargo build --release --locked` (already tested in ci.yml).
- Validate Debian build: `dpkg-buildpackage -uc -us` in repo root (or use cargo-deb if available).
- Validate RPM build: `rpmbuild -bb packaging/vibe-attack.spec` after setting up build tree (or mock rpmbuild environment).
- Verify artifacts exist: `ls -la *.deb *.rpm vibe-attack-*.AppImage` after each build.

**CI (release.yml):**
- Run a test tag push (e.g., v1.0.0-test) to confirm all jobs trigger.
- Verify artifact upload via softprops/action-gh-release@v2 with fail_on_unmatched_files: true (MEM086) catches missing .deb or .rpm immediately.
- Download released assets and spot-check file headers (e.g., `file *.deb *.rpm vibe-attack-*.AppImage`).

**Post-release:**
- Install .deb on Debian 13 or Ubuntu 26.04: `sudo dpkg -i vibe-attack_1.0.0-1_amd64.deb` → verify binaries present.
- Install .rpm on Fedora 44: `sudo rpm -Uvh vibe-attack-1.0.0-1.x86_64.rpm` → verify binaries present.
- Run both binaries: `vibe-attack --help`, `vibe-attack-config --help` (matching proof transcript format in docs/distribution-proofs/).

## Constraints

- **ubuntu-22.04 RPM toolchain**: rpmbuild is not pre-installed; must apt-get install rpm (~200MB). Acceptable; many CI pipelines do this. Alternatively, use a containerized rpmbuild action, but adds complexity.
- **Cargo.toml version is canonical**: All other version strings (spec, PKGBUILD, debian/changelog) must match or CI/packaging workflows will mismatch.
- **AUR sha256sum SKIP placeholders** (MEM093): packaging/PKGBUILD must retain `sha256sums=('SKIP', 'SKIP')` during development and release; only pinned when pushed to AUR via the docs/distribution-proofs/aur/ workflow.
- **sherpa-onnx cache parity** (MEM089): Both ci.yml and release.yml must use identical cache key (`sherpa-onnx-1.12.39-linux-x64`) and path (`target/sherpa-onnx-prebuilt`). The new .deb and .rpm jobs must also cache to avoid re-downloading the prebuilt.
- **Artifact naming conventions**: AppImage → `vibe-attack-${TAG}-x86_64.AppImage`; .deb → `vibe-attack_1.0.0-1_amd64.deb` (Debian convention with underscores); .rpm → `vibe-attack-1.0.0-1.x86_64.rpm` (RPM convention with hyphens). Upload block must match all three with explicit globs.

## Common Pitfalls

- **Version drift**: Cargo.toml is the source of truth; spec/PKGBUILD/debian files must be kept in sync. Use a grep check in CI or a pre-commit hook to catch mismatches.
- **RPM build environment**: rpmbuild expects sources in ~/rpmbuild/SOURCES/, build specs in ~/rpmbuild/SPECS/, and output in ~/rpmbuild/RPMS/. The vibe-attack.spec uses `%prep %autosetup` which expects the tarball to auto-extract. Ensure the tarball path in Source0 matches what CI provides (likely `vibe-attack-1.0.0.tar.gz` fetched from GitHub releases or unpacked from git archive).
- **Missing clang at build time**: MEM092 notes that clang is required by bindgen/clang-sys (transitive of sherpa-onnx-sys). Debian has libclang-dev, RPM has clang-devel. Both spec and debian/control already declare this; verify it's in the install step.
- **onnxruntime runtime dep not bundled**: vibe-attack.spec and PKGBUILD declare onnxruntime as a runtime dependency. Native packages don't bundle the .so; AppImage does (via build.sh). The .deb and .rpm builds must succeed with `Requires: onnxruntime` (or Depends for deb); install tests should verify the runtime dep is satisfied before running binaries.
- **Artifact glob failures**: softprops/action-gh-release with fail_on_unmatched_files: true (MEM086) will error if any glob pattern matches zero files. If .deb or .rpm build job silently skips or fails, the release upload step will loudly fail. This is desirable (prevents broken releases), but requires both jobs to complete successfully first.
- **CHANGELOG.md format**: Keep-a-Changelog format expects `## [1.0.0]` with a release date or `## [Unreleased]` for development. Ensure the date format is consistent (YYYY-MM-DD is standard).

## Open Risks

- **rpmbuild cross-compilation**: If the build spec assumes certain Fedora/RHEL-specific paths or packages, a build on ubuntu-22.04 may fail or produce packages with wrong paths. MEM096 notes that Fedora uses `alsa-lib-devel` (not libasound2-dev). The spec already has the right name, but test on Fedora 44 to confirm the .rpm installs and runs.
- **Debian package conflict with system packages**: If a .deb lands on a system that has a conflicting older vibe-attack package or if dependencies are not satisfied, dpkg may fail. Test install on a clean Debian 13/Ubuntu 26.04 VM or container.
- **GitHub Actions job parallelism timeout**: If build-deb and build-rpm jobs run in parallel and each takes 20+ minutes (first sherpa-onnx build), the combined CI time may exceed timeouts. Mitigated by sherpa-onnx cache (MEM089), but if cache misses, verify job timeout is ≥60 min.
- **Release artifact size**: AppImage + .deb + .rpm + source tarball could exceed GitHub Releases' practical limits (each ~100–300 MB). Confirm total release size is acceptable and storage quota is sufficient.
