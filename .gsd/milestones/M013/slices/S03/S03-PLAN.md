# S03: S03

**Goal:** Push a clean test tag to GitHub and confirm both CI (test, clippy, validate-pkgbuild) and Release (build-appimage, build-deb, build-rpm, release) workflows run all-green end-to-end after the S01/S02 changes, with version-stamped artifact filenames present on the resulting GitHub Release.
**Demo:** GitHub Actions run shows all jobs green; downloaded artifacts have correct version in filename and package metadata

## Must-Haves

- A GitHub Actions run for tag `v1.0.1-test` shows every job in ci.yml and release.yml passing. The resulting GitHub Release (or pre-release/draft) has assets named `vibe-attack-v1.0.1-test-x86_64.AppImage`, `vibe-attack-v1.0.1-test.tar.gz`, `hd2-v1.0.1-test.hdpack`, `vibe-attack_1.0.1-test-1_amd64.deb`, and `vibe-attack-1.0.1-test-1.x86_64.rpm`. The test tag and pre-release are deleted from origin and locally after verification.

## Proof Level

- This slice proves: - This slice proves: final-assembly
- Real runtime required: yes (GitHub Actions workflow execution)
- Human/UAT required: no (verification is via gh CLI introspection of jobs and assets)

## Integration Closure

- Upstream surfaces consumed: `.github/workflows/ci.yml` (post-S02), `.github/workflows/release.yml` (post-S01), `packaging/vibe-attack.spec`, `packaging/debian/changelog`, `packaging/PKGBUILD`, `src/ui/pack_editor.rs`
- New wiring introduced in this slice: none — this is the end-to-end smoke test of the wiring done in S01 and S02
- What remains before the milestone is truly usable end-to-end: S04 (README CI/Release status badges)

## Verification

- Runtime signals: GitHub Actions per-job status (success/failure), workflow run logs, release asset list
- Inspection surfaces: `gh run list`, `gh run view <RUN_ID>`, `gh release view v1.0.1-test --repo chaleyeah/vibe-attack`
- Failure visibility: GH Actions retains per-step logs; failing job names and step exit codes are surfaced via `gh run view --log-failed`
- Redaction constraints: none — public repo, no secrets in artifacts

## Tasks

- [x] **T01: Converted outer /// doc comments in pack_editor.rs to //! inner form to fix clippy::empty_line_after_doc_comments lint** `est:20m`
  Convert the outer `///` doc comments at the top of `src/ui/pack_editor.rs` (lines 1-5) into `//!` inner module doc comments. The current form — outer `///` block followed by a blank line and then `#[cfg(feature = "gui")]` — triggers `clippy::empty_line_after_doc_comments` which is fatal under `-D warnings` in CI. Clippy's own suggestion is to make these inner doc comments since they document the module itself, not the re-export below them. After editing, verify locally with `cargo build --all-targets` (default features) and `cargo build --all-targets --features gui` since `cargo clippy` is not installed in this dev environment (per MEM038/MEM073). The CI clippy job will be the authoritative check once the test tag is pushed in T02.
  - Files: `src/ui/pack_editor.rs`
  - Verify: cargo build --all-targets 2>&1 | grep -E '^(warning|error)' | wc -l | grep -qx 0 && cargo build --all-targets --features gui 2>&1 | grep -E '^(warning|error)' | wc -l | grep -qx 0 && grep -q '^//! Pack editor panel' src/ui/pack_editor.rs

- [x] **T02: Pushed v1.0.1-test tag and triggered CI+Release workflows; CI passed but Release failed on RPM (hyphen in Version field) and Debian (dpkg build-dep check blocks rustup-installed Rust)** `est:30m`
  Create and push a disposable test tag `v1.0.1-test` to the `chaleyeah/vibe-attack` GitHub remote. This triggers both `.github/workflows/ci.yml` and `.github/workflows/release.yml` (both watch `tags: ['v*']`). Monitor both workflow runs and confirm every job succeeds. Do NOT delete the tag in this task — T03 needs it live to inspect release assets.
  - Files: `.github/workflows/ci.yml`, `.github/workflows/release.yml`
  - Verify: gh run list --repo chaleyeah/vibe-attack --event push --limit 2 --json conclusion,name --jq 'map(select(.name=="CI" or .name=="Release")) | length' | grep -qx 2 && gh run list --repo chaleyeah/vibe-attack --event push --limit 2 --json conclusion --jq 'all(.[]; .conclusion=="success")' | grep -qx true && git ls-remote --tags origin v1.0.1-test | grep -q refs/tags/v1.0.1-test

- [x] **T03: Fix RPM hyphen and Debian build-dep blockers in release.yml; clean up stale v1.0.1-test tag/release** `est:25m`
  Apply two surgical fixes to `.github/workflows/release.yml` to unblock the Release workflow, then remove the stale `v1.0.1-test` tag and any partial release left over from the failed T02 run.
  - Files: `.github/workflows/release.yml`
  - Verify: grep -q 'RPM_VERSION="\${TAG//-/~}"' .github/workflows/release.yml && grep -q 'dpkg-buildpackage -uc -us -b -d' .github/workflows/release.yml && python3 -c 'import yaml; yaml.safe_load(open(".github/workflows/release.yml"))' && ! git ls-remote --tags origin v1.0.1-test 2>/dev/null | grep -q refs/tags/v1.0.1-test && ! git tag -l v1.0.1-test | grep -q v1.0.1-test

- [x] **T04: Push v1.0.1-test2 tag and confirm CI + Release workflows finish all-green** `est:30m`
  Create and push a fresh disposable test tag `v1.0.1-test2` to the `chaleyeah/vibe-attack` GitHub remote. Using `-test2` rather than reusing `-test` ensures no GitHub Actions cache state, artifact name collision, or release-name collision from the prior failed T02 run. The new tag triggers both `.github/workflows/ci.yml` and the now-fixed `.github/workflows/release.yml`. Monitor both runs and confirm every job succeeds.
  - Files: `.github/workflows/ci.yml`, `.github/workflows/release.yml`
  - Verify: git ls-remote --tags origin v1.0.1-test2 | grep -q refs/tags/v1.0.1-test2 && gh run list --repo chaleyeah/vibe-attack --event push --limit 10 --json headBranch,name,conclusion --jq '[.[] | select(.headBranch=="v1.0.1-test2")] | length' | grep -qx 2 && gh run list --repo chaleyeah/vibe-attack --event push --limit 10 --json headBranch,name,conclusion --jq '[.[] | select(.headBranch=="v1.0.1-test2") | .conclusion] | all(. == "success")' | grep -qx true

- [x] **T05: Verify v1.0.1-test2 release assets and clean up the test tag** `est:15m`
  Confirm the GitHub Release created by the `v1.0.1-test2` tag contains all five expected assets with correct version-stamped filenames. Note the RPM filename uses `~` (tilde) instead of `-` because of the T03 RPM_VERSION substitution. Then delete the test tag and the release from both origin and locally. This task closes the slice.
  - Files: `.gsd/milestones/M013/slices/S03/T05-RESULT.md`
  - Verify: test -f .gsd/milestones/M013/slices/S03/T05-RESULT.md && grep -q 'vibe-attack-v1.0.1-test2-x86_64.AppImage' .gsd/milestones/M013/slices/S03/T05-RESULT.md && grep -q 'vibe-attack-1.0.1~test2-1.x86_64.rpm' .gsd/milestones/M013/slices/S03/T05-RESULT.md && grep -q 'vibe-attack_1.0.1-test2-1_amd64.deb' .gsd/milestones/M013/slices/S03/T05-RESULT.md && grep -q 'cleanup confirmed' .gsd/milestones/M013/slices/S03/T05-RESULT.md && ! git ls-remote --tags origin v1.0.1-test2 2>/dev/null | grep -q refs/tags/v1.0.1-test2

## Files Likely Touched

- src/ui/pack_editor.rs
- .github/workflows/ci.yml
- .github/workflows/release.yml
- .gsd/milestones/M013/slices/S03/T05-RESULT.md
