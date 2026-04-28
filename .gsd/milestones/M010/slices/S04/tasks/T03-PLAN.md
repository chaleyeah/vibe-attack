---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T03: Document AUR submission workflow and pkgver/sha256sums pinning

Create `docs/distribution-proofs/aur/README.md` documenting the AUR submission process so any future maintainer (including the agent) can repeat it. The doc must include: (1) the pre-submission checklist — pin `pkgver` to the release tag (e.g. `0.1.0` matching git tag `v0.1.0`), compute and pin both `sha256sums` entries (the source tarball from `https://github.com/chaleyeah/vibe-attack/archive/v$pkgver.tar.gz` and the sherpa-onnx prebuilt archive from `https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.12.39/sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2`), provide the exact `sha256sum` and `updpkgsums` commands; (2) the verification checklist — `namcap PKGBUILD` clean, `makepkg -si` succeeds in a clean Arch chroot (`mkarchroot` or `extra-x86_64-build`), `makepkg --offline` succeeds after fetching sources once, runtime smoke test of the installed `vibe-attack --help` and `vibe-attack-config --help`; (3) the submission steps — clone or init the AUR repo at `ssh://aur@aur.archlinux.org/vibe-attack.git`, copy `packaging/PKGBUILD` and a generated `.SRCINFO` (`makepkg --printsrcinfo > .SRCINFO`), commit, push as maintainer `chaleyeah`; (4) a `STATUS:` field at the top using the same convention as `docs/distribution-proofs/appimage/<distro>/transcript.md` so the doc plays cleanly with the proof-transcript pattern (initial value `STATUS: pending submission` until a real submission occurs). Also update `.gsd/DECISIONS.md` only if a structural decision was made in T01 about the `onnxruntime` runtime dep — if T01 chose to remove onnxruntime, append a one-paragraph decision entry; if T01 kept it, no DECISIONS.md change. Verify by reading the file back and running `wc -l docs/distribution-proofs/aur/README.md` (must be > 30 lines).

## Inputs

- ``packaging/PKGBUILD` — referenced by the doc as the submission artifact`
- ``docs/distribution-proofs/appimage/debian12/transcript.md` — STATUS field convention to mirror`
- ``.gsd/DECISIONS.md` — appended only if T01 made a structural decision on onnxruntime`

## Expected Output

- ``docs/distribution-proofs/aur/README.md` — full AUR submission workflow doc with pre-submission, verification, and submission checklists, plus a STATUS field`
- ``.gsd/DECISIONS.md` — appended decision entry only if T01 modified the onnxruntime runtime dep; otherwise unchanged`

## Verification

test -f docs/distribution-proofs/aur/README.md && grep -q "makepkg" docs/distribution-proofs/aur/README.md && grep -q "namcap" docs/distribution-proofs/aur/README.md && grep -q "aur.archlinux.org" docs/distribution-proofs/aur/README.md && grep -q "STATUS:" docs/distribution-proofs/aur/README.md
