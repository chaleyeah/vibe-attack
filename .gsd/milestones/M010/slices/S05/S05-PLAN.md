# S05: README install section rewrite

**Goal:** Rewrite the README.md `## Installation` section so a stranger can install vibe-attack via AppImage or AUR and reach 'stratagem fired by voice' without reading a wiki or asking for help. Demote the existing build-from-source path to a clearly-labelled subsection for contributors.
**Demo:** A person unfamiliar with the project reads README.md and reaches 'stratagem fired by voice' without asking for help

## Must-Haves

- A new reader follows README.md from top to bottom and (1) sees AppImage as the recommended install path with a working download/chmod/run sequence and the libfuse2 note for Debian, (2) sees the AUR command (paru/yay) as the Arch alternative with an onnxruntime runtime-dep note, (3) sees a brief first-run wizard walkthrough covering CreateConfig/InstallModel/SetupUinput/ConfigurePtt with the --skip-wizard flag and a link to docs/uinput-setup.md, (4) can still find the build-from-source path as a clearly-labelled subsection. All pre-existing distribution/packaging/wizard tests still pass.

## Proof Level

- This slice proves: contract — one file changes; verification is mechanical grep checks plus regression-guard test runs.

## Integration Closure

Not provided.

## Verification

- Not provided.

## Tasks

- [x] **T01: Rewrite README.md install section: AppImage primary, AUR alternative, wizard walkthrough, demote build-from-source** `est:45m`
  Rewrite the `## Installation` section of `README.md` (currently lines 24–81) so a stranger can install vibe-attack via AppImage or AUR and reach 'stratagem fired by voice' without reading a wiki. The current section is developer-centric: it leads with `git clone` + `cargo build`, assumes manual Whisper model placement, and never mentions the AppImage or AUR artifacts that S01/S02/S04 produced. Replace it with a four-subsection layout that promotes AppImage to primary, adds AUR as the Arch path, walks through the first-run wizard, and demotes build-from-source to a clearly-labelled contributor subsection.

Do NOT touch lines 1–23 (intro, features, feature flags) or lines 83+ (Usage, Configuration, Troubleshooting, License). The rewrite is scoped to the `## Installation` section only.

The new `## Installation` section must contain these four subsections in this order:

1. **`### AppImage (recommended — Debian, Fedora, Arch, any distro)`**
   - Tell the reader to download the latest `vibe-attack-*-x86_64.AppImage` from `https://github.com/chaleyeah/vibe-attack/releases` (link to the Releases page, NOT a versioned artifact URL — releases move).
   - Show the three-step run sequence in a fenced bash block: `chmod +x vibe-attack-*-x86_64.AppImage` then `./vibe-attack-*-x86_64.AppImage`.
   - Add a Debian/Ubuntu note: `sudo apt install libfuse2` is required (note: libfuse2, NOT libfuse3 — this is a known M010 gotcha).
   - Mention that the first-run wizard launches automatically and link forward to the wizard subsection.

2. **`### AUR (Arch Linux / CachyOS)`**
   - Show the install commands: `paru -S vibe-attack` or `yay -S vibe-attack`.
   - Note that `onnxruntime` is a runtime dependency installed automatically by pacman (so users on minimal installs know what will be pulled — see MEM090 / S04 SUMMARY).
   - Mention the first-run wizard launches automatically.

3. **`### First-Run Wizard`**
   - One-paragraph or four-bullet walkthrough of the four wizard steps in order: CreateConfig (writes `~/.config/vibe-attack/config.yaml`), InstallModel (downloads the Whisper GGML model), SetupUinput (configures `/dev/uinput` permissions via polkit), ConfigurePtt (captures the push-to-talk key via evdev). One sentence per step is enough.
   - Note that the wizard is skipped on relaunch when `~/.config/vibe-attack/config.yaml` already exists.
   - Document the `--skip-wizard` flag for users who provide their own config.
   - Link to `docs/uinput-setup.md` for uinput permission details — do NOT inline the uinput instructions; the existing doc is canonical.

4. **`### Build from Source`** (demoted from primary position; kept for contributors)
   - Move the existing Prerequisites / System Dependencies / Install Rust / uinput / Whisper Model / Clone and Build content into this subsection. Preserve the Debian/Ubuntu and Arch system-dep blocks; add Fedora system deps as a third block: `sudo dnf install gcc alsa-lib-devel pkg-config` (Fedora uses `alsa-lib-devel` rather than Debian's `libasound2-dev` or Arch's `alsa-lib`).
   - Keep the existing four `cargo build --release` variants (default, `--features stt`, `--features stt-vulkan`, `--features stt,gui`).
   - Add a brief note that this path requires manual Whisper model placement and `config.yaml` setup (the wizard is only auto-launched by the installed binary on first run when the config is absent — building from source and running directly skips that path unless the user wipes their config).

Writing style:
- Concise, direct, second-person imperative.
- Use fenced ```bash code blocks for commands.
- Use `inline code` for filenames, flags, and package names.
- Do not introduce any new top-level `##` heading other than what was already there — the wizard walkthrough is a `###` subsection inside `## Installation`, not a new top-level section.

Verification (run all of these after the edit):
- `grep -q '^### AppImage' README.md` → exit 0
- `grep -q '^### AUR' README.md` → exit 0
- `grep -q '^### First-Run Wizard' README.md` → exit 0
- `grep -q '^### Build from Source' README.md` → exit 0
- `grep -q 'paru -S vibe-attack\|yay -S vibe-attack' README.md` → exit 0
- `grep -q -- '--skip-wizard' README.md` → exit 0
- `grep -q 'libfuse2' README.md` → exit 0
- `grep -q 'docs/uinput-setup.md' README.md` → exit 0
- `grep -q 'releases' README.md` → exit 0 (Releases page link present)
- `grep -q 'onnxruntime' README.md` → exit 0 (AUR runtime-dep note)
- `grep -q 'alsa-lib-devel' README.md` → exit 0 (Fedora build-from-source dep)
- `cargo test --test ui_distribution --test packaging --test distribution_proofs --test wizard_proofs -- --test-threads=1` → all pass (regression guard; README change is non-code, so this should be unaffected — but if anything broke, this catches it).

Assumptions documented:
- The exact AUR maintainer / package name is `vibe-attack` per `packaging/PKGBUILD` `pkgname=vibe-attack` (confirmed by S04 SUMMARY).
- The Releases page URL is `https://github.com/chaleyeah/vibe-attack/releases` (research-confirmed, matches `packaging/PKGBUILD url=`).
- The first AppImage release tag may not yet exist at the time of this README edit; that is acceptable — the README links to the Releases page (not a versioned URL), so it stays valid as soon as S03's CI pushes the first tag.
  - Files: `README.md`
  - Verify: grep -q '^### AppImage' README.md && grep -q '^### AUR' README.md && grep -q '^### First-Run Wizard' README.md && grep -q '^### Build from Source' README.md && grep -q 'paru -S vibe-attack\|yay -S vibe-attack' README.md && grep -q -- '--skip-wizard' README.md && grep -q 'libfuse2' README.md && grep -q 'docs/uinput-setup.md' README.md && grep -q 'onnxruntime' README.md && grep -q 'alsa-lib-devel' README.md && cargo test --test ui_distribution --test packaging --test distribution_proofs --test wizard_proofs -- --test-threads=1

## Files Likely Touched

- README.md
