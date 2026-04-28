# S01: AppImage build verification — Research

**Date:** 2026-04-27

## Summary

The AppImage packaging infrastructure is substantially complete and well-structured. `packaging/appimage/build.sh` handles the full pipeline: `cargo build --release --features gui`, AppDir layout, SVG-to-PNG icon conversion (three fallbacks: rsvg-convert, inkscape, convert), ORT `.so` bundling with a four-level discovery chain (`target/release/` → `ORT_DYLIB_PATH` env → `ldconfig` → `/usr` find), sherpa-onnx `.so` bundling (best-effort with warning if absent), AppRun script with `LD_LIBRARY_PATH` prepend, and conditional linuxdeploy + appimagetool invocation. The release CI workflow (`.github/workflows/release.yml`) runs on ubuntu-22.04 on tag push, installs system deps, downloads both AppImage tools, runs `build.sh` with `APPIMAGE_EXTRACT_AND_RUN=1`, renames the output with the tag, and uploads as a GitHub Release asset.

The dual-ORT conflict from M009 S07 is resolved: `sherpa-onnx` uses the `shared` feature so it links against the same `libonnxruntime.so` as the `ort` crate, and `build.sh` bundles both `.so` files. Two integration test suites cover the packaging artifacts statically (`tests/packaging.rs` — verifies build.sh structure and PKGBUILD content) and the first-run/distribution UI (`tests/ui_distribution.rs` — verifies `.desktop`, PKGBUILD fields, build.sh shebang/existence, and Cargo.toml feature gates). These tests run in CI (`cargo test --test packaging` and `cargo test --test ui_distribution`) and pass without hardware.

The primary gap for S01 is that `docs/distribution-proofs/appimage/` does not exist and no distro-level execution transcripts have been captured. The build.sh itself has two potential runtime issues: (1) it copies a second binary `vibe-attack-config` that requires `--features gui` but the `[[bin]]` definition has `required-features = ["gui"]` — this binary will be present because `build.sh` already passes `--features gui`; (2) the linuxdeploy invocation uses `--output appimage` but the final `appimagetool` call is run separately — linuxdeploy may produce its own AppImage before appimagetool runs, which could cause a naming conflict or double-packaging. This ordering should be verified.

## Recommendation

The approach is: (1) run `bash packaging/appimage/build.sh` locally to produce the AppImage (requires linuxdeploy + appimagetool installed), (2) test `./vibe-attack-x86_64.AppImage --version` locally and on Debian 12 / Fedora 39 / Arch VMs, (3) capture `--version` output transcripts into `docs/distribution-proofs/appimage/`, and (4) verify the release CI workflow actually fires and produces a tagged artifact by pushing a test tag (or reviewing the workflow logic). The existing static tests already pass; the gap is purely execution evidence. Do NOT restructure build.sh unless the linuxdeploy double-invocation is confirmed as a problem — the script is already production-quality.

## Implementation Landscape

### Key Files

- `packaging/appimage/build.sh` — complete AppImage build script; handles ORT/sherpa-onnx bundling, AppDir layout, AppRun creation, conditional linuxdeploy+appimagetool invocation
- `packaging/appimage/vibe-attack.desktop` — .desktop metadata (Name, Exec, Type, Icon, Categories); no MimeType; Terminal=false
- `.github/workflows/release.yml` — release CI: ubuntu-22.04, installs linuxdeploy + appimagetool from GitHub releases, runs build.sh, uploads AppImage to GH Release on tag push
- `.github/workflows/ci.yml` — PR/push CI: runs `cargo test --test packaging` and `cargo test --test ui_distribution`; does NOT build the AppImage
- `tests/packaging.rs` — static structure tests for build.sh (ORT bundling, AppRun LD_LIBRARY_PATH, linuxdeploy gate, ORT_DYLIB_PATH fallback) and PKGBUILD (onnxruntime dep)
- `tests/ui_distribution.rs` — static tests for .desktop fields, PKGBUILD required fields, build.sh shebang, Cargo.toml feature gate (default must not include gui)
- `Cargo.toml` — binary: `vibe-attack` (src/main.rs) + `vibe-attack-config` (src/bin/vibe-attack-config.rs, required-features=["gui"]); version 0.1.0; sherpa-onnx with `features=["shared"]` is the ORT conflict fix
- `src/main.rs` — clap-derived CLI with `version` attribute; `./vibe-attack --version` outputs `vibe-attack 0.1.0`
- `packaging/PKGBUILD` — AUR build file; `depends=('alsa-lib' 'onnxruntime')`; builds both binaries
- `packaging/debian/control` — Debian control; Build-Depends includes libclang-dev, libasound2-dev; Depends on libasound2
- `assets/vibe-attack.svg` — source icon; rsvg-convert/inkscape/ImageMagick convert it to PNG during build
- `docs/distribution-proofs/` — directory does NOT exist yet; `docs/distribution-proofs/appimage/` must be created to store per-distro transcripts

### Build Order

1. Confirm `cargo build --release --features gui` succeeds locally (prerequisite for everything).
2. Confirm `libonnxruntime.so` is discoverable — either in `target/release/` (placed by the `ort` crate build script) or via `ORT_DYLIB_PATH`.
3. Confirm `libsherpa-onnx-c-api.so` is discoverable (in `target/release/` after the sherpa-onnx-sys build, or via ldconfig).
4. Run `bash packaging/appimage/build.sh` — verify AppDir is populated and `vibe-attack-x86_64.AppImage` is produced.
5. Run `./vibe-attack-x86_64.AppImage --version` on the build host.
6. Repeat steps 4-5 in Debian 12, Fedora 39, and Arch VMs.
7. Capture transcripts into `docs/distribution-proofs/appimage/{debian12,fedora39,arch}/`.
8. Create `docs/distribution-proofs/appimage/` directory and commit transcripts.

### Verification Approach

- `cargo test --test packaging` — all 5 static tests must pass (already pass)
- `cargo test --test ui_distribution` — all tests covering .desktop, PKGBUILD, build.sh must pass (already pass)
- `bash packaging/appimage/build.sh 2>&1 | tee build.log` — must exit 0; must print "Done: vibe-attack-x86_64.AppImage"
- `./vibe-attack-x86_64.AppImage --version` must print `vibe-attack 0.1.0`
- `du -sh vibe-attack-x86_64.AppImage` — must be under 50 MB (M010 success criterion)
- On each target distro: `./vibe-attack-x86_64.AppImage --version` must succeed without installing any system packages
- Transcript format for proofs: stdout + stderr of `--version` invocation, distro name, kernel version, and AppImage size

## Constraints

- AppImage must run on Debian 12, Fedora 39, and Arch latest — these are the three mandated target distros (M010 success criteria)
- Final AppImage must be under 50 MB (M010 success criteria)
- `APPIMAGE_EXTRACT_AND_RUN=1` is required in CI because GitHub Actions runners do not have FUSE; this env var is set in release.yml already
- `libclang-dev` must be present at build time (clap derive + bindgen for sherpa-onnx-sys); this is installed in CI but may be absent on user machines
- `libasound2-dev` must be present at build time; bundled via linuxdeploy for runtime
- The sherpa-onnx `shared` feature is mandatory — it is what resolves the dual-ORT conflict (MEM: project_dual_ort_conflict.md)
- Clippy component may not be present in the local environment (MEM038); use `cargo build` as verification gate locally

## Common Pitfalls

- **APPIMAGE_EXTRACT_AND_RUN=1 missing** — without this, appimagetool/linuxdeploy fail silently in no-FUSE environments (CI, containers, some VMs); already set in release.yml but must be set manually when testing locally in a container
- **ORT .so not found** — the `ort` crate copies `libonnxruntime.so` to `target/release/` during a full release build, but only after `cargo build --release` completes; running build.sh before the cargo build finishes will trigger the exit-1 error path; always run `cargo build --release --features gui` to completion first
- **linuxdeploy double-packaging** — build.sh calls `linuxdeploy --appdir $APPDIR --output appimage` then separately calls `appimagetool $APPDIR`; linuxdeploy may already produce a `vibe-attack-x86_64.AppImage` before appimagetool runs; the final appimagetool call will overwrite it, which is correct behavior, but log output may be confusing
- **Icon missing** — if none of rsvg-convert, inkscape, or ImageMagick convert is installed, the PNG is not generated and the AppImage lacks an icon (warning only, not fatal); install `librsvg2-bin` (Debian) or `librsvg` (Arch) to avoid
- **vibe-attack-config missing** — build.sh copies `target/release/vibe-attack-config` but this binary is only built with `--features gui`; since build.sh passes `--features gui` this is fine, but if someone runs only `cargo build --release` (no gui) first, the copy step will fail
- **sherpa-onnx-c-api.so location** — the `sherpa-onnx` crate with the `shared` feature places the `.so` in `target/release/build/sherpa-onnx-sys-*/out/` or `target/release/`; build.sh's `find_so` searches `target/release/` first, then ldconfig, then falls back to a recursive find — if the build cache is cold, this may miss the file and emit a WARNING; a full `cargo build --release --features gui` should place it in `target/release/` via the build script
- **50 MB size limit** — the AppImage bundles ORT (~30 MB) + sherpa-onnx + whisper-rs (if stt feature) + alsa libs; omitting the `stt` feature from the AppImage build keeps size under budget; build.sh currently builds with `--features gui` only (no `stt`), which is correct for the base AppImage

## Open Risks

- **linuxdeploy strip behavior** — linuxdeploy strips debug symbols by default, which may interact poorly with Rust panic backtraces; may need `--plugin linuxdeploy-plugin-checkrt` if target glibc version on old Debian 12 is too old for the ubuntu-22.04 CI runner's glibc
- **glibc version mismatch** — ubuntu-22.04 uses glibc 2.35; Debian 12 uses glibc 2.36 (newer, OK); Fedora 39 uses glibc 2.38 (newer, OK); Arch is rolling (always newer); no downward glibc compat issue expected, but should be verified
- **FUSE availability on target distros** — test VMs must have `libfuse2` (for AppImage type-1) or `libfuse3` with the appimage-binfmt wrapper; Arch and Fedora may require explicit `fuse2` package install
- **AppImage size** — with ORT bundled, size may approach or exceed 50 MB; must measure early; if over budget, investigate `strip` on the binary or ORT compression
- **No `docs/distribution-proofs/appimage/` exists** — this directory and its per-distro transcript files must be created as new artifacts in S01; no template or format has been established yet
