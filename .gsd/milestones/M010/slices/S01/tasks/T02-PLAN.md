---
estimated_steps: 1
estimated_files: 4
skills_used: []
---

# T02: Capture build-host transcript and seed Debian/Fedora/Arch proof directories

Create docs/distribution-proofs/appimage/{debian12,fedora39,arch}/ and populate each with a transcript.md. The directory matching the actual build host (whichever distro the runner is on — likely debian-derived Ubuntu) gets the real transcript captured by running scripts/verify-appimage.sh from T01. The other two directories get a STATUS: pending VM run transcript that includes: (a) the exact command sequence to reproduce (one-liner: `bash packaging/appimage/build.sh && bash scripts/verify-appimage.sh docs/distribution-proofs/appimage/<distro>/transcript.md`), (b) the required system packages for that distro (Fedora: alsa-lib-devel clang-devel librsvg2-tools fuse-libs; Arch: alsa-lib clang librsvg fuse2; Debian 12 mirrors release.yml), (c) STATUS: pending VM run, and (d) all metadata fields present (with `pending` placeholders) so tests/distribution_proofs.rs can assert structure regardless of completion state. Also add a top-level docs/distribution-proofs/appimage/README.md explaining the proof format, the per-distro reproduction procedure, and the policy that pending-VM-run transcripts are acceptable until a human or CI matrix completes the VM runs in S06.

## Inputs

- ``scripts/verify-appimage.sh` — created in T01; used to capture the real build-host transcript`
- ``packaging/appimage/build.sh` — referenced by per-distro reproduction commands in seed transcripts`

## Expected Output

- ``docs/distribution-proofs/appimage/README.md` — explains transcript format, STATUS values, and per-distro reproduction procedure`
- ``docs/distribution-proofs/appimage/debian12/transcript.md` — real build-host transcript OR seeded with STATUS: pending VM run (depending on host distro)`
- ``docs/distribution-proofs/appimage/fedora39/transcript.md` — seeded with STATUS: pending VM run plus per-distro reproduction one-liner`
- ``docs/distribution-proofs/appimage/arch/transcript.md` — seeded with STATUS: pending VM run plus per-distro reproduction one-liner`

## Verification

test -f docs/distribution-proofs/appimage/README.md && for d in debian12 fedora39 arch; do test -f docs/distribution-proofs/appimage/$d/transcript.md && grep -q '^STATUS: ' docs/distribution-proofs/appimage/$d/transcript.md && grep -q '^DISTRO: ' docs/distribution-proofs/appimage/$d/transcript.md || exit 1; done
