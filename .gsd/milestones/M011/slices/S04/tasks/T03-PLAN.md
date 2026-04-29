---
estimated_steps: 13
estimated_files: 1
skills_used: []
---

# T03: Extend tests/packaging.rs to assert release.yml declares build-deb, build-rpm, and matching artifact globs

Add new static assertions to `tests/packaging.rs` that mechanically verify the release.yml contract. These tests run with `cargo test --test packaging` and require no build tools — they read the YAML as a string and grep for required substrings. This pattern matches the existing `release_yml_uploads_source_tarball` and `release_yml_uploads_hd2_hdpack` tests and the precedent set by MEM103.

Add these tests:
  1. `release_yml_has_build_deb_job` — assert the file contains `^  build-deb:` (anchored at column 2 — YAML job declaration).
  2. `release_yml_has_build_rpm_job` — assert the file contains `^  build-rpm:`.
  3. `release_yml_uploads_deb_artifact` — assert the upload block contains a glob matching `vibe-attack_*.deb` (Debian uses underscores in filenames).
  4. `release_yml_uploads_rpm_artifact` — assert the upload block contains a glob matching `vibe-attack-*.x86_64.rpm` (RPM convention with hyphens and arch in filename).
  5. `release_yml_caches_sherpa_onnx_in_all_release_jobs` — assert the sherpa cache key `sherpa-onnx-1.12.39-linux-x64` appears at least 3 times in release.yml (once per build job: appimage, deb, rpm) — this enforces MEM089 parity at PR-review time rather than discovering it at release time.

Use the same `read_file()` helper already in tests/packaging.rs. Use simple `.contains()` and `.matches().count()` checks; do not pull in a YAML parser — the existing tests intentionally use string contains for robustness against YAML formatting changes.

Also bump the existing `release_yml_uploads_source_tarball` assertion only if needed (current pattern likely still passes after T02). Do NOT delete or rename existing tests — they are the contract for AppImage + tarball + hdpack and must continue to assert.

Why: this is the slice's automated stopping condition. The slice cannot rely on a real tag-push for verification (that's S05's domain per MEM111). Static contract tests catch missing jobs/globs at PR time, exactly mirroring the rationale for MEM086 and MEM103. Without these tests, a future refactor to release.yml could silently drop one of the new jobs and ship a broken release.

Failure modes (Q5): if T02 named the jobs differently (e.g. `deb-build` instead of `build-deb`) these tests will fail — desirable; failure points at the exact contract violation. If MEM093's SKIP-sums policy were ever relaxed and PKGBUILD sha256sums changed, the existing PKGBUILD tests would not catch that — out of scope for this slice.
Load profile (Q6): tests are pure string reads, sub-second; no I/O concerns.
Negative tests (Q7): not applicable — these are positive-presence assertions.

## Inputs

- ``tests/packaging.rs``
- ``.github/workflows/release.yml``

## Expected Output

- ``tests/packaging.rs``

## Verification

cargo test --test packaging 2>&1 | tee /tmp/s04-t03.log && grep -q 'release_yml_has_build_deb_job .* ok' /tmp/s04-t03.log && grep -q 'release_yml_has_build_rpm_job .* ok' /tmp/s04-t03.log && grep -q 'release_yml_uploads_deb_artifact .* ok' /tmp/s04-t03.log && grep -q 'release_yml_uploads_rpm_artifact .* ok' /tmp/s04-t03.log && grep -q 'release_yml_caches_sherpa_onnx_in_all_release_jobs .* ok' /tmp/s04-t03.log

## Observability Impact

These tests are themselves an observability surface — they make the workflow contract inspectable and assertable in CI without needing a real tag push. A future agent debugging release-pipeline drift can run `cargo test --test packaging` locally and see immediately which contract clause is broken (missing job, missing glob, cache parity drift).
