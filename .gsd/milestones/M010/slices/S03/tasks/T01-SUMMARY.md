---
id: T01
parent: S03
milestone: M010
key_files:
  - .github/workflows/release.yml
key_decisions:
  - Copied sherpa cache block verbatim from ci.yml to maintain consistency between CI and release workflows
  - Used zip -j (junk paths) for hdpack so pack.yaml lands at the archive root rather than nested under profiles/hd2/
  - Renamed upload step to 'Upload release assets' to reflect that it now handles three artifact types
duration: 
verification_result: passed
completed_at: 2026-04-28T04:12:11.260Z
blocker_discovered: false
---

# T01: Extend release.yml with sherpa-onnx cache, source tarball, and HD2 hdpack steps covering all three release artifacts

**Extend release.yml with sherpa-onnx cache, source tarball, and HD2 hdpack steps covering all three release artifacts**

## What Happened

Read the existing `.github/workflows/release.yml` and `.github/workflows/ci.yml` to identify the exact sherpa cache block pattern and step ordering. Made three additive changes to `release.yml`:

1. **Sherpa cache block** — inserted between `Cache Rust build artifacts` (Swatinem/rust-cache@v2) and `Install system dependencies`, copying the two-step pattern verbatim from `ci.yml`: `actions/cache@v4` with id `sherpa-cache`, path `target/sherpa-onnx-prebuilt`, key `sherpa-onnx-1.12.39-linux-x64`, immediately followed by a conditional `cargo build -p sherpa-onnx-sys` step gated on `if: steps.sherpa-cache.outputs.cache-hit != 'true'`.

2. **Source tarball step** — inserted after `Rename AppImage with version tag`, uses `git archive --format=tar.gz --prefix="vibe-attack-${TAG}/" HEAD -o "vibe-attack-${TAG}.tar.gz"` with `TAG="${GITHUB_REF_NAME}"`.

3. **HD2 pack bundle step** — inserted after the tarball step, uses `zip -j "hd2-${TAG}.hdpack" profiles/hd2/pack.yaml` (confirmed `profiles/hd2/pack.yaml` exists in the repo).

4. **Upload step updated** — renamed from "Upload AppImage as release asset" to "Upload release assets" and changed `files:` from a single-line value to an explicit newline-separated list covering all three globs: `vibe-attack-*-x86_64.AppImage`, `vibe-attack-*.tar.gz`, `hd2-*.hdpack`. Kept `fail_on_unmatched_files: true` so the workflow fails loudly if any artifact is missing.

No new env exports were needed — `GITHUB_REF_NAME` is already available in the runner environment. The `zip` command is preinstalled on `ubuntu-22.04` runners so no apt-get install step was required.

## Verification

Ran the task plan verification command: `bash -c 'grep -q "sherpa-onnx-1.12.39-linux-x64" .github/workflows/release.yml && grep -q "git archive" .github/workflows/release.yml && grep -q "hd2-.*\.hdpack" .github/workflows/release.yml && grep -q "vibe-attack-\*\.tar\.gz" .github/workflows/release.yml && grep -q "fail_on_unmatched_files: true" .github/workflows/release.yml'` — all five grep checks passed (exit 0).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `bash -c 'grep -q "sherpa-onnx-1.12.39-linux-x64" .github/workflows/release.yml && grep -q "git archive" .github/workflows/release.yml && grep -q "hd2-.*\.hdpack" .github/workflows/release.yml && grep -q "vibe-attack-\*\.tar\.gz" .github/workflows/release.yml && grep -q "fail_on_unmatched_files: true" .github/workflows/release.yml'` | 0 | ✅ pass | 50ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `.github/workflows/release.yml`
