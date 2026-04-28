# S04: AUR PKGBUILD finalization and submission — Research

**Date:** 2026-04-27

## Summary

The current `packaging/PKGBUILD` is a first-draft skeleton: `pkgver=0.1.0` and `sha256sums=('SKIP')` are placeholders that must be replaced with the actual release tag and a computed sha256 before submission. The `makedepends` array only lists `('rust' 'cargo')` — it is **missing `clang`**, which is required at build time because `bindgen` (pulled in by `whisper-rs-sys`, an optional dep via the `stt` feature) uses `clang-sys`. The RPM spec (`packaging/vibe-attack.spec`) and Debian control (`packaging/debian/control`) both already include the clang build dep (`clang-devel` and `libclang-dev` respectively), confirming this is a known gap specific to the PKGBUILD.

The most critical offline-build risk is `sherpa-onnx-sys`'s build script, which **downloads a prebuilt `.tar.bz2` archive from GitHub releases at build time** (`https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.12.39/sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2`). The build script supports two env vars to bypass this download: `SHERPA_ONNX_LIB_DIR` (pointing to a pre-extracted `lib/` directory) and `SHERPA_ONNX_ARCHIVE_DIR` (pointing to a directory containing the `.tar.bz2` archive). Without one of these being set, `makepkg --offline` will fail because the build step hits the network. The PKGBUILD must either pre-fetch and include the archive as a second `source=` entry, or set `SHERPA_ONNX_ARCHIVE_DIR` to `$srcdir` in the `build()` function.

The `build()` function runs `cargo build --release --locked` twice (once without `--features gui`, once with `--features gui`). The `--locked` flag is correct and requires a `Cargo.lock` to be present in the source tarball — this is standard. The `package()` function installs both binaries (`vibe-attack` and `vibe-attack-config`), desktop entry, SVG icon, README, and LICENSE. The desktop entry at `packaging/appimage/vibe-attack.desktop` is correctly pathed. The `depends` array lists `alsa-lib` and `onnxruntime` — however, with `sherpa-onnx` using `features = ["shared"]`, the shared `libonnxruntime.so` is bundled inside the `sherpa-onnx` prebuilt archive and copied next to the binary at build time (rpath `$ORIGIN`). This means the AUR `onnxruntime` package in `depends` may be redundant or conflicting — needs verification.

## Recommendation

Fix the PKGBUILD in this order:

1. **Add `clang` to `makedepends`**: `makedepends=('rust' 'cargo' 'clang')`. This is required by `bindgen`/`clang-sys` (via `whisper-rs-sys` in the `stt` optional feature and potentially other paths).

2. **Solve the sherpa-onnx offline download**: The cleanest AUR-compliant approach is to add the sherpa-onnx prebuilt archive as a second `source=` entry so `makepkg` downloads it during the fetch phase (not build phase), and set `SHERPA_ONNX_ARCHIVE_DIR="$srcdir"` in `build()`. The archive URL is `https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.12.39/sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2`. The sha256 must be computed against this file.

3. **Re-evaluate the `onnxruntime` runtime dep**: Since `sherpa-onnx` bundles `libonnxruntime.so` next to the binary (rpath `$ORIGIN`), the AUR `onnxruntime` package may not be needed at runtime. If the bundled `.so` is truly self-contained, remove `onnxruntime` from `depends` to avoid a false dependency that could cause namcap warnings or version conflicts.

4. **Pin `pkgver` and compute `sha256sums`**: Replace `0.1.0` with the actual release version and replace `'SKIP'` with the real sha256 of the source tarball + the sherpa-onnx archive.

5. **Run `namcap PKGBUILD`** and **`namcap vibe-attack-*.pkg.tar.zst`** to surface any remaining issues before submission.

6. **Submit to AUR** as maintainer `chaleyeah` via `git push` to `aur.archlinux.org:vibe-attack.git`.

## Implementation Landscape

### Key Files

- `packaging/PKGBUILD` — Current state: `pkgver=0.1.0` (placeholder), `sha256sums=('SKIP')` (placeholder), `makedepends=('rust' 'cargo')` (missing `clang`), `depends=('alsa-lib' 'onnxruntime')` (onnxruntime dep may be unnecessary given shared bundling), no provision for sherpa-onnx offline build. Build function calls `cargo build --release --locked` twice (base + gui feature). Package function installs 2 binaries, desktop entry, icon, docs, license.
- `Cargo.toml` — Package `vibe-attack`, version `0.1.0`. Key deps: `sherpa-onnx = { version = "1.12.39", features = ["shared"] }`, `ort = "=2.0.0-rc.10"`, `silero-vad-rust = "6.2.1"`. Optional: `whisper-rs = "0.16.0"` (feature `stt`). GUI feature: `eframe`, `ureq`, `ksni`, `rfd`.
- `packaging/appimage/vibe-attack.desktop` — Desktop entry file referenced by PKGBUILD; correct path.
- `packaging/debian/control` — Has `libclang-dev` in `Build-Depends`; confirms clang is a known build requirement.
- `packaging/vibe-attack.spec` — Has `clang-devel` in `BuildRequires`; same confirmation.
- `/home/chadmin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sherpa-onnx-sys-1.12.39/build.rs` — Build script that downloads prebuilt libs. Respects `SHERPA_ONNX_LIB_DIR` and `SHERPA_ONNX_ARCHIVE_DIR` env vars. For shared mode on Linux x86_64, downloads `sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2`.

### Build Order

1. Add `clang` to `makedepends` in PKGBUILD
2. Add sherpa-onnx prebuilt archive as a `source=` entry with sha256; set `SHERPA_ONNX_ARCHIVE_DIR="$srcdir"` in `build()`
3. Decide on `onnxruntime` in `depends` (test whether bundled `.so` is sufficient at runtime; if yes, remove)
4. Replace `pkgver` and `sha256sums` placeholders once a release tag is cut (or test with a dev tarball)
5. Build locally with `makepkg -si` in a clean chroot or with `--syncdeps`
6. Run `namcap PKGBUILD` and `namcap *.pkg.tar.zst`
7. Test offline: `makepkg --nobuild` followed by `makepkg --noextract --noprepare` with network blocked
8. Push to AUR

### Verification Approach

- `namcap PKGBUILD` — catches missing deps, bad install paths, wrong permissions
- `namcap vibe-attack-*.pkg.tar.zst` — post-build package checks: linked libraries, missing deps
- `makepkg -si` in a clean Arch chroot (e.g., `mkarchroot` or `extra-x86_64-build`) — proves dependencies are complete
- `makepkg --offline` test (after pre-downloading sources): proves network is not needed during build phase
- Runtime smoke test: `vibe-attack --help` and `vibe-attack-config --help` from installed package

## Constraints

- `makepkg --offline` must succeed (no network during build phase) — requires sherpa-onnx archive to be pre-fetched as a `source=` entry
- `clang` must be in `makedepends` (sherpa-onnx/bindgen requires it at build time)
- `Cargo.lock` must be included in the source tarball (PKGBUILD uses `--locked`)
- AUR submission requires maintainer `chaleyeah` to push to `aur.archlinux.org:vibe-attack.git`

## Common Pitfalls

- **Missing `clang` in makedepends** — `bindgen` (via `whisper-rs-sys`) uses `clang-sys`, which requires `libclang.so` at build time; namcap may or may not catch this automatically since it's a build-time dep
- **sherpa-onnx network download during `cargo build`** — the build script silently hits GitHub releases unless `SHERPA_ONNX_LIB_DIR` or `SHERPA_ONNX_ARCHIVE_DIR` is set; `makepkg --offline` will fail without this fix
- **sha256sum placeholders** — `'SKIP'` is fine for local testing but must be pinned to actual checksums before AUR submission (AUR CI/namcap will flag `SKIP`)
- **Double `onnxruntime` linkage** — the bundled `libonnxruntime.so` from sherpa-onnx (rpath `$ORIGIN`) and the system `onnxruntime` package could conflict; the existing `ORT_DYLIB_PATH` auto-discovery in `coordinator.rs` points to `libonnxruntime.so` next to the binary, suggesting the bundled one is intended
- **`whisper-rs` is optional** — `stt` feature is not enabled in the `build()` function's `cargo build --release --locked` invocation; if it were, `clang` would be mandatory. Even without `stt`, `clang` may still be required depending on transitive deps — verify with a clean build.

## Open Risks

- **AUR review timing** — submission requires a release tag to exist on GitHub; `pkgver=0.1.0` must match a real `v0.1.0` tag with a corresponding tarball
- **sherpa-onnx archive sha256** — the archive must be downloaded once to compute sha256; this is a one-time manual step before submission
- **`onnxruntime` AUR package version skew** — if the AUR `onnxruntime` package version does not match the one bundled by sherpa-onnx-sys 1.12.39, runtime conflicts could arise; the bundled approach (removing `onnxruntime` from `depends`) is safer
- **namcap false negatives** — namcap does not always catch missing build-time deps (it focuses on installed package); a clean chroot build is the definitive test
- **`clang` vs `clang-sys` without stt feature** — if the `stt` feature is not enabled in the PKGBUILD `build()` command, `whisper-rs-sys` and its `bindgen`/`clang-sys` dep may not be compiled; however, `coreaudio-sys` also depends on `bindgen` (macOS only, likely filtered). Audit the full transitive dep tree on a clean Arch build to confirm whether `clang` is needed for the non-stt build path.
