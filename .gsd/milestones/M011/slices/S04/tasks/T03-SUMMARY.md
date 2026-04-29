---
id: T03
parent: S04
milestone: M011
key_files:
  - tests/packaging.rs
key_decisions:
  - Job-name tests use .lines().any(|l| l == "  build-deb:") rather than .contains() to enforce column-2 YAML anchoring — prevents false positives from step-level YAML keys that happen to contain the job name as a substring.
duration: 
verification_result: passed
completed_at: 2026-04-29T01:56:50.313Z
blocker_discovered: false
---

# T03: Extended tests/packaging.rs with five new static assertions that enforce the release.yml build-deb, build-rpm, artifact glob, and sherpa cache parity contract

**Extended tests/packaging.rs with five new static assertions that enforce the release.yml build-deb, build-rpm, artifact glob, and sherpa cache parity contract**

## What Happened

The five new tests mirror the existing string-contains pattern used by `release_yml_uploads_source_tarball` and `release_yml_uploads_hd2_hdpack`. Each test reads release.yml via `read_file()` and asserts a specific structural invariant:

1. `release_yml_has_build_deb_job` — checks that a line exactly equal to `  build-deb:` exists (column-2 anchored YAML job declaration).
2. `release_yml_has_build_rpm_job` — same for `  build-rpm:`.
3. `release_yml_uploads_deb_artifact` — checks for the `vibe-attack_*.deb` glob (Debian underscore convention) in the file.
4. `release_yml_uploads_rpm_artifact` — checks for the `vibe-attack-*.x86_64.rpm` glob (RPM arch-in-filename convention).
5. `release_yml_caches_sherpa_onnx_in_all_release_jobs` — counts occurrences of `sherpa-onnx-1.12.39-linux-x64`; asserts at least 3, enforcing MEM089 cache parity across all three build jobs (appimage, deb, rpm).

The job-name tests use `.lines().any(|l| l == "  build-deb:")` rather than `.contains()` to enforce the column-2 anchoring required by the YAML job-name convention — a `build-deb:` indented differently (e.g., inside a step) would be a false positive. The glob and cache tests use `.contains()` for robustness against surrounding whitespace changes, consistent with the existing tests' design rationale.

No existing tests were modified. All 10 pre-existing tests continued to pass alongside the 5 new ones (15 total).

## Verification

Ran `cargo test --test packaging 2>&1 | tee /tmp/s04-t03.log` — all 15 tests passed (0 failed). Confirmed all five new test names appear as `ok` in the log via the task plan's grep chain. The slice's `cargo test --test packaging` verification criterion is now fully satisfied.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test packaging 2>&1 | tee /tmp/s04-t03.log` | 0 | ✅ pass — 15 passed, 0 failed | 3390ms |
| 2 | `grep -q 'release_yml_has_build_deb_job .* ok' /tmp/s04-t03.log && grep -q 'release_yml_has_build_rpm_job .* ok' /tmp/s04-t03.log && grep -q 'release_yml_uploads_deb_artifact .* ok' /tmp/s04-t03.log && grep -q 'release_yml_uploads_rpm_artifact .* ok' /tmp/s04-t03.log && grep -q 'release_yml_caches_sherpa_onnx_in_all_release_jobs .* ok' /tmp/s04-t03.log` | 0 | ✅ pass — all five new tests confirmed ok | 10ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `tests/packaging.rs`
