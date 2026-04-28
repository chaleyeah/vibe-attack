# S03: Release CI Workflow Extension — Research

**Date:** 2026-04-28
**Slice:** M010/S03 — Release CI workflow extension
**Risk:** low
**Depends:** S01

## Summary

S03 extends `.github/workflows/release.yml` to produce three artifacts on a tag push: the AppImage (already produced), a source tarball, and a bundled HD2 `.hdpack`. The existing workflow is functional but incomplete — it produces only the AppImage and is missing the sherpa-onnx prebuilt cache step that CI has, the source tarball step, and the `.hdpack` bundling step. This is low-risk, well-understood YAML work against a known GitHub Actions pattern.

The HD2 pack lives at `profiles/hd2/pack.yaml` (no sounds directory). Producing the `.hdpack` in CI is a one-liner: `cargo run --bin vibe-attack -- pack export hd2 hd2.hdpack` or a direct shell zip of `profiles/hd2/`. The source tarball is `git archive --format=tar.gz HEAD -o vibe-attack-<TAG>.tar.gz`.

A structural test asserting the release workflow YAML contains the three upload steps should be added to `tests/packaging.rs` (already the home for static workflow assertions).

## Recommendation

Make three additive changes to `.github/workflows/release.yml`:

1. **Add the sherpa-onnx prebuilt cache step** — copied verbatim from `ci.yml` (key `sherpa-onnx-1.12.39-linux-x64`); without it the release build will recompile sherpa-onnx-sys every time, adding ~10 min to the release job.
2. **Add a source tarball step** — `git archive --format=tar.gz --prefix=vibe-attack-${TAG}/ HEAD -o vibe-attack-${TAG}.tar.gz` immediately after the AppImage rename.
3. **Add an HD2 `.hdpack` bundling step** — zip `profiles/hd2/` into `hd2-${TAG}.hdpack` using a shell one-liner (no Rust binary invocation needed; the pack format is a ZIP with `pack.yaml` at root, matching `Pack::export`).
4. **Update the upload step** — pass all three artifact globs to `softprops/action-gh-release@v2`.
5. **Add a packaging test** — assert the release YAML references all three artifact types (AppImage, tarball, hdpack) in `tests/packaging.rs`.

## Implementation Landscape

### Key Files

- `.github/workflows/release.yml` — the sole file requiring change; currently 63 lines, produces AppImage only; missing sherpa cache, tarball, hdpack steps
- `.github/workflows/ci.yml` — reference for sherpa-onnx cache step pattern (copy verbatim: `actions/cache@v4`, path `target/sherpa-onnx-prebuilt`, key `sherpa-onnx-1.12.39-linux-x64`, rebuild-if-miss pattern)
- `profiles/hd2/pack.yaml` — source for the bundled HD2 pack; only `pack.yaml` exists (no `sounds/` dir), so the hdpack is just a zip of this single file
- `packaging/appimage/build.sh` — already correct; no changes needed
- `tests/packaging.rs` — existing static-assertion test file; add 1-2 new tests asserting release.yml contains tarball and hdpack upload patterns

### Build Order

1. **Add sherpa cache step to release.yml** — unblocks faster CI and is a prerequisite for a correct build; copy from ci.yml, insert between `rust-cache` and `Install system dependencies`.
2. **Add tarball step** — insert after `Rename AppImage with version tag`; uses `git archive`.
3. **Add hdpack step** — insert after tarball; uses `zip -j hd2-${TAG}.hdpack profiles/hd2/pack.yaml`.
4. **Update upload step** — change `files:` to include all three globs; verify `fail_on_unmatched_files: true` stays.
5. **Add packaging tests** — add `release_yml_uploads_tarball` and `release_yml_uploads_hdpack` tests to `tests/packaging.rs`.
6. **Run tests** — `cargo test --test packaging -- --test-threads=1` to confirm green.

### Verification Approach

- `cargo test --test packaging -- --test-threads=1` — structural gate; all existing 5 tests plus 2 new ones must pass
- Manual review: confirm `.github/workflows/release.yml` has 4 upload artifact entries and sherpa cache step
- `git archive --format=tar.gz HEAD | tar tz | head` — smoke test tarball generation locally
- `zip -j /tmp/hd2-test.hdpack profiles/hd2/pack.yaml && unzip -l /tmp/hd2-test.hdpack` — confirm hdpack format valid

## Constraints

- `linuxdeploy` and `appimagetool` must remain pinned to `continuous` (the project's current approach per existing workflow); do not change versions
- `softprops/action-gh-release@v2` — already pinned; keep as is
- `fail_on_unmatched_files: true` must remain in the upload step — it's a correctness guard
- sherpa-onnx cache key `sherpa-onnx-1.12.39-linux-x64` must match exactly what ci.yml uses (shared cache across jobs in the same repo)
- The hdpack is a ZIP — `Pack::export` writes Stored (uncompressed) ZipWriter entries; a shell `zip -j` with default deflate is acceptable for the release artifact (users import it via the GUI which calls `Pack::import`, which uses the zip crate's reader — compatible with any ZIP variant)
- The `zip` system command is available on ubuntu-22.04; no extra apt-get install needed

## Common Pitfalls

- **Missing sherpa rebuild guard** — the cache step must include the `if: steps.sherpa-cache.outputs.cache-hit != 'true'` rebuild step or a cache miss will produce a broken build silently. Copy the full two-step pattern from ci.yml.
- **TAG variable scope** — `GITHUB_REF_NAME` is already used for the AppImage rename; reuse it for tarball and hdpack names in the same shell step or export it as an env var earlier.
- **Glob ambiguity in softprops upload** — use explicit newline-separated globs in the `files:` block, not comma-separated, to avoid shell expansion issues.
- **hdpack path** — the upload step runs from the workspace root; produce the hdpack at workspace root (e.g., `hd2-${TAG}.hdpack`) so the glob `hd2-*.hdpack` resolves without a path prefix.
