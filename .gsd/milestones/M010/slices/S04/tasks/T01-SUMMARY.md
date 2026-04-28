---
id: T01
parent: S04
milestone: M010
key_files:
  - packaging/PKGBUILD
key_decisions:
  - Keep onnxruntime in depends: libsherpa-onnx-c-api.so has RPATH=$ORIGIN and NEEDED:libonnxruntime.so, but $ORIGIN only resolves when both .so files are co-located (AppImage). In the Arch native package only binaries land in /usr/bin/, so system onnxruntime package is required.
  - Use SKIP placeholder for sherpa-onnx archive sha256sum: will be pinned with real hash in T03 as part of the release-time workflow.
duration: 
verification_result: passed
completed_at: 2026-04-28T04:19:16.498Z
blocker_discovered: false
---

# T01: Fix PKGBUILD makedepends (add clang), add sherpa-onnx 1.12.39 prebuilt archive as second source entry with SHERPA_ONNX_ARCHIVE_DIR export, and document onnxruntime depends decision with inline comment

**Fix PKGBUILD makedepends (add clang), add sherpa-onnx 1.12.39 prebuilt archive as second source entry with SHERPA_ONNX_ARCHIVE_DIR export, and document onnxruntime depends decision with inline comment**

## What Happened

Read the existing PKGBUILD and compared against the Debian control and RPM spec files to confirm the missing deps. Three changes were made to `packaging/PKGBUILD`:

1. **`clang` added to `makedepends`**: `bindgen`/`clang-sys` (transitive deps of `sherpa-onnx-sys`) require `libclang.so` at build time. The Debian `Build-Depends` has `libclang-dev` and the RPM spec has `clang-devel` — the Arch PKGBUILD was the only packaging file missing this.

2. **sherpa-onnx prebuilt archive added as `source[1]`**: Points to `https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.12.39/sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2`. The `sherpa-onnx-sys` build.rs (confirmed in `~/.cargo/registry/src/.../sherpa-onnx-sys-1.12.39/build.rs`) checks `SHERPA_ONNX_ARCHIVE_DIR` as an escape hatch before attempting a network download. Without this, makepkg would try to download during `cargo build` inside the build sandbox, which fails in clean chroot environments. `sha256sums` for this entry is set to `'SKIP'` as a placeholder; T03 pins the real value.

3. **`export SHERPA_ONNX_ARCHIVE_DIR="$srcdir"` in `build()`**: Tells the `sherpa-onnx-sys` build script to look in `$srcdir` (where makepkg unpacks the second source) for the archive, bypassing the network call entirely.

4. **`onnxruntime` kept in `depends` with explanatory comment**: Audited the binary via `readelf -d`. The main `vibe-attack` binary only has `NEEDED: libsherpa-onnx-c-api.so`, but `libsherpa-onnx-c-api.so` itself has `NEEDED: libonnxruntime.so` with `RPATH: $ORIGIN`. The `$ORIGIN` rpath means at runtime the dynamic linker looks for `libonnxruntime.so` in the same directory as the `.so` file itself. In the AppImage, both `.so` files land together in `usr/lib/` so `$ORIGIN` resolves correctly. But in the native Arch package, only the two binaries are installed to `/usr/bin/` — `libsherpa-onnx-c-api.so` and `libonnxruntime.so` are NOT installed, so the `$ORIGIN` trick does not apply. The system `onnxruntime` package (which provides `/usr/lib/libonnxruntime.so`) is therefore a genuine runtime dependency and must remain in `depends`. An inline comment explains this distinction.

## Verification

Ran three grep checks from the task plan verification command, all passing:
- `grep -q "^makedepends=.*clang" packaging/PKGBUILD` → exit 0
- `grep -q "sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2" packaging/PKGBUILD` → exit 0
- `grep -q "SHERPA_ONNX_ARCHIVE_DIR" packaging/PKGBUILD` → exit 0

Then ran `cargo build --release` which completed successfully in ~4s (reusing cached artifacts), confirming the PKGBUILD edits introduce no in-tree build breakage.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `grep -q "^makedepends=.*clang" packaging/PKGBUILD` | 0 | ✅ pass | 20ms |
| 2 | `grep -q "sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2" packaging/PKGBUILD` | 0 | ✅ pass | 15ms |
| 3 | `grep -q "SHERPA_ONNX_ARCHIVE_DIR" packaging/PKGBUILD` | 0 | ✅ pass | 15ms |
| 4 | `cargo build --release` | 0 | ✅ pass | 3700ms |

## Deviations

none

## Known Issues

sha256sums for both source entries remain 'SKIP' — T03 is responsible for pinning the real values at release time.

## Files Created/Modified

- `packaging/PKGBUILD`
