---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Fix PKGBUILD makedepends, sherpa-onnx offline source, and onnxruntime depends

Edit `packaging/PKGBUILD` to make it AUR-submission-ready apart from the release-tag-specific values (pkgver and sha256sums of the source tarball, which are pinned at release time in T03). Concretely: (1) add `clang` to `makedepends` because `bindgen`/`clang-sys` (transitively required) needs `libclang.so` at build time — this is already in the Debian and RPM packaging files; (2) add the sherpa-onnx 1.12.39 prebuilt archive as a second `source=` entry pointing at `https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.12.39/sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2` so makepkg fetches it during the source phase, not during cargo's build-time network call; (3) export `SHERPA_ONNX_ARCHIVE_DIR="$srcdir"` at the top of `build()` so the `sherpa-onnx-sys` build script picks the archive up locally and skips its network download — this is the documented escape hatch in `sherpa-onnx-sys/build.rs`; (4) audit the runtime `depends` array — sherpa-onnx with `features = ["shared"]` bundles `libonnxruntime.so` next to the binary via rpath `$ORIGIN` (S07 of M001 set this up). Either remove `onnxruntime` from `depends` (with a comment explaining the bundled `.so`) or keep it with a brief rationale comment. Pick one explicitly and document the choice. Use `'SKIP'` for the sherpa-onnx archive sha256 placeholder for now; T03 pins the real value. Do NOT change `pkgver` — keep `0.1.0` as a placeholder until the v0.1.0 tag is cut. Run `cargo build` afterwards to confirm nothing in-tree breaks (this verifies the file is well-formed, not that makepkg works — that is a release-time check).

## Inputs

- ``packaging/PKGBUILD` — current placeholder PKGBUILD with missing clang and no sherpa-onnx offline provision`
- ``packaging/debian/control` — confirms libclang-dev is needed (Build-Depends)`
- ``packaging/vibe-attack.spec` — confirms clang-devel is needed (BuildRequires)`
- ``Cargo.toml` — confirms sherpa-onnx 1.12.39 with shared feature`

## Expected Output

- ``packaging/PKGBUILD` — updated with clang in makedepends, sherpa-onnx archive as second source entry, SHERPA_ONNX_ARCHIVE_DIR exported in build(), and an explicit decision on the onnxruntime runtime dep with an inline comment`

## Verification

grep -q "^makedepends=.*clang" packaging/PKGBUILD && grep -q "sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2" packaging/PKGBUILD && grep -q "SHERPA_ONNX_ARCHIVE_DIR" packaging/PKGBUILD && cargo build --release

## Observability Impact

No runtime signals. Inspection: `cat packaging/PKGBUILD`. Failure visibility: at release time, missing clang surfaces as `bindgen` failures during `cargo build`; missing SHERPA_ONNX_ARCHIVE_DIR surfaces as a network connection in the build script; namcap will flag remaining issues.
