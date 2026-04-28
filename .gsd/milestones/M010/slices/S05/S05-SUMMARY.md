---
id: S05
parent: M010
milestone: M010
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - ["README.md"]
key_decisions:
  - [
  "Releases page URL used (not versioned artifact URL) so the link stays valid as future tags are pushed",
  "libfuse2 explicitly called out (not libfuse3) matching the known M010 AppImage Debian gotcha",
  "Fedora build deps block added as third distro (alsa-lib-devel vs Debian libasound2-dev / Arch alsa-lib)",
  "uinput instructions linked to docs/uinput-setup.md rather than inlined — avoids duplication drift",
  "onnxruntime runtime dep noted in AUR section per PKGBUILD depends= confirmed by S04"
]
patterns_established:
  - [
  "README install sections should be ordered by user persona: binary installers first, source builders last",
  "Link to Releases page (not versioned artifact URL) for AppImage downloads",
  "Per-distro system dep blocks: Debian block, Arch block, Fedora block — all three required for cross-distro coverage"
]
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-28T11:17:45.721Z
blocker_discovered: false
---

# S05: README install section rewrite

**README.md Installation section rewritten end-user-first: AppImage primary, AUR alternative, first-run wizard walkthrough, build-from-source demoted to contributor subsection**

## What Happened

The existing README.md Installation section was developer-centric: it led with `git clone` + `cargo build`, assumed manual Whisper model placement, and made no mention of the AppImage or AUR artifacts produced by S01/S02/S04. A stranger following it would hit immediate friction before reaching a working install.

T01 replaced the entire `## Installation` section with a four-subsection layout ordered by what a new user actually needs:

1. **AppImage (recommended — Debian, Fedora, Arch, any distro)**: Links to the Releases page (not a versioned artifact URL, so the link stays valid as future tags are pushed), shows the two-command chmod/run sequence, includes the critical libfuse2 Debian/Ubuntu note (not libfuse3 — a known M010 gotcha), and mentions the first-run wizard.

2. **AUR (Arch Linux / CachyOS)**: Shows both `paru -S vibe-attack` and `yay -S vibe-attack`, notes that `onnxruntime` is a runtime dependency pulled automatically by pacman (matching PKGBUILD `depends=('alsa-lib' 'onnxruntime')` from S04), and mentions the wizard.

3. **First-Run Wizard**: Four-bullet walkthrough of CreateConfig → InstallModel → SetupUinput → ConfigurePtt; documents that the wizard is skipped on relaunch when `~/.config/vibe-attack/config.yaml` exists; documents `--skip-wizard`; links to `docs/uinput-setup.md` for uinput permission details (not inlined — the existing doc is canonical).

4. **Build from Source**: The existing prerequisites content moved here; a Fedora system deps block added (`sudo dnf install gcc alsa-lib-devel pkg-config` — Fedora uses `alsa-lib-devel` vs Debian's `libasound2-dev` or Arch's `alsa-lib`); all four `cargo build` variants kept; a note added that this path requires manual model placement and config setup since the wizard auto-launch only applies to installed binaries.

Lines 1–23 (intro, features, feature flags) and lines after Installation (Usage, Configuration, Troubleshooting, License) were not touched. All 11 grep assertions passed and all 41 cargo tests in the regression guard suite passed.

## Verification

Ran 12 verification checks — all passed:
- 11 grep assertions (AppImage heading, AUR heading, First-Run Wizard heading, Build from Source heading, paru/yay commands, --skip-wizard flag, libfuse2, docs/uinput-setup.md link, releases URL, onnxruntime note, alsa-lib-devel): all exit 0
- `cargo test --test ui_distribution --test packaging --test distribution_proofs --test wizard_proofs -- --test-threads=1`: 41 tests passed, 0 failed

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

none

## Known Limitations

The first AppImage release tag may not yet exist when a reader visits the Releases page — this is acceptable since the link points to the page (not a versioned URL) and will be valid once S03's CI pushes the first tag.

## Follow-ups

S06 (Final distribution UAT) depends on S03 and S05 both being complete. S05 is now done; S06 can proceed once S03 CI has produced a downloadable AppImage artifact.

## Files Created/Modified

- `README.md` — Installation section rewritten: AppImage primary, AUR alternative, first-run wizard walkthrough, build-from-source demoted to contributor subsection with Fedora deps added
