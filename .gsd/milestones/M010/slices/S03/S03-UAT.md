# S03: Release CI workflow extension — UAT

**Milestone:** M010
**Written:** 2026-04-28T04:14:21.186Z

# S03: Release CI workflow extension — UAT

**Milestone:** M010
**Written:** 2026-04-28

## UAT Type

- UAT mode: artifact-driven
- Why this mode is sufficient: This slice proves structural contract — the release.yml YAML references all required artifact patterns. Real-runtime proof (an actual tag push to GitHub) is deferred to S06 final UAT, since invoking GitHub Actions from a planning context is out of scope and the workflow only fires on `push: tags: 'v*'`.

## Preconditions

- Working directory: `/home/chadmin/Github/hd-linux-voice`
- `cargo` available in PATH
- `.github/workflows/release.yml` and `tests/packaging.rs` present

## Smoke Test

Run `cargo test --test packaging -- --test-threads=1` and confirm output ends with `test result: ok. 7 passed`.

## Test Cases

### 1. All packaging tests pass (7 total)

1. Run: `cargo test --test packaging -- --test-threads=1`
2. **Expected:** `test result: ok. 7 passed; 0 failed; 0 ignored` — includes `release_yml_uploads_source_tarball` and `release_yml_uploads_hd2_hdpack`

### 2. Sherpa-onnx cache block present in release.yml

1. Run: `grep "sherpa-onnx-1.12.39-linux-x64" .github/workflows/release.yml`
2. **Expected:** Line(s) matching the cache key appear — confirms parity with ci.yml cache block

### 3. Source tarball step present

1. Run: `grep "git archive" .github/workflows/release.yml`
2. **Expected:** Line matching `git archive --format=tar.gz` appears — confirms tarball creation step

### 4. HD2 hdpack step present

1. Run: `grep "hd2-.*\.hdpack" .github/workflows/release.yml`
2. **Expected:** Line matching `hd2-*.hdpack` glob appears in the upload step

### 5. Upload step covers all three artifact globs

1. Run: `grep -A5 "softprops/action-gh-release" .github/workflows/release.yml`
2. **Expected:** Output includes `vibe-attack-*-x86_64.AppImage`, `vibe-attack-*.tar.gz`, and `hd2-*.hdpack` in the `files:` block

### 6. fail_on_unmatched_files guard present

1. Run: `grep "fail_on_unmatched_files: true" .github/workflows/release.yml`
2. **Expected:** Exactly one matching line — confirms the upload step will fail loudly if any artifact is missing

### 7. zip uses junk-paths flag for hdpack

1. Run: `grep "zip -j" .github/workflows/release.yml`
2. **Expected:** Line matching `zip -j "hd2-${TAG}.hdpack" profiles/hd2/pack.yaml` — confirms pack.yaml lands at archive root

## Edge Cases

### Tarball prefix includes trailing slash

1. Run: `grep 'prefix.*vibe-attack' .github/workflows/release.yml`
2. **Expected:** Pattern `--prefix="vibe-attack-${TAG}/"` with trailing slash — standard git archive convention for proper tarball extraction

### Conditional sherpa build step gated correctly

1. Run: `grep "cache-hit != 'true'" .github/workflows/release.yml`
2. **Expected:** The conditional build step for `cargo build -p sherpa-onnx-sys` is gated on `if: steps.sherpa-cache.outputs.cache-hit != 'true'`

## Failure Signals

- Any packaging test failing (non-zero exit from cargo test)
- `grep` returning no output for any structural check above
- `fail_on_unmatched_files: true` absent from release.yml upload step

## Not Proven By This UAT

- An actual tag push triggering the workflow on GitHub Actions (deferred to S06)
- AppImage, tarball, and hdpack actually appearing as downloadable artifacts in GitHub Releases (deferred to S06)
- Tarball integrity / extractability (deferred to S06)
- hdpack content correctness (deferred to S06)

## Notes for Tester

The `--test-threads=1` flag is mandatory for the packaging test suite (MEM005/MEM074 — shared tmpdir flake prevention). Omitting it may cause intermittent failures even when all tests are correct.
