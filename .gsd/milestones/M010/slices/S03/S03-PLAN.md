# S03: Release CI workflow extension

**Goal:** Extend `.github/workflows/release.yml` so a tag push (`v*`) produces three release artifacts — the AppImage (already produced), a source tarball (`vibe-attack-${TAG}.tar.gz`), and a bundled HD2 pack (`hd2-${TAG}.hdpack`) — and add packaging tests asserting all three artifact patterns are referenced in the workflow YAML.
**Demo:** Tag push triggers release workflow; AppImage, tarball, .hdpack appear in GitHub Releases; workflow passes

## Must-Haves

- After this slice: `.github/workflows/release.yml` declares the sherpa-onnx prebuilt cache step (matching `ci.yml` key `sherpa-onnx-1.12.39-linux-x64`), produces `vibe-attack-${TAG}.tar.gz` via `git archive`, produces `hd2-${TAG}.hdpack` via `zip` of `profiles/hd2/pack.yaml`, and the `softprops/action-gh-release@v2` step uploads all three globs with `fail_on_unmatched_files: true`. `tests/packaging.rs` gains two new tests asserting tarball and hdpack artifact references in `release.yml`. All packaging tests (existing 5 + new 2 = 7) pass under `cargo test --test packaging -- --test-threads=1`.

## Proof Level

- This slice proves: contract — this slice proves the release CI workflow YAML structurally references all three artifacts; real-runtime proof (an actual tag push producing artifacts on GitHub Releases) is deferred to S06 final UAT, since invoking GitHub Actions from a planning context is out of scope and the workflow only fires on `push: tags: 'v*'`.

## Integration Closure

Not provided.

## Verification

- Not provided.

## Tasks

- [x] **T01: Extend release.yml with sherpa cache, source tarball, and HD2 hdpack steps** `est:30m`
  Add three additive changes to `.github/workflows/release.yml`: (1) a sherpa-onnx prebuilt cache block copied verbatim from `.github/workflows/ci.yml` (the two-step pattern: `actions/cache@v4` with id `sherpa-cache`, path `target/sherpa-onnx-prebuilt`, key `sherpa-onnx-1.12.39-linux-x64`, immediately followed by a conditional `cargo build -p sherpa-onnx-sys` step gated on `if: steps.sherpa-cache.outputs.cache-hit != 'true'`); (2) a step that creates the source tarball via `git archive --format=tar.gz --prefix=vibe-attack-${TAG}/ HEAD -o vibe-attack-${TAG}.tar.gz` using `GITHUB_REF_NAME` for the tag; (3) a step that creates the bundled HD2 pack via `zip -j hd2-${TAG}.hdpack profiles/hd2/pack.yaml`. Then update the existing `softprops/action-gh-release@v2` step so its `files:` block uses an explicit newline-separated list covering all three artifact globs (`vibe-attack-*-x86_64.AppImage`, `vibe-attack-*.tar.gz`, `hd2-*.hdpack`); keep `fail_on_unmatched_files: true`. The `zip` command is preinstalled on `ubuntu-22.04` runners — no extra apt-get install is required. Step ordering: insert the sherpa cache block between `Cache Rust build artifacts` (Swatinem/rust-cache@v2) and `Install system dependencies`; insert the tarball and hdpack steps after `Rename AppImage with version tag` and before the upload step. Reuse `${GITHUB_REF_NAME}` as the tag source in the same shell `run:` blocks (no new env exports needed).
  - Files: `.github/workflows/release.yml`, `.github/workflows/ci.yml`, `profiles/hd2/pack.yaml`
  - Verify: bash -c 'grep -q "sherpa-onnx-1.12.39-linux-x64" .github/workflows/release.yml && grep -q "git archive" .github/workflows/release.yml && grep -q "hd2-.*\.hdpack" .github/workflows/release.yml && grep -q "vibe-attack-\*\.tar\.gz" .github/workflows/release.yml && grep -q "fail_on_unmatched_files: true" .github/workflows/release.yml'

- [ ] **T02: Add packaging tests asserting release.yml uploads tarball and hdpack** `est:15m`
  Append two new `#[test]` functions to `tests/packaging.rs` following the existing `read_file` helper pattern: (1) `release_yml_uploads_source_tarball` — asserts `.github/workflows/release.yml` contains both `git archive` and the glob `vibe-attack-*.tar.gz` (or equivalent tarball file pattern) so the workflow demonstrably produces and uploads a source tarball; (2) `release_yml_uploads_hd2_hdpack` — asserts the workflow contains both a `zip` invocation referencing `profiles/hd2/pack.yaml` and the glob `hd2-*.hdpack`. Use the same `read_file("`.github/workflows/release.yml`")` style as the existing five tests; assert with `assert!(src.contains("..."), "...; got:\n{src}")` for parity. Do NOT change `read_file` or the existing five tests. Test names must be snake_case and live at the bottom of `tests/packaging.rs`. Run `cargo test --test packaging -- --test-threads=1` and confirm 7 tests pass (5 existing + 2 new). The `--test-threads=1` flag is mandatory per MEM005/MEM074 (shared-tmpdir flake prevention).
  - Files: `tests/packaging.rs`, `.github/workflows/release.yml`
  - Verify: cargo test --test packaging -- --test-threads=1 2>&1 | grep -q 'test result: ok. 7 passed'

## Files Likely Touched

- .github/workflows/release.yml
- .github/workflows/ci.yml
- profiles/hd2/pack.yaml
- tests/packaging.rs
