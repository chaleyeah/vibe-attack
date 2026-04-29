# S05: Publish GitHub Release v1.0.0 — Research

**Date:** 2026-04-28

## Summary

S05 is the final milestone slice: push the `v1.0.0` tag to trigger the release workflow, confirm GitHub Actions produces all four artifacts (AppImage, .deb, .rpm, source tarball), verify the release is live, and pin real sha256sums into `packaging/PKGBUILD` for the AUR submission.

The release.yml workflow was fully built and statically tested in S04. It is a 4-job architecture: `build-appimage`, `build-deb`, `build-rpm` run in parallel and each emit `upload-artifact`; a `release` collector job runs `softprops/action-gh-release@v2` with `fail_on_unmatched_files: true` after all three complete. The workflow fires on `push: tags: v*`. There are no existing GitHub Releases on this repository — the only prior tag is `v0.1.0` (unrelated release artifact).

This is straightforward execution work: commit the current staged changes (git status shows several modified/untracked GSD files but no source code changes), push the `v1.0.0` tag, wait for CI, then pin sha256sums. No code changes are required before the tag push. The only post-release code change is writing two sha256 hex strings into `packaging/PKGBUILD`'s `sha256sums` array.

## Recommendation

**T01:** Commit any outstanding changes to main, then `git tag v1.0.0 && git push origin v1.0.0`. Monitor the resulting Actions run via `gh run watch` or `gh run list`. No source changes are needed — all packaging manifests already read `1.0.0` and the workflow is already in place.

**T02:** After the release job completes, confirm all five artifact globs resolved by listing the release assets via `gh release view v1.0.0 --json assets`. Then compute sha256sums for both PKGBUILD source entries and update `packaging/PKGBUILD`'s `sha256sums` field.

The sherpa-onnx tarball sha256 can be pre-computed now (it is a fixed upstream artifact that doesn't change). The project source tarball sha256 requires the live `v1.0.0` tag to be present on GitHub before it can be hashed.

## Implementation Landscape

### Key Files

- `.github/workflows/release.yml` — 4-job workflow that fires on `v*` tags; already correct, no changes needed
- `packaging/PKGBUILD` — `sha256sums=('SKIP','SKIP')` at lines 21-22; both entries need real hashes after tag push
- `docs/distribution-proofs/aur/README.md` — operator-runbook for AUR submission steps; already complete; documents the exact `curl | sha256sum` commands to use
- `CHANGELOG.md` — `## [1.0.0] - 2026-04-28` block already in place; no changes needed

### Build Order

1. **Tag push** — this is the only action that triggers the release workflow. Commit outstanding GSD files first so HEAD is clean, then tag and push. The tag must be `v1.0.0` (matches `v*` trigger and the `1.0.0` strings baked into the RPM source tarball prefix and spec).
2. **Monitor CI run** — `gh run list --workflow=release.yml --limit=3` or `gh run watch <run-id>`. Each build job takes ~5-10 min (Rust compile + sherpa-onnx cache). The release job fires after all three complete.
3. **Verify release assets** — `gh release view v1.0.0 --json assets --jq '.assets[].name'` should list: `vibe-attack-v1.0.0-x86_64.AppImage`, `vibe-attack-v1.0.0.tar.gz`, `hd2-v1.0.0.hdpack`, one `vibe-attack_1.0.0-1_amd64.deb`, one `vibe-attack-1.0.0-1.x86_64.rpm`.
4. **Pin sha256sums** — compute both hashes and edit `packaging/PKGBUILD`. Commit as `packaging: pin PKGBUILD sha256sums for v1.0.0`.

### Verification Approach

**Release live:**
```bash
gh release view v1.0.0 --json tagName,isDraft,assets \
  --jq '{tag: .tagName, draft: .isDraft, count: (.assets | length)}'
# Expected: tag=v1.0.0, draft=false, count=5
```

**AppImage URL reachable (unblocks S02/T03):**
```bash
curl -sI "https://github.com/chaleyeah/vibe-attack/releases/latest/download/vibe-attack-x86_64.AppImage" \
  | head -1
# Expected: HTTP/2 302 (GitHub redirect to CDN) — not 404
```
Note: the redirect-to-CDN pattern is expected (GitHub Releases always redirect). HTTP 200 only appears after following the redirect.

**PKGBUILD hashes present:**
```bash
grep sha256sums packaging/PKGBUILD
# Expected: two 64-char hex strings, no 'SKIP'
```

**Packaging tests still pass:**
```bash
cargo test --test packaging
# Expected: 15 passed
```

## Constraints

- `git push origin v1.0.0` requires the `workflow` scope on the GitHub token — `gh auth status` confirms this is present (`Token scopes: 'gist', 'read:org', 'repo', 'workflow'`).
- The RPM tarball is generated with hardcoded prefix `vibe-attack-1.0.0/` (MEM117). The tag must be `v1.0.0` for the AppImage/tarball filenames (which use `GITHUB_REF_NAME`) to match the `vibe-attack-*` globs in the release step. These are consistent.
- The source[0] sha256 in PKGBUILD is for `https://github.com/chaleyeah/vibe-attack/archive/v1.0.0.tar.gz` — GitHub generates this from the tag. It is only available after the tag is pushed.
- The source[1] sha256 (sherpa-onnx 1.12.39 prebuilt) is a fixed upstream artifact and can be pre-computed or computed post-tag.

## Common Pitfalls

- **AppImage download URL for final transcripts** — S02/T03 is unblocked by this release. The final transcripts use `releases/latest/download/vibe-attack-x86_64.AppImage` (no version in the filename). The release.yml names it `vibe-attack-${TAG}-x86_64.AppImage`. The `latest/download/` path resolves via GitHub's redirect to the most recent release asset matching that suffix — this will work because the asset name ends with `-x86_64.AppImage`.
- **GSD file commit before tag** — git status shows untracked `.gsd/` files. These should be committed before tagging so the tag points to a clean, fully-documented HEAD.
- **`fail_on_unmatched_files: true` is a hard gate** — if any build job fails to produce its artifact, the entire release job fails loudly. This is intentional (MEM086), but means a single job failure prevents any release from publishing.
- **AUR sha256sums update is post-release work** — PKGBUILD update cannot happen until the GitHub Release is live and the tarball URL returns 200. Do not attempt to pre-compute source[0] hash.
