---
id: S03
parent: M010
milestone: M010
provides:
  - ["release.yml structurally references sherpa-onnx cache, source tarball, HD2 hdpack, and multi-artifact upload with fail_on_unmatched_files: true", "7 packaging contract tests covering AppImage, PKGBUILD, and release workflow artifacts"]
requires:
  []
affects:
  []
key_files:
  - [".github/workflows/release.yml", "tests/packaging.rs"]
key_decisions:
  - ["Copied sherpa cache block verbatim from ci.yml to maintain key/path/conditional parity between CI and release workflows", "Used zip -j (junk-paths) for hdpack so pack.yaml lands at archive root rather than nested under profiles/hd2/", "Renamed upload step to 'Upload release assets' to reflect multi-artifact responsibility", "Used prefix-match assertions in new tests (git archive + vibe-attack- prefix; profiles/hd2/pack.yaml + .hdpack suffix) for robustness against minor YAML reformatting"]
patterns_established:
  - ["Explicit newline-separated glob list in softprops/action-gh-release@v2 files: block with fail_on_unmatched_files: true as the canonical release upload pattern", "Packaging test suite always invoked with --test-threads=1 to prevent shared-tmpdir flakes", "Release workflow structural assertions live in tests/packaging.rs as contract tests that catch regressions without requiring a real tag push"]
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-28T04:14:21.186Z
blocker_discovered: false
---

# S03: Release CI workflow extension

**release.yml extended with sherpa-onnx cache, source tarball, and HD2 hdpack steps; all three artifact globs upload on tag push with fail_on_unmatched_files: true; 7 packaging tests pass**

## What Happened

S03 extended `.github/workflows/release.yml` with three additive changes and added two new packaging tests, bringing the total to 7 passing.

**T01 — Extend release.yml (3 additions + 1 update):**

1. **Sherpa-onnx cache block** inserted between `Cache Rust build artifacts` (Swatinem/rust-cache@v2) and `Install system dependencies`, copied verbatim from `.github/workflows/ci.yml`: `actions/cache@v4` with id `sherpa-cache`, path `target/sherpa-onnx-prebuilt`, key `sherpa-onnx-1.12.39-linux-x64`, immediately followed by a conditional `cargo build -p sherpa-onnx-sys` step gated on `if: steps.sherpa-cache.outputs.cache-hit != 'true'`. This ensures release builds share the same prebuilt cache as CI builds and do not recompile sherpa-onnx-sys on cache hits.

2. **Source tarball step** inserted after `Rename AppImage with version tag`: `git archive --format=tar.gz --prefix="vibe-attack-${TAG}/" HEAD -o "vibe-attack-${TAG}.tar.gz"` using `TAG="${GITHUB_REF_NAME}"`. No new env exports needed.

3. **HD2 pack bundle step** inserted after the tarball step: `zip -j "hd2-${TAG}.hdpack" profiles/hd2/pack.yaml`. The `-j` flag (junk paths) ensures `pack.yaml` lands at the archive root rather than nested under `profiles/hd2/`. `profiles/hd2/pack.yaml` was confirmed to exist in the repo. `zip` is preinstalled on `ubuntu-22.04` runners — no apt-get install needed.

4. **Upload step updated**: renamed from "Upload AppImage as release asset" to "Upload release assets"; `files:` changed from a single-line value to an explicit newline-separated list covering all three globs: `vibe-attack-*-x86_64.AppImage`, `vibe-attack-*.tar.gz`, `hd2-*.hdpack`; `fail_on_unmatched_files: true` retained so the workflow fails loudly if any artifact is absent.

**T02 — Add two packaging tests:**

Appended two `#[test]` functions to `tests/packaging.rs` following the existing `read_file` pattern:
- `release_yml_uploads_source_tarball`: asserts `release.yml` contains `git archive` and `vibe-attack-` (prefix-match, robust to minor glob reformatting)
- `release_yml_uploads_hd2_hdpack`: asserts `release.yml` contains `profiles/hd2/pack.yaml` and `.hdpack`

Both tests ran under `cargo test --test packaging -- --test-threads=1` (mandatory single-thread per MEM005/MEM074). All 7 tests passed (5 pre-existing + 2 new).

## Verification

**T01 verification:** `bash -c 'grep -q "sherpa-onnx-1.12.39-linux-x64" .github/workflows/release.yml && grep -q "git archive" .github/workflows/release.yml && grep -q "hd2-.*\.hdpack" .github/workflows/release.yml && grep -q "vibe-attack-\*\.tar\.gz" .github/workflows/release.yml && grep -q "fail_on_unmatched_files: true" .github/workflows/release.yml'` — exit 0, all five structural checks passed.

**T02 verification:** `cargo test --test packaging -- --test-threads=1` — exit 0, output: `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`.

Both verifications run fresh at slice-close time, not from cached output.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

T02 used prefix/suffix matches (`git archive` + `vibe-attack-` prefix; `profiles/hd2/pack.yaml` + `.hdpack` suffix) rather than exact full-glob strings. This is more robust against minor YAML reformatting and still fully covers the structural contract.

## Known Limitations

Real-runtime proof — an actual tag push producing AppImage, tarball, and hdpack in GitHub Releases — is deferred to S06 final UAT. This slice proves structural contract only (workflow YAML references all required patterns and all 7 packaging tests pass).

## Follow-ups

S06 final UAT should push a real tag and verify all three artifacts appear as downloadable assets in GitHub Releases with correct content and sizes.

## Files Created/Modified

- `.github/workflows/release.yml` — Added sherpa-onnx cache block, source tarball step, HD2 hdpack step; updated upload step to cover all three artifact globs with fail_on_unmatched_files: true
- `tests/packaging.rs` — Appended release_yml_uploads_source_tarball and release_yml_uploads_hd2_hdpack test functions (total: 7 tests)
