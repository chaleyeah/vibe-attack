# S04: AUR PKGBUILD finalization and submission

**Goal:** Finalize packaging/PKGBUILD so it builds offline on a clean Arch chroot, passes namcap clean, and is ready for AUR submission as maintainer chaleyeah. Specifically: add `clang` to makedepends, pin the sherpa-onnx prebuilt archive as a second `source=` entry with a real sha256 (and set `SHERPA_ONNX_ARCHIVE_DIR="$srcdir"` in `build()`), audit whether `onnxruntime` belongs in `depends` given the bundled `libonnxruntime.so`, and document the release-time pkgver/sha256sums pinning workflow plus the AUR submission steps.
**Demo:** namcap clean; makepkg -si installs working binary; AUR package visible at aur.archlinux.org

## Must-Haves

- `packaging/PKGBUILD` contains `clang` in makedepends array
- `packaging/PKGBUILD` lists the sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2 archive as a second `source=` entry with a non-SKIP sha256
- `packaging/PKGBUILD` sets the `SHERPA_ONNX_ARCHIVE_DIR` env var in `build()` so cargo's sherpa-onnx-sys build script does not download from the network
- The runtime `depends` array reflects an explicit decision (either `onnxruntime` retained with rationale comment, or removed because sherpa-onnx bundles `libonnxruntime.so`)
- `tests/packaging.rs` has at least three new assertions covering the above (clang, sherpa archive source line, SHERPA_ONNX_ARCHIVE_DIR usage)
- `docs/distribution-proofs/aur/README.md` documents pre-submission steps (pin pkgver, compute sha256sums, run namcap, makepkg --offline) and submission steps (push to aur.archlinux.org:vibe-attack.git)
- `cargo test --test packaging` passes

## Proof Level

- This slice proves: Contract — this slice proves the PKGBUILD and supporting docs are submission-ready. Real `makepkg` runs against a clean Arch chroot (and the actual `git push` to AUR) happen at release time outside the CI environment available here. Static structural assertions enforce that the PKGBUILD has the right shape; the operational `makepkg -si` proof is the responsibility of S06 final UAT and the human maintainer at submission.

## Integration Closure

Upstream surfaces consumed: `Cargo.toml` (sherpa-onnx version 1.12.39, package version 0.1.0); `packaging/appimage/vibe-attack.desktop` (referenced verbatim by PKGBUILD); `assets/vibe-attack.svg`; `README.md`; `LICENSE`. New wiring: `SHERPA_ONNX_ARCHIVE_DIR` env var bridge between PKGBUILD `source=` array and `sherpa-onnx-sys` build script (so cargo skips its network download). What remains before the milestone is usable end-to-end: the actual `makepkg -si` clean-chroot run, namcap output capture, and `git push` to aur.archlinux.org happen at release time in S06 / by the human maintainer — they are deliberately out of scope here because no Arch chroot is available in the planning/execution environment.

## Verification

- Runtime signals: none (this slice modifies build-time packaging metadata only — no runtime code paths change). Inspection surfaces: `cat packaging/PKGBUILD`, `cargo test --test packaging`, `cat docs/distribution-proofs/aur/README.md`. Failure visibility: namcap output, makepkg stderr, and `cargo test --test packaging` failure output during pre-submission verification. Redaction: none — all artifacts are public packaging files.

## Tasks

- [x] **T01: Fix PKGBUILD makedepends, sherpa-onnx offline source, and onnxruntime depends** `est:1h`
  Edit `packaging/PKGBUILD` to make it AUR-submission-ready apart from the release-tag-specific values (pkgver and sha256sums of the source tarball, which are pinned at release time in T03). Concretely: (1) add `clang` to `makedepends` because `bindgen`/`clang-sys` (transitively required) needs `libclang.so` at build time — this is already in the Debian and RPM packaging files; (2) add the sherpa-onnx 1.12.39 prebuilt archive as a second `source=` entry pointing at `https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.12.39/sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2` so makepkg fetches it during the source phase, not during cargo's build-time network call; (3) export `SHERPA_ONNX_ARCHIVE_DIR="$srcdir"` at the top of `build()` so the `sherpa-onnx-sys` build script picks the archive up locally and skips its network download — this is the documented escape hatch in `sherpa-onnx-sys/build.rs`; (4) audit the runtime `depends` array — sherpa-onnx with `features = ["shared"]` bundles `libonnxruntime.so` next to the binary via rpath `$ORIGIN` (S07 of M001 set this up). Either remove `onnxruntime` from `depends` (with a comment explaining the bundled `.so`) or keep it with a brief rationale comment. Pick one explicitly and document the choice. Use `'SKIP'` for the sherpa-onnx archive sha256 placeholder for now; T03 pins the real value. Do NOT change `pkgver` — keep `0.1.0` as a placeholder until the v0.1.0 tag is cut. Run `cargo build` afterwards to confirm nothing in-tree breaks (this verifies the file is well-formed, not that makepkg works — that is a release-time check).
  - Files: `packaging/PKGBUILD`
  - Verify: grep -q "^makedepends=.*clang" packaging/PKGBUILD && grep -q "sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2" packaging/PKGBUILD && grep -q "SHERPA_ONNX_ARCHIVE_DIR" packaging/PKGBUILD && cargo build --release

- [x] **T02: Add packaging.rs assertions enforcing PKGBUILD AUR-readiness** `est:45m`
  Extend `tests/packaging.rs` with three new assertions that prove T01's edits stay in place: (1) `pkgbuild_has_clang_in_makedepends` — read `packaging/PKGBUILD` and assert the file contains `clang` inside the `makedepends=` array (use a regex or substring match, e.g., assert the file contains `'clang'` and `makedepends=` on a nearby line); (2) `pkgbuild_includes_sherpa_onnx_offline_source` — assert the file contains the substring `sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2` so the offline-build archive is wired in; (3) `pkgbuild_sets_sherpa_onnx_archive_dir` — assert the file contains `SHERPA_ONNX_ARCHIVE_DIR` so the cargo build script picks up the local archive. The existing `pkgbuild_declares_onnxruntime_runtime_dep` test must be updated or removed depending on the T01 decision: if T01 keeps `onnxruntime` in `depends`, leave the existing test alone; if T01 removes it, replace the test with `pkgbuild_documents_onnxruntime_decision` that asserts the file contains a comment line referencing onnxruntime + bundled to record the decision. Follow the style of existing tests in the file (use `read_file` helper, simple substring/regex). After editing, run `cargo test --test packaging` to confirm all assertions pass.
  - Files: `tests/packaging.rs`
  - Verify: cargo test --test packaging -- pkgbuild

- [x] **T03: Document AUR submission workflow and pkgver/sha256sums pinning** `est:1h`
  Create `docs/distribution-proofs/aur/README.md` documenting the AUR submission process so any future maintainer (including the agent) can repeat it. The doc must include: (1) the pre-submission checklist — pin `pkgver` to the release tag (e.g. `0.1.0` matching git tag `v0.1.0`), compute and pin both `sha256sums` entries (the source tarball from `https://github.com/chaleyeah/vibe-attack/archive/v$pkgver.tar.gz` and the sherpa-onnx prebuilt archive from `https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.12.39/sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2`), provide the exact `sha256sum` and `updpkgsums` commands; (2) the verification checklist — `namcap PKGBUILD` clean, `makepkg -si` succeeds in a clean Arch chroot (`mkarchroot` or `extra-x86_64-build`), `makepkg --offline` succeeds after fetching sources once, runtime smoke test of the installed `vibe-attack --help` and `vibe-attack-config --help`; (3) the submission steps — clone or init the AUR repo at `ssh://aur@aur.archlinux.org/vibe-attack.git`, copy `packaging/PKGBUILD` and a generated `.SRCINFO` (`makepkg --printsrcinfo > .SRCINFO`), commit, push as maintainer `chaleyeah`; (4) a `STATUS:` field at the top using the same convention as `docs/distribution-proofs/appimage/<distro>/transcript.md` so the doc plays cleanly with the proof-transcript pattern (initial value `STATUS: pending submission` until a real submission occurs). Also update `.gsd/DECISIONS.md` only if a structural decision was made in T01 about the `onnxruntime` runtime dep — if T01 chose to remove onnxruntime, append a one-paragraph decision entry; if T01 kept it, no DECISIONS.md change. Verify by reading the file back and running `wc -l docs/distribution-proofs/aur/README.md` (must be > 30 lines).
  - Files: `docs/distribution-proofs/aur/README.md`, `.gsd/DECISIONS.md`
  - Verify: test -f docs/distribution-proofs/aur/README.md && grep -q "makepkg" docs/distribution-proofs/aur/README.md && grep -q "namcap" docs/distribution-proofs/aur/README.md && grep -q "aur.archlinux.org" docs/distribution-proofs/aur/README.md && grep -q "STATUS:" docs/distribution-proofs/aur/README.md

## Files Likely Touched

- packaging/PKGBUILD
- tests/packaging.rs
- docs/distribution-proofs/aur/README.md
- .gsd/DECISIONS.md
