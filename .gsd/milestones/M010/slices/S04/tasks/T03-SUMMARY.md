---
id: T03
parent: S04
milestone: M010
key_files:
  - docs/distribution-proofs/aur/README.md
key_decisions:
  - DECISIONS.md left unchanged: T01 kept onnxruntime in depends (no structural removal), so the task plan condition for appending a decision entry was not met
duration: 
verification_result: passed
completed_at: 2026-04-28T04:21:38.924Z
blocker_discovered: false
---

# T03: Create docs/distribution-proofs/aur/README.md with full AUR submission workflow: sha256 pinning commands, namcap/chroot verification checklist, and push-to-AUR steps for maintainer chaleyeah

**Create docs/distribution-proofs/aur/README.md with full AUR submission workflow: sha256 pinning commands, namcap/chroot verification checklist, and push-to-AUR steps for maintainer chaleyeah**

## What Happened

Read T01-SUMMARY to confirm onnxruntime was kept in depends (not removed), so no DECISIONS.md entry was needed. Read the existing PKGBUILD (with SKIP placeholders for sha256sums) and the STATUS field convention from docs/distribution-proofs/appimage/debian12/transcript.md.

Created docs/distribution-proofs/aur/README.md (204 lines) with four sections:

1. **Pre-submission checklist**: Pin pkgver to release tag (no `v` prefix), compute sha256 for both sources using `sha256sum` + curl commands and/or `updpkgsums`, with explicit examples for source[0] (project tarball from github.com/chaleyeah/vibe-attack) and source[1] (sherpa-onnx 1.12.39 prebuilt archive from k2-fsa/sherpa-onnx).

2. **Verification checklist**: `namcap PKGBUILD` clean pass, `extra-x86_64-build` in clean Arch chroot, `makepkg --offline` after initial fetch, runtime smoke test (`vibe-attack --help`, `vibe-attack-config --help`).

3. **AUR submission steps**: SSH config for aur.archlinux.org, `git clone ssh://aur@aur.archlinux.org/vibe-attack.git` for first-time setup, copy PKGBUILD, generate `.SRCINFO` via `makepkg --printsrcinfo`, commit and push as chaleyeah.

4. **Notes**: Explains the onnxruntime runtime dependency rationale (RPATH=$ORIGIN only works in AppImage; native Arch package needs system onnxruntime) and the SHERPA_ONNX_ARCHIVE_DIR mechanism.

STATUS field at top is set to `pending submission` per the proof-transcript convention. DECISIONS.md was left unchanged because T01 kept onnxruntime (no structural removal to document).

## Verification

Ran the task plan verification command — all five grep/test checks passed (file exists, contains makepkg, namcap, aur.archlinux.org, STATUS:). Line count was 204, well above the 30-line minimum. Then ran `cargo test --test packaging` — all 10 tests passed including the three T02 assertions about clang makedep, sherpa-onnx offline source, and SHERPA_ONNX_ARCHIVE_DIR.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `test -f docs/distribution-proofs/aur/README.md && grep -q 'makepkg' docs/distribution-proofs/aur/README.md && grep -q 'namcap' docs/distribution-proofs/aur/README.md && grep -q 'aur.archlinux.org' docs/distribution-proofs/aur/README.md && grep -q 'STATUS:' docs/distribution-proofs/aur/README.md` | 0 | ✅ pass | 25ms |
| 2 | `wc -l docs/distribution-proofs/aur/README.md` | 0 | ✅ pass (204 lines > 30) | 10ms |
| 3 | `cargo test --test packaging` | 0 | ✅ pass (10/10 tests) | 80ms |

## Deviations

none

## Known Issues

sha256sums in packaging/PKGBUILD still contain 'SKIP' placeholders — must be replaced with real hashes at release time using the workflow documented in docs/distribution-proofs/aur/README.md

## Files Created/Modified

- `docs/distribution-proofs/aur/README.md`
