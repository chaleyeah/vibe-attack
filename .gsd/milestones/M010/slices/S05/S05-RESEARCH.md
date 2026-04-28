# S05: README install section rewrite — Research

**Date:** 2026-04-28
**Slice:** M010/S05
**Risk:** low
**Depends:** S01, S02, S04

## Summary

S05 is a documentation-only slice. The goal is to rewrite the `README.md` Installation section so a stranger can follow it from zero to "stratagem fired by voice" using either the AppImage or the AUR package — without reading a wiki or asking for help.

The current README (`README.md`) has a developer-centric install section: it assumes the reader will `git clone`, run `cargo build --release --features stt,gui`, download a Whisper model manually, and hand-configure `config.yaml`. There is no mention of the AppImage, no AUR install command, and no first-run wizard walkthrough. Every sentence assumes the reader is the author.

S01 established the AppImage proof harness and the GitHub Releases artifact shape (`vibe-attack-${TAG}-x86_64.AppImage`). S02 established the wizard UAT transcripts and the `--skip-wizard` flag. S04 finalized the PKGBUILD and documented the AUR submission workflow with maintainer `chaleyeah`. All three dependency slices are complete. S05 now owns wiring those artifacts into the README.

The rewrite is low-risk and well-scoped: one file (`README.md`), no code changes, no test additions, verification by reading the result. The primary success criterion is: a person unfamiliar with the project reads the new README and reaches "stratagem fired by voice" without asking for help.

## Recommendation

Rewrite the `## Installation` section of `README.md` in place. Keep the developer/build-from-source path (it is valid and used by contributors) but demote it to a collapsible section or a clearly labelled "Build from Source" subsection. Promote the AppImage path to the top as the primary install method. Add the AUR path as a clearly labelled alternative. Add a first-run wizard walkthrough section.

Do not touch anything outside `## Installation` and the new wizard walkthrough. The Features, Usage, Configuration, Troubleshooting, and License sections are out of scope.

## Implementation Landscape

### Key Files

- `README.md` — the only file that changes. Current install section: lines 26–81. Lines 1–25 (intro, features, feature flags) and lines 83–133 (usage, config, troubleshooting, license) are out of scope.
- `docs/distribution-proofs/aur/README.md` — AUR submission workflow doc from S04; confirms the AUR package name (`vibe-attack`), maintainer (`chaleyeah`), and install commands (`paru -S vibe-attack` / `yay -S vibe-attack`).
- `docs/distribution-proofs/appimage/README.md` — AppImage proof doc from S01; confirms the artifact naming convention: `vibe-attack-${TAG}-x86_64.AppImage`, produced by CI on tag push.
- `.github/workflows/release.yml` — confirms CI uploads three artifacts to GitHub Releases: `vibe-attack-*-x86_64.AppImage`, `vibe-attack-*.tar.gz`, `hd2-*.hdpack`.
- `docs/distribution-proofs/wizard/README.md` — wizard UAT doc from S02; confirms the four wizard scenarios and the `--skip-wizard` flag.
- `docs/distribution-proofs/wizard/debian12/transcript.md` — wizard reproduction steps; confirms the exact sequence (CreateConfig → InstallModel → SetupUinput → ConfigurePtt) and that `polkit` agent must be running.
- `packaging/PKGBUILD` — confirms `pkgname=vibe-attack`, `url=https://github.com/chaleyeah/vibe-attack`, and that `onnxruntime` is a runtime dep on Arch (needed for the AUR install note).
- `docs/uinput-setup.md` — existing doc; the wizard section in README should link to it for uinput permission details rather than duplicating them.

### New README Install Section Structure

The rewrite should produce the following subsection hierarchy under `## Installation`:

1. **AppImage (recommended — Debian, Fedora, Arch, any distro)**
   - Download link pattern: GitHub Releases page (`https://github.com/chaleyeah/vibe-attack/releases`)
   - chmod +x and run
   - Note: Debian/Ubuntu needs `libfuse2` (`sudo apt install libfuse2`)
   - First-run wizard runs automatically on first launch

2. **AUR (Arch Linux / CachyOS)**
   - `paru -S vibe-attack` or `yay -S vibe-attack`
   - Note: `onnxruntime` is a runtime dependency (installed automatically by pacman)
   - First-run wizard runs automatically on first launch

3. **First-Run Wizard**
   - Brief prose walkthrough: 4 steps (CreateConfig, InstallModel, SetupUinput, ConfigurePtt)
   - What each step does in one sentence
   - Note: wizard skipped on relaunch when `~/.config/vibe-attack/config.yaml` exists
   - `--skip-wizard` flag for users who provide their own config
   - Link to `docs/uinput-setup.md` for uinput permission details

4. **Build from Source** (moved from primary position, kept for contributors)
   - Prerequisites (Rust toolchain, system deps per distro)
   - `cargo build --release --features stt,gui`
   - Brief note that this path requires manual Whisper model placement and `config.yaml` setup

### Build Order

T01 (only task): rewrite `README.md` install section. There is no risky code integration; the only dependency is having the artifact names and wizard steps correct — both are confirmed by the S01/S02/S04 summaries above.

### Verification Approach

No automated test exists for README prose quality. Verification is:

1. Read the new `## Installation` section top-to-bottom and confirm the AppImage path, AUR path, wizard walkthrough, and build-from-source path are all present and internally consistent.
2. `grep -q 'AppImage' README.md` — AppImage section present.
3. `grep -q 'yay -S vibe-attack\|paru -S vibe-attack' README.md` — AUR command present.
4. `grep -q 'skip-wizard\|--skip-wizard' README.md` — wizard flag documented.
5. `grep -q 'uinput-setup' README.md` — uinput doc linked.
6. `grep -q 'libfuse2' README.md` — Debian FUSE note present.
7. `cargo test --test ui_distribution --test packaging --test distribution_proofs -- --test-threads=1` — all pre-existing tests still pass (README change is non-code; this is a regression guard).

## Constraints

- Do not change lines outside `## Installation` and the new wizard walkthrough subsection.
- The GitHub Releases URL is `https://github.com/chaleyeah/vibe-attack/releases` — use this, not a versioned artifact URL (the actual tag is unknown at write time).
- The AUR package name is `vibe-attack` per `packaging/PKGBUILD pkgname`.
- `onnxruntime` is a runtime dep on Arch (MEM090) — mention it in the AUR section so users on minimal installs know what will be pulled.
- `libfuse2` (not `libfuse3`) is required on Debian 12 — this was flagged in the M010 context as a known gotcha.
- The wizard binary is `vibe-attack-config`, invoked automatically by the AppImage/installed binary on first launch when `~/.config/vibe-attack/config.yaml` is absent.
- Do not add a `## First-Run Wizard` top-level section that duplicates the existing `## Usage` section — keep the wizard walkthrough inside `## Installation` as a subsection.

## Common Pitfalls

- **Linking to a versioned artifact URL** — releases move; always link to the Releases page, not a pinned artifact URL.
- **Describing manual model placement in the primary path** — the wizard handles model download; only the build-from-source path requires manual model placement.
- **Duplicating uinput permission instructions** — `docs/uinput-setup.md` already covers this; link, don't inline.
- **Forgetting the Fedora system dep name difference** — `libasound2-dev` (Debian) vs `alsa-lib-devel` (Fedora) vs `alsa-lib` (Arch); these belong in the build-from-source subsection only, not the AppImage path.
