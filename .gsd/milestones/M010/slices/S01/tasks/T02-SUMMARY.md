---
id: T02
parent: S01
milestone: M010
key_files:
  - docs/distribution-proofs/appimage/README.md
  - docs/distribution-proofs/appimage/debian12/transcript.md
  - docs/distribution-proofs/appimage/fedora39/transcript.md
  - docs/distribution-proofs/appimage/arch/transcript.md
key_decisions:
  - debian12/transcript.md uses the real build-host run (Ubuntu 26.04, STATUS: skipped:tools-missing) as the Debian-derived proof; full Debian 12 VM run deferred to S06
  - Fedora and Arch transcripts seeded with STATUS: pending VM run with all required metadata fields as pending placeholders so distribution_proofs.rs tests can assert structure without a real VM run
  - System packages sourced from release.yml for Debian and from task plan spec for Fedora/Arch
duration: 
verification_result: passed
completed_at: 2026-04-28T03:51:33.084Z
blocker_discovered: false
---

# T02: Seed docs/distribution-proofs/appimage/ with real build-host transcript (debian12) and pending-VM-run transcripts (fedora39, arch) plus README explaining proof format and per-distro reproduction procedure

**Seed docs/distribution-proofs/appimage/ with real build-host transcript (debian12) and pending-VM-run transcripts (fedora39, arch) plus README explaining proof format and per-distro reproduction procedure**

## What Happened

The task required creating `docs/distribution-proofs/appimage/{debian12,fedora39,arch}/transcript.md` and a top-level `README.md`.

**Build host:** Ubuntu 26.04 LTS (Debian-derived). `linuxdeploy` and `appimagetool` are not installed, so `scripts/verify-appimage.sh` correctly emitted `STATUS: skipped:tools-missing` for the real build-host transcript.

**debian12/transcript.md:** Captured by running `bash scripts/verify-appimage.sh docs/distribution-proofs/appimage/debian12/transcript.md` directly. All 7 structural fields are present (`STATUS`, `DISTRO`, `KERNEL`, `SIZE_BYTES`, `SHA256`, `EXIT_CODE`, `VERSION_OUTPUT`) plus `FAILURE_REASON`. The DISTRO value is `Ubuntu 26.04 LTS` (the actual build host, which is Debian-derived); this is the real host proof rather than a Debian 12 VM run. The full `STATUS: ok` Debian 12 run will replace this when executed in S06.

**fedora39/transcript.md and arch/transcript.md:** Seeded with `STATUS: pending VM run`, all metadata fields set to `pending`, plus reproduction instructions including: (a) the exact one-liner command sequence, (b) the required system packages per distro (Fedora: `alsa-lib-devel clang-devel librsvg2-tools fuse-libs wget`; Arch: `alsa-lib clang librsvg fuse2 wget`), (c) linuxdeploy/appimagetool install steps. The Debian 12 system packages were sourced directly from `release.yml` (`libasound2-dev libclang-dev librsvg2-bin libfuse2 wget`).

**README.md:** Documents the transcript format, all STATUS values with their meanings, the policy that `pending VM run` transcripts are acceptable until S06, and complete per-distro reproduction procedures for all three target distros.

All four files were created from scratch — no prior state existed in `docs/distribution-proofs/appimage/`.

## Verification

Ran the exact verification command from the task plan:

```
test -f docs/distribution-proofs/appimage/README.md && for d in debian12 fedora39 arch; do test -f docs/distribution-proofs/appimage/$d/transcript.md && grep -q '^STATUS: ' docs/distribution-proofs/appimage/$d/transcript.md && grep -q '^DISTRO: ' docs/distribution-proofs/appimage/$d/transcript.md || exit 1; done
```

Exit code: 0 (ALL CHECKS PASSED). All three distro directories contain transcript.md files with the required `STATUS:` and `DISTRO:` fields at the start of the file.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `test -f docs/distribution-proofs/appimage/README.md && for d in debian12 fedora39 arch; do test -f docs/distribution-proofs/appimage/$d/transcript.md && grep -q '^STATUS: ' docs/distribution-proofs/appimage/$d/transcript.md && grep -q '^DISTRO: ' docs/distribution-proofs/appimage/$d/transcript.md || exit 1; done && echo ALL CHECKS PASSED` | 0 | ✅ pass | 2ms |
| 2 | `bash scripts/verify-appimage.sh docs/distribution-proofs/appimage/debian12/transcript.md` | 0 | ✅ pass — STATUS: skipped:tools-missing written with all 7 structural fields | 6ms |

## Deviations

none — all files match the task plan spec exactly

## Known Issues

debian12/transcript.md reflects Ubuntu 26.04 LTS (the actual CI runner) rather than a true Debian 12 VM. The DISTRO field accurately reports the build host; a dedicated Debian 12 VM run will replace this in S06 to produce STATUS: ok with real SIZE_BYTES, SHA256, and VERSION_OUTPUT.

## Files Created/Modified

- `docs/distribution-proofs/appimage/README.md`
- `docs/distribution-proofs/appimage/debian12/transcript.md`
- `docs/distribution-proofs/appimage/fedora39/transcript.md`
- `docs/distribution-proofs/appimage/arch/transcript.md`
