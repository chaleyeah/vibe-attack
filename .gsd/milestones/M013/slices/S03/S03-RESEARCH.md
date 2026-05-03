# S03: Verify end-to-end release pipeline with test tag — Research

**Date:** 2026-05-03

## Summary

S01 and S02 fixed the versioning bugs and added PKGBUILD validation. The release pipeline (release.yml) has already run successfully once — the v1.0.0 tag produced all four artifacts (AppImage, .deb, .rpm, AUR-ready tarball) with correct version-stamped filenames confirmed on GitHub Releases. The primary task for S03 is to push a clean test tag that exercises the full pipeline *after the S01/S02 changes* and confirm everything is green end-to-end.

There is one active blocker: the CI `clippy` job currently fails on every push because `src/ui/pack_editor.rs:5` has an empty blank line after a `///` doc comment (clippy::empty_line_after_doc_comments, fatal under `-D warnings`). This was introduced by a post-S01/S02 commit and **must be fixed before the test tag can produce an all-green CI run**. The fix is a one-liner: convert the outer `///` block to `//!` inner module doc comments, which is exactly what clippy suggests.

CI and Release both trigger only on `v*` tags. A test tag such as `v1.0.1-rc.1` would trigger both workflows and is the correct mechanism. The `${GITHUB_REF_NAME#v}` stripping in both workflows strips the `v` prefix cleanly for any `vX.Y.Z...` pattern, so a pre-release tag will produce correctly versioned artifact filenames.

## Recommendation

1. Fix the clippy `empty_line_after_doc_comments` lint in `src/ui/pack_editor.rs` (change lines 1-5 from `///` to `//!` inner doc comments, matching clippy's own suggestion).
2. Commit and push a test tag (e.g. `v1.0.1-rc.1`) to GitHub.
3. Monitor the Actions run; verify all jobs green — CI (test, clippy, validate-pkgbuild) + Release (build-appimage, build-deb, build-rpm, release).
4. Confirm artifact filenames on the draft/pre-release contain the correct version string (not `1.0.0`).
5. Delete the test tag and pre-release after verification.

## Implementation Landscape

### Key Files

- `.github/workflows/ci.yml` — Runs on `v*` tags; jobs: test, clippy, validate-pkgbuild. Currently failing on `Clippy (gui feature)` due to pack_editor.rs lint.
- `.github/workflows/release.yml` — Runs on `v*` tags; jobs: build-appimage, build-deb, build-rpm (parallel), then release. All ran and passed for v1.0.0.
- `src/ui/pack_editor.rs` — Lines 1-5: `///` outer doc comments followed by blank line before `#[cfg(feature = "gui")]`. Clippy requires these to be `//!` inner doc comments (module-level). One-line fix.
- `packaging/vibe-attack.spec` — Source spec; `Version: 1.0.0` is still hardcoded here, but release.yml rewrites it via `sed "s/^Version:.*/Version: ${TAG}/"` into `~/rpmbuild/SPECS/`. This is correct — do not change the source spec.
- `packaging/debian/changelog` — Top entry is `1.0.0-1`; release.yml prepends a new entry with the tag version before `dpkg-buildpackage`. Correct behavior.
- `packaging/PKGBUILD` — sha256sums pinned to real v1.0.0 tarball hashes. CI validate-pkgbuild job only checks syntax and field presence (not hashes), so this passes regardless of test tag.

### Build Order

1. Fix clippy lint in `src/ui/pack_editor.rs` — **must come first**; without this, CI will fail on every tag push and the test tag cannot go green.
2. Commit the fix.
3. Push test tag (e.g. `v1.0.1-rc.1`) to GitHub remote.
4. Monitor both workflow runs in GitHub Actions and confirm all jobs pass.
5. Inspect the GitHub Release draft — check artifact filenames contain the test version.
6. Clean up: delete the test tag from remote and local, delete the draft pre-release from GitHub Releases.

### Verification Approach

**Local pre-checks (before pushing tag):**
```bash
# Confirm clippy lint is fixed
cargo clippy --all-targets --features gui -- -D warnings

# Confirm validate-pkgbuild logic still works
bash -n packaging/PKGBUILD && echo "syntax OK"
bash -c "source packaging/PKGBUILD; for f in pkgname pkgver pkgrel arch license; do echo \"\$f=\${!f}\"; done"

# Simulate RPM version sed
sed "s/^Version:.*/Version:        1.0.1/" packaging/vibe-attack.spec | grep "^Version"

# Simulate deb changelog stamp
TAG="1.0.1-rc.1"; { echo "vibe-attack (${TAG}-1) unstable; urgency=medium"; echo ""; echo "  * Release ${TAG}."; echo ""; echo " -- Chris Chale <chrischale@gmail.com>  $(date -R)"; echo ""; cat packaging/debian/changelog; } | head -10
```

**Remote verification (after tag push):**
```bash
# Watch CI run
gh run list --repo chaleyeah/vibe-attack --limit 5

# Check specific run jobs
gh run view <RUN_ID> --repo chaleyeah/vibe-attack

# List release assets to verify version in filenames
gh release view v1.0.1-rc.1 --repo chaleyeah/vibe-attack
```

**Expected artifact names for tag `v1.0.1-rc.1`:**
- `vibe-attack-v1.0.1-rc.1-x86_64.AppImage`
- `vibe-attack-v1.0.1-rc.1.tar.gz`
- `hd2-v1.0.1-rc.1.hdpack`
- `vibe-attack_1.0.1-rc.1-1_amd64.deb`
- `vibe-attack-1.0.1-rc.1-1.x86_64.rpm`

**Cleanup:**
```bash
git push origin :refs/tags/v1.0.1-rc.1   # delete remote tag
git tag -d v1.0.1-rc.1                    # delete local tag
gh release delete v1.0.1-rc.1 --repo chaleyeah/vibe-attack --yes
```

## Common Pitfalls

- **Pre-release tag format** — The `.` in `rc.1` is valid in semver but `${GITHUB_REF_NAME#v}` will yield `1.0.1-rc.1` which dpkg-buildpackage may complain about (Debian epoch/tilde notation). Safer test tag: `v1.0.1` or `v1.0.1-1`. Alternatively use a tag that looks like a clean version: `v1.0.1-test` is fine for RPM but Debian version strings have restrictions. **Recommended**: use `v1.0.1` as a clean test tag, or if v1.0.1 would conflict with a future real release, push it to a pre-release and immediately delete it.

- **Tag uniqueness** — `v1.0.1` should not be "burned" if there's intent to use it as a real release. Consider pushing `v1.0.1` as a real next version if the pipeline passes, or use a clearly disposable tag pattern agreed with the maintainer.

- **Node.js 20 deprecation warnings** — GitHub Actions is warning that `actions/cache@v4`, `actions/checkout@v4`, and related actions run on Node.js 20 (deprecated, EOL September 2026). These are currently just warnings and will not fail the workflow. Bumping to `@v5` or `@v4.x` variants that use Node.js 24 is optional for S03 but worth noting as future work.

- **MEM117 gotcha applies** — The RPM tarball prefix must match the spec's `Version:` field (post-sed rewrite). The `git archive --prefix="vibe-attack-${TAG}/"` in release.yml uses `TAG="${GITHUB_REF_NAME#v}"` which is consistent with the `sed` rewrite. This is already correct from S01.

## Open Risks

- If `v1.0.1` is pushed as a test tag and the run fails for a new reason (not the clippy fix), the iteration time is ~10 minutes per attempt. Have the local pre-checks passing first.
- The Rust build cache (`Swatinem/rust-cache@v2`) may not have a warm cache for the test tag if the `gui` feature crate list changed since v1.0.0 — RPM build in particular took 10m16s on v1.0.0. The test run may be slow.

## Sources

- GitHub Actions run 25088427524 (v1.0.0 release, all green) — confirmed all four artifact names in the release.
- GitHub Actions run 25289985501 (latest CI, failing) — confirmed clippy failure at `src/ui/pack_editor.rs:5`.
