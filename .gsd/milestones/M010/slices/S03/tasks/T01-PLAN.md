---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T01: Extend release.yml with sherpa cache, source tarball, and HD2 hdpack steps

Add three additive changes to `.github/workflows/release.yml`: (1) a sherpa-onnx prebuilt cache block copied verbatim from `.github/workflows/ci.yml` (the two-step pattern: `actions/cache@v4` with id `sherpa-cache`, path `target/sherpa-onnx-prebuilt`, key `sherpa-onnx-1.12.39-linux-x64`, immediately followed by a conditional `cargo build -p sherpa-onnx-sys` step gated on `if: steps.sherpa-cache.outputs.cache-hit != 'true'`); (2) a step that creates the source tarball via `git archive --format=tar.gz --prefix=vibe-attack-${TAG}/ HEAD -o vibe-attack-${TAG}.tar.gz` using `GITHUB_REF_NAME` for the tag; (3) a step that creates the bundled HD2 pack via `zip -j hd2-${TAG}.hdpack profiles/hd2/pack.yaml`. Then update the existing `softprops/action-gh-release@v2` step so its `files:` block uses an explicit newline-separated list covering all three artifact globs (`vibe-attack-*-x86_64.AppImage`, `vibe-attack-*.tar.gz`, `hd2-*.hdpack`); keep `fail_on_unmatched_files: true`. The `zip` command is preinstalled on `ubuntu-22.04` runners — no extra apt-get install is required. Step ordering: insert the sherpa cache block between `Cache Rust build artifacts` (Swatinem/rust-cache@v2) and `Install system dependencies`; insert the tarball and hdpack steps after `Rename AppImage with version tag` and before the upload step. Reuse `${GITHUB_REF_NAME}` as the tag source in the same shell `run:` blocks (no new env exports needed).

## Inputs

- ``.github/workflows/release.yml``
- ``.github/workflows/ci.yml``
- ``profiles/hd2/pack.yaml``

## Expected Output

- ``.github/workflows/release.yml``

## Verification

bash -c 'grep -q "sherpa-onnx-1.12.39-linux-x64" .github/workflows/release.yml && grep -q "git archive" .github/workflows/release.yml && grep -q "hd2-.*\.hdpack" .github/workflows/release.yml && grep -q "vibe-attack-\*\.tar\.gz" .github/workflows/release.yml && grep -q "fail_on_unmatched_files: true" .github/workflows/release.yml'

## Observability Impact

CI runtime signal: the release workflow must fail loudly on a missing artifact (kept by `fail_on_unmatched_files: true`). No new logs or status surfaces introduced — the GitHub Actions run page is the inspection surface for CI failures.
