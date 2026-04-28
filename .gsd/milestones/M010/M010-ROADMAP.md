# M010: Distribution — AppImage, AUR, First-Run Wizard

**Vision:** A stranger can download a single AppImage, mark it executable, run it on Debian, Fedora, or Arch, walk through the first-run wizard, and fire a Helldivers 2 stratagem by voice — without reading a wiki or asking anyone. AUR users get yay -S vibe-attack. The project ships.

## Success Criteria

- cargo test passes throughout
- cargo clippy -D warnings clean
- AppImage produced by release CI on tag push; downloadable artifact under 50 MB
- AppImage runs on Debian 12, Fedora 39, and Arch latest in clean VMs — verified transcripts in docs/distribution-proofs/
- AUR PKGBUILD passes namcap clean; makepkg -si produces a working installation on Arch
- First-run wizard completes end-to-end on each target distro: mic test → PTT key capture → Whisper model download → HD2 pack load → stratagem fired by voice
- Wizard does not reappear on subsequent launches (config.yaml present = wizard skipped)
- Wizard surfaces clear remediation for uinput permission denied and mic unavailable failure modes
- README install section walks a new user from download to stratagem-fired-by-voice
- Release CI workflow builds AppImage + source tarball + bundled HD2 .hdpack on tag push

## Slices

- [ ] **S01: AppImage build verification** `risk:medium` `depends:[]`
  > After this: AppImage runs ./vibe-attack-x86_64.AppImage --version on all three target distros; recorded transcripts in docs/distribution-proofs/appimage/

- [ ] **S02: First-run wizard end-to-end UAT** `risk:medium` `depends:[]`
  > After this: Wizard completes on all three distros; relaunch skips wizard; stratagem fires by voice; three UAT transcripts under docs/distribution-proofs/wizard/

- [ ] **S03: Release CI workflow extension** `risk:low` `depends:[S01]`
  > After this: Tag push triggers release workflow; AppImage, tarball, .hdpack appear in GitHub Releases; workflow passes

- [ ] **S04: AUR PKGBUILD finalization and submission** `risk:medium` `depends:[]`
  > After this: namcap clean; makepkg -si installs working binary; AUR package visible at aur.archlinux.org

- [ ] **S05: README install section rewrite** `risk:low` `depends:[S01,S02,S04]`
  > After this: A person unfamiliar with the project reads README.md and reaches 'stratagem fired by voice' without asking for help

- [ ] **S06: Final distribution UAT** `risk:low` `depends:[S03,S05]`
  > After this: Three transcripts under docs/distribution-proofs/final/ — one per distro — each showing stratagem fired by voice from a clean AppImage install

## Boundary Map

## Boundary Map

### Internal boundaries touched
- **packaging/appimage/build.sh** — fixes for target-distro compatibility
- **packaging/PKGBUILD** — clang in makedepends, pinned pkgver, correct sha256sums
- **.github/workflows/release.yml** — extend for AppImage + tarball + .hdpack on tag push
- **src/ui/wizard.rs** — fix failure modes (uinput, mic, model download retry)
- **src/ui/first_run.rs** — verify wizard-skip logic
- **README.md** — install section rewrite
- **docs/distribution-proofs/** — new directory for proof transcripts
- **tests/ui_distribution.rs** — packaging assertions

### Untouched (explicitly out of scope)
- Flatpak, .deb, .rpm formal packages
- Auto-update, code signing
- Windows / macOS (future)
