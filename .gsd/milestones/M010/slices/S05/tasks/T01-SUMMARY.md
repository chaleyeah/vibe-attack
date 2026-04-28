---
id: T01
parent: S05
milestone: M010
key_files:
  - README.md
key_decisions:
  - Releases page URL used (not versioned artifact URL) so the link stays valid as future tags are pushed
  - libfuse2 explicitly called out (not libfuse3) matching the known M010 AppImage gotcha
  - Fedora build deps added as third block (alsa-lib-devel vs Debian libasound2-dev)
  - uinput instructions linked to docs/uinput-setup.md rather than inlined
duration: 
verification_result: passed
completed_at: 2026-04-28T11:16:16.643Z
blocker_discovered: false
---

# T01: Rewrote README.md Installation section: AppImage primary, AUR alternative, first-run wizard walkthrough, build-from-source demoted to contributor subsection

**Rewrote README.md Installation section: AppImage primary, AUR alternative, first-run wizard walkthrough, build-from-source demoted to contributor subsection**

## What Happened

Replaced the entire `## Installation` section (previously a developer-centric `git clone` + `cargo build` flow) with a four-subsection layout targeting end users first:

1. **AppImage (recommended)** — links to the Releases page (not a versioned URL), shows the two-command run sequence, adds the `libfuse2` Debian/Ubuntu note (not libfuse3 — known gotcha), and mentions the first-run wizard.
2. **AUR (Arch / CachyOS)** — shows both `paru -S vibe-attack` and `yay -S vibe-attack`, notes that `onnxruntime` is a runtime dep pulled automatically by pacman (per PKGBUILD `depends=('alsa-lib' 'onnxruntime')`), and mentions the wizard.
3. **First-Run Wizard** — four-bullet walkthrough of CreateConfig → InstallModel → SetupUinput → ConfigurePtt, documents that the wizard is skipped on relaunch when config exists, documents `--skip-wizard`, and links to `docs/uinput-setup.md` for uinput details.
4. **Build from Source** — moved the existing Prerequisites / System Dependencies / Install Rust / clone-and-build content here; added a Fedora system deps block (`sudo dnf install gcc alsa-lib-devel pkg-config`); kept all four `cargo build` variants; added a note that this path requires manual model placement and config setup.

Lines 1–23 (intro, features, feature flags) and lines after the Installation section (Usage, Configuration, Troubleshooting, License) were not touched. The `docs/uinput-setup.md` link was preserved, not inlined. All 11 grep assertions and all 41 cargo tests pass.

## Verification

Ran all 12 verification checks from the task plan:
- All 11 grep assertions returned exit 0
- `cargo test --test ui_distribution --test packaging --test distribution_proofs --test wizard_proofs -- --test-threads=1` → 41 tests passed, 0 failed

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `grep -q '^### AppImage' README.md` | 0 | ✅ pass | 5ms |
| 2 | `grep -q '^### AUR' README.md` | 0 | ✅ pass | 4ms |
| 3 | `grep -q '^### First-Run Wizard' README.md` | 0 | ✅ pass | 4ms |
| 4 | `grep -q '^### Build from Source' README.md` | 0 | ✅ pass | 4ms |
| 5 | `grep -q 'paru -S vibe-attack\|yay -S vibe-attack' README.md` | 0 | ✅ pass | 4ms |
| 6 | `grep -q -- '--skip-wizard' README.md` | 0 | ✅ pass | 4ms |
| 7 | `grep -q 'libfuse2' README.md` | 0 | ✅ pass | 4ms |
| 8 | `grep -q 'docs/uinput-setup.md' README.md` | 0 | ✅ pass | 4ms |
| 9 | `grep -q 'releases' README.md` | 0 | ✅ pass | 4ms |
| 10 | `grep -q 'onnxruntime' README.md` | 0 | ✅ pass | 4ms |
| 11 | `grep -q 'alsa-lib-devel' README.md` | 0 | ✅ pass | 4ms |
| 12 | `cargo test --test ui_distribution --test packaging --test distribution_proofs --test wizard_proofs -- --test-threads=1` | 0 | ✅ pass | 1100ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `README.md`
