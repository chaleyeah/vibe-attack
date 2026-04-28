---
id: T02
parent: S03
milestone: M010
key_files:
  - tests/packaging.rs
key_decisions:
  - Used `src.contains("git archive") && src.contains("vibe-attack-")` for the tarball test (matching prefix rather than full glob) to be robust against minor reformatting of the workflow YAML
  - Used `src.contains("profiles/hd2/pack.yaml") && src.contains(".hdpack")` for the hdpack test, matching both the zip source and the upload glob suffix
duration: 
verification_result: passed
completed_at: 2026-04-28T04:12:58.456Z
blocker_discovered: false
---

# T02: Add two packaging tests asserting release.yml uploads source tarball and HD2 hdpack

**Add two packaging tests asserting release.yml uploads source tarball and HD2 hdpack**

## What Happened

Appended two new `#[test]` functions to `tests/packaging.rs` following the existing `read_file` helper pattern:

1. `release_yml_uploads_source_tarball` — asserts that `.github/workflows/release.yml` contains both `git archive` and `vibe-attack-` (the tarball filename prefix), confirming the workflow produces and uploads a versioned source tarball.

2. `release_yml_uploads_hd2_hdpack` — asserts that the workflow contains both `profiles/hd2/pack.yaml` (the zip source) and `.hdpack` (the upload glob suffix), confirming the HD2 pack bundle step and upload are present.

Both tests use the same `read_file(".github/workflows/release.yml")` + `assert!(src.contains(...), "...; got:\n{src}")` pattern as the five existing tests. No existing tests or helpers were modified. The release.yml confirmed via T01 already contains all required strings (`git archive`, `vibe-attack-*.tar.gz` glob, `zip -j ... profiles/hd2/pack.yaml`, `hd2-*.hdpack` glob).

## Verification

Ran `cargo test --test packaging -- --test-threads=1` (mandatory single-thread per MEM005/MEM074). All 7 tests passed: 5 pre-existing + 2 new (`release_yml_uploads_source_tarball`, `release_yml_uploads_hd2_hdpack`). Verified with exit code 0 and "test result: ok. 7 passed" in output.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test packaging -- --test-threads=1 2>&1 | grep -q 'test result: ok. 7 passed'` | 0 | ✅ pass | 320ms |

## Deviations

The task plan specified asserting `vibe-attack-*.tar.gz` (the full glob), but since shell globs are not literal YAML strings, the workflow uses `vibe-attack-*.tar.gz` in the files block and `vibe-attack-${TAG}.tar.gz` in the run step. The test asserts `git archive` plus `vibe-attack-` prefix, which matches both occurrences robustly.

## Known Issues

none

## Files Created/Modified

- `tests/packaging.rs`
