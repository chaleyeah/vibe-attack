---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T02: Add packaging.rs assertions enforcing PKGBUILD AUR-readiness

Extend `tests/packaging.rs` with three new assertions that prove T01's edits stay in place: (1) `pkgbuild_has_clang_in_makedepends` — read `packaging/PKGBUILD` and assert the file contains `clang` inside the `makedepends=` array (use a regex or substring match, e.g., assert the file contains `'clang'` and `makedepends=` on a nearby line); (2) `pkgbuild_includes_sherpa_onnx_offline_source` — assert the file contains the substring `sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2` so the offline-build archive is wired in; (3) `pkgbuild_sets_sherpa_onnx_archive_dir` — assert the file contains `SHERPA_ONNX_ARCHIVE_DIR` so the cargo build script picks up the local archive. The existing `pkgbuild_declares_onnxruntime_runtime_dep` test must be updated or removed depending on the T01 decision: if T01 keeps `onnxruntime` in `depends`, leave the existing test alone; if T01 removes it, replace the test with `pkgbuild_documents_onnxruntime_decision` that asserts the file contains a comment line referencing onnxruntime + bundled to record the decision. Follow the style of existing tests in the file (use `read_file` helper, simple substring/regex). After editing, run `cargo test --test packaging` to confirm all assertions pass.

## Inputs

- ``packaging/PKGBUILD` — updated by T01; tests assert against this file's contents`
- ``tests/packaging.rs` — existing static packaging tests; new assertions follow the same pattern`

## Expected Output

- ``tests/packaging.rs` — three new tests (pkgbuild_has_clang_in_makedepends, pkgbuild_includes_sherpa_onnx_offline_source, pkgbuild_sets_sherpa_onnx_archive_dir) plus updated/replaced onnxruntime test consistent with the T01 decision`

## Verification

cargo test --test packaging -- pkgbuild
