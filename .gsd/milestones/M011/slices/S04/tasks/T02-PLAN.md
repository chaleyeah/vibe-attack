---
estimated_steps: 25
estimated_files: 1
skills_used: []
---

# T02: Add build-deb and build-rpm jobs to release.yml and extend artifact upload globs

Add two new jobs to `.github/workflows/release.yml`, parallel to the existing `build-appimage` job. Both run on `ubuntu-22.04`, both reuse the exact sherpa-onnx cache block (path: `target/sherpa-onnx-prebuilt`, key: `sherpa-onnx-1.12.39-linux-x64`) per MEM089 — copy the block verbatim from `build-appimage`. Then extend the existing upload step (or migrate it into a small `release` job that depends on all three build jobs).

Approach for the .deb job (`build-deb`):
  1. Checkout, install Rust, rust-cache, sherpa cache + conditional rebuild (MEM089 parity).
  2. apt-get install: `libasound2-dev libclang-dev devscripts debhelper dh-cargo` (devscripts/debhelper provide `dpkg-buildpackage` and `dh`; existing `packaging/debian/rules` already uses dh and cargo build).
  3. Run `dpkg-buildpackage -uc -us -b` from repo root. This emits `vibe-attack_1.0.0-1_amd64.deb` into the parent directory (Debian convention) — move it back into the workflow workspace.
  4. `actions/upload-artifact@v4` with name `deb` so the final upload-release step can pull it.

Approach for the .rpm job (`build-rpm`):
  1. Checkout, install Rust, rust-cache, sherpa cache + conditional rebuild (MEM089 parity).
  2. apt-get install: `libasound2-dev libclang-dev rpm` (the `rpm` package on ubuntu-22.04 provides `rpmbuild`).
  3. Set up rpmbuild tree: `mkdir -p ~/rpmbuild/{SOURCES,SPECS,BUILD,RPMS,SRPMS}`. The .spec uses `%autosetup` which expects a tarball matching `Source0`. Create the source tarball locally with `git archive --format=tar.gz --prefix=vibe-attack-1.0.0/ HEAD -o ~/rpmbuild/SOURCES/vibe-attack-1.0.0.tar.gz`. Copy `packaging/vibe-attack.spec` to `~/rpmbuild/SPECS/`.
  4. Run `rpmbuild -bb ~/rpmbuild/SPECS/vibe-attack.spec`. This produces `vibe-attack-1.0.0-1.x86_64.rpm` in `~/rpmbuild/RPMS/x86_64/`. Copy it back to the workspace.
  5. `actions/upload-artifact@v4` with name `rpm`.

Then modify the existing upload-release step (currently inside `build-appimage`):
  - The cleanest refactor is to rename the upload step's job to a new `release` job that `needs: [build-appimage, build-deb, build-rpm]`. Use `actions/download-artifact@v4` to fetch all three artifact bundles, then run the existing `softprops/action-gh-release@v2` step with these explicit globs (newline-separated, fail_on_unmatched_files: true per MEM086):
      vibe-attack-*-x86_64.AppImage
      vibe-attack-*.tar.gz
      hd2-*.hdpack
      vibe-attack_*.deb
      vibe-attack-*.x86_64.rpm
  - The AppImage build job must `actions/upload-artifact@v4` its outputs (AppImage, tarball, hdpack) so the release job can collect them.

Alternative if the refactor is too invasive: keep `build-appimage` as the sole uploader and have build-deb/build-rpm `actions/upload-artifact` to it via job needs + `download-artifact` step inserted before the `softprops/action-gh-release@v2` step. Either approach is fine — the key invariants are (a) all three jobs cache sherpa-onnx with the same key, (b) the final upload step uses explicit globs with `fail_on_unmatched_files: true`, (c) AppImage globs continue to match.

Why: this is the slice's primary deliverable — without these jobs, S05 cannot publish .deb or .rpm artifacts. Sharing the sherpa-onnx cache key across all three release jobs is critical to keep CI under the 60-min job budget; otherwise each job triggers a fresh sherpa-onnx-sys download and the combined runtime spikes.

Failure modes (Q5): rpmbuild's `%autosetup` will fail if the source tarball name doesn't match the spec's Source0 expansion (`vibe-attack-1.0.0.tar.gz`). dpkg-buildpackage will fail if libclang-dev is missing (clang is a transitive dep of sherpa-onnx-sys per MEM092). softprops/action-gh-release with `fail_on_unmatched_files: true` will fail loudly if any glob matches zero files — desirable per MEM086.
Load profile (Q6): one tag push triggers all three jobs in parallel; with sherpa cache hit each job is ~10–15 min; cold-cache first run could be 30–40 min (MEM089). GitHub Actions' default 6-hour job timeout is sufficient.
Negative tests (Q7): not exercised in this task — the unit-level assertion is in T03 (does the workflow YAML contain the right job names and globs?). A real tag-push validation is S05's responsibility per MEM111.

## Inputs

- ``.github/workflows/release.yml``
- ``packaging/vibe-attack.spec``
- ``packaging/debian/rules``
- ``packaging/debian/control``
- ``Cargo.toml``

## Expected Output

- ``.github/workflows/release.yml``

## Verification

yamllint .github/workflows/release.yml 2>/dev/null || python3 -c 'import yaml; yaml.safe_load(open(".github/workflows/release.yml"))' && grep -q '^  build-deb:' .github/workflows/release.yml && grep -q '^  build-rpm:' .github/workflows/release.yml && grep -q 'vibe-attack_\*\.deb' .github/workflows/release.yml && grep -q 'vibe-attack-\*\.x86_64\.rpm' .github/workflows/release.yml && grep -q 'fail_on_unmatched_files: true' .github/workflows/release.yml && grep -c 'sherpa-onnx-1.12.39-linux-x64' .github/workflows/release.yml | awk '{ exit ($1 < 3) }'

## Observability Impact

Adds two new GitHub Actions job names (`build-deb`, `build-rpm`) visible in the Actions UI and in PR checks. `fail_on_unmatched_files: true` in the upload step (MEM086) ensures missing artifacts fail the release upload loudly rather than producing a partial release — critical failure-visibility signal for S05's tag push.
