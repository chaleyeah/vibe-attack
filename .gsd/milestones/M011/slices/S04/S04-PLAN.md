# S04: Version bump + release CI (.deb / .rpm jobs)

**Goal:** Bump the project to version 1.0.0 across every packaging manifest and extend the release workflow so a tag push produces AppImage + .deb + .rpm + source tarball + hdpack artifacts (instead of only AppImage + tarball + hdpack today). The slice closes the "buildable releases" gap before S05 publishes the actual GitHub Release.
**Demo:** `Cargo.toml`, `vibe-attack.spec`, and `PKGBUILD` read `1.0.0`; `CHANGELOG.md` has a dated `[1.0.0]` block; `release.yml` builds and uploads AppImage + .deb + .rpm + source tarball on a real test-tag push.

## Must-Haves

- Cargo.toml line 3 reads `version = "1.0.0"`.
- packaging/vibe-attack.spec line 2 reads `Version:        1.0.0` and the %changelog has an entry dated for today (2026-04-28) under `1.0.0-1`.
- packaging/PKGBUILD line 3 reads `pkgver=1.0.0`; sha256sums remain `'SKIP'` per MEM093.
- packaging/debian/changelog top entry reads `vibe-attack (1.0.0-1) unstable; urgency=medium` with a 2026-04-28 timestamp.
- CHANGELOG.md gains `## [1.0.0] - 2026-04-28` heading; existing `## [Unreleased]` content is moved under it (or `## [Unreleased]` left empty).
- .github/workflows/release.yml has two new jobs `build-deb` and `build-rpm`, both reusing the sherpa-onnx cache block from build-appimage (MEM089 parity).
- The `softprops/action-gh-release@v2` step in release.yml is updated so its `files:` block contains explicit globs for AppImage, source tarball, hdpack, .deb, and .rpm — `fail_on_unmatched_files: true` retained (MEM086).
- New unit tests in `tests/packaging.rs` assert the workflow contains `build-deb`, `build-rpm`, and the deb/rpm globs in the upload block — and `cargo test --test packaging` passes.</successCriteria>
- <parameter name="proofLevel">- This slice proves: integration (workflow definition + manifest version coherence; real tag-push is S05's job).
- Real runtime required: no (static workflow YAML and manifest checks; no actual CI run executed in this slice).
- Human/UAT required: no.

## Proof Level

- This slice proves: Not provided.

## Integration Closure

- Upstream surfaces consumed: existing `build-appimage` job in `.github/workflows/release.yml` (sherpa-onnx cache block, system-deps install pattern, softprops upload block); `packaging/vibe-attack.spec`, `packaging/debian/rules`, `packaging/debian/control`, `packaging/PKGBUILD` already define their respective build recipes.
- New wiring introduced in this slice: two new jobs (`build-deb`, `build-rpm`) added to `release.yml`; `.deb` and `.rpm` glob entries added to the existing upload step; new contract assertions in `tests/packaging.rs` covering those job names and globs.
- What remains before the milestone is truly usable end-to-end: pushing a real `v1.0.0` tag and verifying GitHub Releases serves AppImage + .deb + .rpm + tarball + hdpack — that is S05's responsibility per the roadmap and per MEM111.

## Verification

- Runtime signals: GitHub Actions job logs (build-deb / build-rpm) produce streamed output during a real tag push; not exercised in this slice.
- Inspection surfaces: `cargo test --test packaging` is the deterministic local check; release.yml job names appear in the GitHub Actions UI for any future tag.
- Failure visibility: `fail_on_unmatched_files: true` in the upload step makes a missing .deb/.rpm artifact fail the release upload step loudly (MEM086).
- Redaction constraints: none — no secrets in scope.

## Tasks

- [x] **T01: Bump version to 1.0.0 across Cargo.toml, spec, PKGBUILD, debian/changelog, and CHANGELOG.md** `est:20m`
  Atomic version bump across all packaging manifests. Today's date is 2026-04-28 — use that date for the spec %changelog entry, debian/changelog timestamp, and the CHANGELOG.md `## [1.0.0]` heading. The rule is: `grep -rn "0\.1\.0" Cargo.toml packaging/ CHANGELOG.md` must return zero hits afterward. PKGBUILD's `sha256sums=('SKIP', 'SKIP')` MUST remain unchanged per MEM093 — sha256sums are pinned only at AUR submission time, not at version bump time. Do not modify packaging/debian/control (it does not embed a version). For CHANGELOG.md, create a new `## [1.0.0] - 2026-04-28` block above the existing `## [Unreleased]`, and move the current Added/Fixed/Changed entries from Unreleased into the 1.0.0 block; leave `## [Unreleased]` as an empty header for future work.

Why: every other task in this slice and S05 depends on the manifests reading 1.0.0; otherwise release artifact filenames will be wrong (vibe-attack_0.1.0-1_amd64.deb vs the expected 1.0.0) and AppImage rename will pick up the wrong tag.
  - Files: `Cargo.toml`, `packaging/vibe-attack.spec`, `packaging/PKGBUILD`, `packaging/debian/changelog`, `CHANGELOG.md`
  - Verify: grep -rn "0\.1\.0" Cargo.toml packaging/ CHANGELOG.md | grep -v 'sherpa-onnx' | grep -v 'silero' ; test $? -ne 0 && grep -q '^version = "1.0.0"' Cargo.toml && grep -q '^Version:        1.0.0' packaging/vibe-attack.spec && grep -q '^pkgver=1.0.0' packaging/PKGBUILD && head -1 packaging/debian/changelog | grep -q '1.0.0-1' && grep -q '## \[1.0.0\] - 2026-04-28' CHANGELOG.md

- [x] **T02: Add build-deb and build-rpm jobs to release.yml and extend artifact upload globs** `est:1h30m`
  Add two new jobs to `.github/workflows/release.yml`, parallel to the existing `build-appimage` job. Both run on `ubuntu-22.04`, both reuse the exact sherpa-onnx cache block (path: `target/sherpa-onnx-prebuilt`, key: `sherpa-onnx-1.12.39-linux-x64`) per MEM089 — copy the block verbatim from `build-appimage`. Then extend the existing upload step (or migrate it into a small `release` job that depends on all three build jobs).

Approach for the .deb job (`build-deb`):
  1. Checkout, install Rust, rust-cache, sherpa cache + conditional rebuild (MEM089 parity).
  2. apt-get install: `libasound2-dev libclang-dev devscripts debhelper dh-cargo` (devscripts/debhelper provide `dpkg-buildpackage` and `dh`; existing `packaging/debian/rules` already uses dh and cargo build).
  3. Run `dpkg-buildpackage -uc -us -b` from repo root. This emits `vibe-attack_1.0.0-1_amd64.deb` into the parent directory (Debian convention) — move it back into the workflow workspace.
  4. `actions/upload-artifact@v4` with name `deb` so the final upload-release step can pull it.

Approach for the .rpm job (`build-rpm`):
  1. Checkout, install Rust, rust-cache, sherpa cache + conditional rebuild (MEM089 parity).
  2. apt-get install: `libasound2-dev libclang-dev rpm` (the `rpm` package on ubuntu-22.04 provides `rpmbuild`).
  3. Set up rpmbuild tree: `mkdir -p ~/rpmbuild/{SOURCES,SPECS,BUILD,RPMS,SRPMS}`. The .spec uses `%autosetup` which expects a tarball matching `Source0`. Create the source tarball locally with `git archive --format=tar.gz --prefix=vibe-attack-1.0.0/ HEAD -o ~/rpmbuild/SOURCES/vibe-attack-1.0.0.tar.gz`. Copy `packaging/vibe-attack.spec` to `~/rpmbuild/SPECS/`.
  4. Run `rpmbuild -bb ~/rpmbuild/SPECS/vibe-attack.spec`. This produces `vibe-attack-1.0.0-1.x86_64.rpm` in `~/rpmbuild/RPMS/x86_64/`. Copy it back to the workspace.
  5. `actions/upload-artifact@v4` with name `rpm`.

Then modify the existing upload-release step (currently inside `build-appimage`):
  - The cleanest refactor is to rename the upload step's job to a new `release` job that `needs: [build-appimage, build-deb, build-rpm]`. Use `actions/download-artifact@v4` to fetch all three artifact bundles, then run the existing `softprops/action-gh-release@v2` step with these explicit globs (newline-separated, fail_on_unmatched_files: true per MEM086):
      vibe-attack-*-x86_64.AppImage
      vibe-attack-*.tar.gz
      hd2-*.hdpack
      vibe-attack_*.deb
      vibe-attack-*.x86_64.rpm
  - The AppImage build job must `actions/upload-artifact@v4` its outputs (AppImage, tarball, hdpack) so the release job can collect them.

Alternative if the refactor is too invasive: keep `build-appimage` as the sole uploader and have build-deb/build-rpm `actions/upload-artifact` to it via job needs + `download-artifact` step inserted before the `softprops/action-gh-release@v2` step. Either approach is fine — the key invariants are (a) all three jobs cache sherpa-onnx with the same key, (b) the final upload step uses explicit globs with `fail_on_unmatched_files: true`, (c) AppImage globs continue to match.

Why: this is the slice's primary deliverable — without these jobs, S05 cannot publish .deb or .rpm artifacts. Sharing the sherpa-onnx cache key across all three release jobs is critical to keep CI under the 60-min job budget; otherwise each job triggers a fresh sherpa-onnx-sys download and the combined runtime spikes.

Failure modes (Q5): rpmbuild's `%autosetup` will fail if the source tarball name doesn't match the spec's Source0 expansion (`vibe-attack-1.0.0.tar.gz`). dpkg-buildpackage will fail if libclang-dev is missing (clang is a transitive dep of sherpa-onnx-sys per MEM092). softprops/action-gh-release with `fail_on_unmatched_files: true` will fail loudly if any glob matches zero files — desirable per MEM086.
Load profile (Q6): one tag push triggers all three jobs in parallel; with sherpa cache hit each job is ~10–15 min; cold-cache first run could be 30–40 min (MEM089). GitHub Actions' default 6-hour job timeout is sufficient.
Negative tests (Q7): not exercised in this task — the unit-level assertion is in T03 (does the workflow YAML contain the right job names and globs?). A real tag-push validation is S05's responsibility per MEM111.
  - Files: `.github/workflows/release.yml`
  - Verify: yamllint .github/workflows/release.yml 2>/dev/null || python3 -c 'import yaml; yaml.safe_load(open(".github/workflows/release.yml"))' && grep -q '^  build-deb:' .github/workflows/release.yml && grep -q '^  build-rpm:' .github/workflows/release.yml && grep -q 'vibe-attack_\*\.deb' .github/workflows/release.yml && grep -q 'vibe-attack-\*\.x86_64\.rpm' .github/workflows/release.yml && grep -q 'fail_on_unmatched_files: true' .github/workflows/release.yml && grep -c 'sherpa-onnx-1.12.39-linux-x64' .github/workflows/release.yml | awk '{ exit ($1 < 3) }'

- [ ] **T03: Extend tests/packaging.rs to assert release.yml declares build-deb, build-rpm, and matching artifact globs** `est:30m`
  Add new static assertions to `tests/packaging.rs` that mechanically verify the release.yml contract. These tests run with `cargo test --test packaging` and require no build tools — they read the YAML as a string and grep for required substrings. This pattern matches the existing `release_yml_uploads_source_tarball` and `release_yml_uploads_hd2_hdpack` tests and the precedent set by MEM103.

Add these tests:
  1. `release_yml_has_build_deb_job` — assert the file contains `^  build-deb:` (anchored at column 2 — YAML job declaration).
  2. `release_yml_has_build_rpm_job` — assert the file contains `^  build-rpm:`.
  3. `release_yml_uploads_deb_artifact` — assert the upload block contains a glob matching `vibe-attack_*.deb` (Debian uses underscores in filenames).
  4. `release_yml_uploads_rpm_artifact` — assert the upload block contains a glob matching `vibe-attack-*.x86_64.rpm` (RPM convention with hyphens and arch in filename).
  5. `release_yml_caches_sherpa_onnx_in_all_release_jobs` — assert the sherpa cache key `sherpa-onnx-1.12.39-linux-x64` appears at least 3 times in release.yml (once per build job: appimage, deb, rpm) — this enforces MEM089 parity at PR-review time rather than discovering it at release time.

Use the same `read_file()` helper already in tests/packaging.rs. Use simple `.contains()` and `.matches().count()` checks; do not pull in a YAML parser — the existing tests intentionally use string contains for robustness against YAML formatting changes.

Also bump the existing `release_yml_uploads_source_tarball` assertion only if needed (current pattern likely still passes after T02). Do NOT delete or rename existing tests — they are the contract for AppImage + tarball + hdpack and must continue to assert.

Why: this is the slice's automated stopping condition. The slice cannot rely on a real tag-push for verification (that's S05's domain per MEM111). Static contract tests catch missing jobs/globs at PR time, exactly mirroring the rationale for MEM086 and MEM103. Without these tests, a future refactor to release.yml could silently drop one of the new jobs and ship a broken release.

Failure modes (Q5): if T02 named the jobs differently (e.g. `deb-build` instead of `build-deb`) these tests will fail — desirable; failure points at the exact contract violation. If MEM093's SKIP-sums policy were ever relaxed and PKGBUILD sha256sums changed, the existing PKGBUILD tests would not catch that — out of scope for this slice.
Load profile (Q6): tests are pure string reads, sub-second; no I/O concerns.
Negative tests (Q7): not applicable — these are positive-presence assertions.
  - Files: `tests/packaging.rs`
  - Verify: cargo test --test packaging 2>&1 | tee /tmp/s04-t03.log && grep -q 'release_yml_has_build_deb_job .* ok' /tmp/s04-t03.log && grep -q 'release_yml_has_build_rpm_job .* ok' /tmp/s04-t03.log && grep -q 'release_yml_uploads_deb_artifact .* ok' /tmp/s04-t03.log && grep -q 'release_yml_uploads_rpm_artifact .* ok' /tmp/s04-t03.log && grep -q 'release_yml_caches_sherpa_onnx_in_all_release_jobs .* ok' /tmp/s04-t03.log

## Files Likely Touched

- Cargo.toml
- packaging/vibe-attack.spec
- packaging/PKGBUILD
- packaging/debian/changelog
- CHANGELOG.md
- .github/workflows/release.yml
- tests/packaging.rs
