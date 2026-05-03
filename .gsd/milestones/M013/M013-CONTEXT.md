# M013: CI Build Revamp & Package Distribution

**Gathered:** 2026-05-03
**Status:** Ready for planning

## Project Description

Fix the release pipeline so that pushing a `vN.N.N` git tag reproducibly produces all four package artifacts (AppImage, `.deb`, `.rpm`, AUR-ready tarball) with the correct version embedded in each — no manual fixups, no hardcoded version strings.

## Why This Milestone

The v1.0.0 release worked, but only after manual intervention: the RPM spec had `Version: 1.0.0` hardcoded, the Debian changelog wasn't stamped with the tag version, and there was no CI validation of the PKGBUILD. The goal is a clean pipeline where a tag push is the only required action. This matters now because the project is at a releasable state and the release tooling should be trustworthy before the next version.

## User-Visible Outcome

### When this milestone is complete, the user can:

- Push a `vN.N.N` tag and watch all 7 CI/Release jobs go green with no manual intervention
- Download the resulting GitHub Release assets and find the correct version string embedded in each artifact's filename and package metadata
- Confirm the AUR PKGBUILD is validated in CI on every tag push

### Entry point / environment

- Entry point: `git tag vN.N.N && git push origin vN.N.N`
- Environment: GitHub Actions (CI + Release workflows), GitHub Releases
- Live dependencies involved: GitHub Actions, GitHub Releases, `gh` CLI

## Completion Class

- Contract complete means: both workflow files (`ci.yml`, `release.yml`) parse correctly and all jobs are defined with correct steps
- Integration complete means: a real test tag push triggers all 7 jobs and they pass end-to-end with correctly versioned artifacts on the resulting GitHub Release
- Operational complete means: test tag and release are cleaned up after verification; main branch reflects the fixed pipeline state

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- Tag `v1.0.1-test` pushed → all 7 CI/Release jobs green → 5 release assets present on GitHub Release with correct version in filenames
- RPM spec `Version:` field in the built `.rpm` matches the tag (not `1.0.0`)
- Debian changelog `(version)` line matches the tag
- Test tag `v1.0.1-test` and its GitHub Release are deleted after verification

## Architectural Decisions

### Inner doc comments for pack_editor.rs

**Decision:** Convert `///` outer doc comments at the top of `src/ui/pack_editor.rs` to `//!` inner module doc comments.

**Rationale:** The `clippy::empty_line_after_doc_comments` lint fires under `-D warnings` when an outer `///` block is followed by a blank line before `#[cfg(feature = "gui")]`. Clippy's own suggestion is `//!` since the comments document the module itself. This is a prerequisite for a clean CI run.

**Alternatives Considered:**
- Remove the blank line between `///` and `#[cfg]` — technically fixes the lint but is semantically wrong; these are module docs, not item docs
- Suppress with `#[allow(clippy::...)]` — adds noise and hides a real issue

---

### RPM tarball prefix fix

**Decision:** Use `GITHUB_REF_NAME` (without `refs/tags/` prefix) in the RPM spec `Source0` tarball URL and in `release.yml`'s tarball creation step.

**Rationale:** The v1.0.0 release had the tarball prefix and the spec `Version:` field both hardcoded. `GITHUB_REF_NAME` gives the bare tag name (e.g. `v1.0.1`) which is what both the GitHub tarball URL and the RPM version field need.

**Alternatives Considered:**
- `GITHUB_REF` — includes `refs/tags/` prefix, requires stripping
- Hardcoded version — what we had; breaks every release

---

### Debian changelog stamping

**Decision:** Stamp the Debian changelog with the tag version in `release.yml` using `sed` before `dpkg-buildpackage`.

**Rationale:** `dpkg-buildpackage` reads the version from `debian/changelog`; without stamping it the `.deb` always reports the static version in the committed changelog.

**Alternatives Considered:**
- Commit a version bump to `debian/changelog` per release — requires an extra commit/PR per release; defeats the tag-driven model

---

### PKGBUILD CI validation

**Decision:** Add a `validate-pkgbuild` job to `ci.yml` that runs `namcap` (or a shell-based field check) on `packaging/PKGBUILD` on every push/PR.

**Rationale:** The PKGBUILD was shipping without any CI validation. A syntax or field regression would only be caught at AUR submission time.

**Alternatives Considered:**
- Validate only on tag push — misses regressions during development
- No validation — status quo; too late to catch issues

## Error Handling Strategy

The release pipeline is a sequence of independent artifact-build jobs. Failures are surfaced via GitHub Actions per-job status. The acceptance test uses `gh run view --log-failed` to inspect any failing step. There are no retries — a failure means fixing the workflow and re-pushing the test tag. No user-facing error messages are involved (this is a CI pipeline, not a user-facing feature).

## Risks and Unknowns

- Debian version string format — `v1.0.1-test` contains a hyphen which is valid in Debian version strings, but `dpkg` parses the epoch:upstream-revision format; `1.0.1~test` might be safer — accepted risk, will verify in T02/T03
- GitHub Actions Node.js 20 deprecation warnings — non-blocking for now; actions/upload-artifact@v3 etc. may emit warnings but jobs still pass
- `cargo clippy` not installed on dev machine — CI is the authoritative clippy check; local verification uses `cargo build --all-targets`

## Existing Codebase / Prior Art

- `.github/workflows/release.yml` — the workflow with the RPM/deb versioning bugs fixed in S01
- `.github/workflows/ci.yml` — the CI workflow with the validate-pkgbuild job added in S02
- `packaging/PKGBUILD` — the AUR build script being validated
- `packaging/vibe-attack.spec` — the RPM spec with the now-corrected `Version:` field
- `packaging/debian/changelog` — the Debian changelog stamped at release time
- `src/ui/pack_editor.rs` — the file with the clippy lint that blocks a clean CI run (T01 of S03)

## Relevant Requirements

- Clean tag-push release pipeline — this milestone is the primary delivery vehicle
- Distribution on Debian, Red Hat, and Arch — each artifact format (deb, rpm, PKGBUILD) is a direct requirement

## Scope

### In Scope

- Fix RPM `Version:` hardcoding in `release.yml` and `vibe-attack.spec`
- Fix Debian changelog version stamping in `release.yml`
- Add `validate-pkgbuild` job to `ci.yml`
- Add README CI/Release status badges (S04)
- Fix `pack_editor.rs` clippy lint (T01, S03)
- End-to-end smoke test via test tag push (S03)

### Out of Scope / Non-Goals

- Windows packaging (future/deferred)
- Snap, Flatpak, or other packaging formats
- Automated AUR submission (manual AUR publish remains out of scope)
- Semantic versioning automation / changelog generation
- Updating Node.js action versions to eliminate deprecation warnings (non-blocking)

## Technical Constraints

- `cargo clippy` is not installed on the dev machine; use `cargo build --all-targets` locally; CI is authoritative for clippy
- Test suite must run with `--test-threads=1` due to shared-tmpdir flake in `test_pack_export_import_with_sounds`
- GitHub Actions workflows must continue to support Debian, Red Hat, and Arch build targets

## Integration Points

- GitHub Actions — runs CI and Release workflows on push/tag
- GitHub Releases — hosts the built artifacts
- `gh` CLI — used to inspect workflow runs and release assets during verification

## Testing Requirements

S03 is the integration test for this entire milestone: push a real tag to a real GitHub repository, observe all 7 jobs pass, download and inspect artifacts. There are no unit tests for CI workflow files. The `validate-pkgbuild` job in CI is itself a continuous regression test for the PKGBUILD.

## Acceptance Criteria

- **S01:** `release.yml` diff shows RPM tarball prefix uses `GITHUB_REF_NAME`, deb changelog stamped with tag version, `dh-cargo` removed
- **S02:** `ci.yml` `validate-pkgbuild` job runs and passes, showing all required fields and function definitions present
- **S03:** GitHub Actions run shows all 7 jobs green; downloaded artifacts have correct version in filename and package metadata; test tag and release cleaned up after
- **S04:** README shows two rendered badge images linking to the correct Actions workflow runs

## Open Questions

- Debian version string for test tag — `v1.0.1-test` vs `v1.0.1~test` — current thinking: test with hyphen first; if `dpkg` rejects it, switch to tilde notation
