---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T02: Add packaging tests asserting release.yml uploads tarball and hdpack

Append two new `#[test]` functions to `tests/packaging.rs` following the existing `read_file` helper pattern: (1) `release_yml_uploads_source_tarball` — asserts `.github/workflows/release.yml` contains both `git archive` and the glob `vibe-attack-*.tar.gz` (or equivalent tarball file pattern) so the workflow demonstrably produces and uploads a source tarball; (2) `release_yml_uploads_hd2_hdpack` — asserts the workflow contains both a `zip` invocation referencing `profiles/hd2/pack.yaml` and the glob `hd2-*.hdpack`. Use the same `read_file("`.github/workflows/release.yml`")` style as the existing five tests; assert with `assert!(src.contains("..."), "...; got:\n{src}")` for parity. Do NOT change `read_file` or the existing five tests. Test names must be snake_case and live at the bottom of `tests/packaging.rs`. Run `cargo test --test packaging -- --test-threads=1` and confirm 7 tests pass (5 existing + 2 new). The `--test-threads=1` flag is mandatory per MEM005/MEM074 (shared-tmpdir flake prevention).

## Inputs

- ``tests/packaging.rs``
- ``.github/workflows/release.yml``

## Expected Output

- ``tests/packaging.rs``

## Verification

cargo test --test packaging -- --test-threads=1 2>&1 | grep -q 'test result: ok. 7 passed'
