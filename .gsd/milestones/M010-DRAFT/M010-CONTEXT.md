# M010: Distribution — AppImage, AUR, First-Run Wizard

**Gathered:** 2026-04-27
**Status:** Draft — ready for grilling and decomposition

## Project Description

vibe-attack has a working pipeline, a tray, a config window (after M008), and a complete pack system (after M009). The remaining gap is **distribution**: there is no real AppImage anyone can download, the AUR package has not been submitted, and the first-run wizard exists in code (`src/ui/wizard.rs`, 812 LOC) but has not been validated end-to-end on a clean machine.

This milestone makes the project actually installable by someone who isn't the author. It also closes the first-run-wizard requirement (UI-04), which is the bridge between "downloaded the AppImage" and "fired a stratagem by voice" — the wizard tests the mic, captures the PTT key, downloads the Whisper model, and loads the HD2 pack.

## Why This Milestone

The project's stated audience is "small release — other users installing and running it." Without distribution, that audience cannot exist. The AppImage script is scaffolded (`packaging/appimage/build.sh`); PKGBUILD exists at `packaging/PKGBUILD`. Neither has produced a verified artifact a real user has downloaded and run. M010 verifies both, fixes whatever breaks, and ships.

The first-run wizard belongs in this milestone because it's the first thing a new user sees, and a broken wizard wastes the entire distribution effort.

## User-Visible Outcome

### When this milestone is complete, the user can:

- Download a single `vibe-attack-x86_64.AppImage` file from the project release page, mark it executable, and run it on Debian, Fedora, or Arch — no system package install required
- Install `vibe-attack` from the AUR (`yay -S vibe-attack` or `paru -S vibe-attack`) on Arch and CachyOS
- On first run, walk through the wizard: pick a microphone, capture a PTT key, download the Whisper model, load the HD2 pack — and end up with a working setup
- Quit and re-launch; the wizard does not re-appear; the saved config is honored

### Entry point / environment

- Entry point: `./vibe-attack-x86_64.AppImage` or `vibe-attack` (after AUR install)
- Environment: clean Linux install (Debian 12 / Fedora 39 / Arch latest), no prior vibe-attack state
- Live dependencies involved: D-Bus session bus, PipeWire/PulseAudio, evdev, uinput, internet (model download)

## Completion Class

- Contract complete means: AppImage build script produces a self-contained `.AppImage` artifact in CI; PKGBUILD passes `namcap` and builds cleanly; wizard step machine has full unit-test coverage for happy path and three failure modes (mic unavailable, model download fails, uinput permission denied)
- Integration complete means: the AppImage runs on Debian 12, Fedora 39, and Arch latest in a clean VM or container (no host vibe-attack install); the wizard completes end-to-end on each; the AUR package builds and installs from a `makepkg` invocation
- Operational complete means: a user follows the README install instructions, downloads the artifact, and reaches "stratagem fires by voice" without filing an issue; the tray, config window, and pack switching all continue to work post-install

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- The AppImage runs on Debian 12, Fedora 39 (or Red Hat-family equivalent), and Arch — verified in clean VMs, recorded in `docs/distribution-proofs/` per existing latency-proof convention
- The AUR PKGBUILD has been submitted and accepted; `makepkg -si` produces a working installation
- A first-run wizard run on a clean install ends with a successful voice-fired stratagem — recorded as a UAT video or transcript
- The release CI workflow (`.github/workflows/release.yml`) produces both the AppImage and a tarball on tag push

## Architectural Decisions

### AppImage build system

**Decision:** Continue with `linuxdeploy` + `appimagetool` orchestrated by `packaging/appimage/build.sh`. Bundle `libonnxruntime.so` and `libsherpa-onnx-c-api.so` via the existing find-and-copy logic. AppRun sets `LD_LIBRARY_PATH` so dlopen resolves both `.so` files inside the FUSE mount.

**Rationale:** The build script already works for the local case (S07 of M001 proved ORT bundling). `linuxdeploy` is the standard. Switching to a different bundler would discard work for no clear gain.

**Alternatives Considered:**
- Flatpak — broader desktop-store reach but requires sandbox exemptions for uinput and D-Bus that complicate first-run UX
- Native packages only (.deb / .rpm / AUR) — three more builds to maintain; the AppImage covers the long tail of distros

### First-run detection

**Decision:** First-run state is detected by absence of `~/.config/vibe-attack/config.yaml`. The wizard creates the config file on completion. A `--skip-wizard` flag forces past it for users who hand-roll their own config. The wizard binary is `vibe-attack-config` (already exists), invoked automatically on the first launch of `vibe-attack` if the config is missing.

**Rationale:** Single source of truth for "am I configured" — the config file. No state files, no markers, no version stamping for first-run-vs-not. Composable with users who copy a config from another machine.

**Alternatives Considered:**
- Separate `~/.config/vibe-attack/.first-run-done` marker — duplicates the meaning of "config exists"
- Wizard always runs and checks — irritating on relaunch; wastes the user's time

### CI release pipeline

**Decision:** Extend `.github/workflows/release.yml` (already exists) to produce three artifacts on a tag: AppImage, source tarball, and an `.hdpack` of the bundled HD2 pack for separate distribution. AUR submission is manual (paru/yay surface) and tracked in the milestone summary.

**Rationale:** Tagging is the existing release signal. CI artifacts are the easiest distribution surface. AUR cannot be fully automated from GitHub Actions without storing AUR SSH keys, which is a security cost we don't need to take for a small release.

**Alternatives Considered:**
- Auto-AUR-submit on tag — adds AUR key management to CI; high risk for a small project
- Manual everything — wastes work the existing release workflow already does

---

## Error Handling Strategy

AppImage failures (missing FUSE, GLIBC mismatch, missing libonnxruntime) produce specific stderr messages with remediation: "Install libfuse2: sudo apt install libfuse2", "GLIBC version too old: minimum required is 2.35". Wizard step failures show inline (mic test fails → "no audio detected, check device", model download fails → "network error: retry / select local file"). uinput permission errors trigger a guided remediation step (`usermod -aG input $USER` + `newgrp input`), the existing wizard already has this — verify it actually completes.

## Risks and Unknowns

- AppImage on Debian 12 may need libfuse2 (not libfuse3); document explicitly in README
- ORT version pinning vs distro `libonnxruntime` packages — bundling our own avoids the risk; verify size impact (currently ~5MB)
- AUR package review may flag missing `clang` from `makedepends` (called out in S05 of M007); fix before submission
- Wizard's Whisper-model download uses HTTPS to huggingface.co — must handle redirects, partial transfers, and resume; currently relies on `reqwest`'s default behavior
- evdev PTT capture in the wizard requires read access to `/dev/input/*` — must verify the wizard handles "user not in input group" gracefully and surfaces the remediation
- First-launch UX on Wayland: the egui wizard window may appear behind other windows on some compositors — needs verification on KDE Plasma, GNOME, Hyprland

## Existing Codebase / Prior Art

- `packaging/appimage/build.sh` — full build script with ORT + sherpa bundling and AppRun generation; runs locally, hasn't been verified in CI
- `packaging/appimage/vibe-attack.desktop` — desktop file (verify Categories, MimeType, X-Linux-Voice-* fields are correct)
- `packaging/PKGBUILD` — AUR package definition; pkgver, sha256 placeholders need release-tag values
- `packaging/vibe-attack.spec` — RPM spec for Fedora/RHEL (lower priority than AppImage but useful for some users)
- `packaging/debian/` — debian/control, rules, changelog (for `.deb` if we choose to ship one)
- `.github/workflows/release.yml` — existing release workflow; extend for tag-triggered artifact builds
- `src/ui/wizard.rs` — 812 LOC wizard with state machine, model download, mic test, PTT capture, uinput setup
- `src/ui/first_run.rs` — `FirstRunState` struct and `SetupStep` enum; tested separately
- `tests/ui_distribution.rs` — distribution-related test scaffolding; extend with packaging assertions

## Relevant Requirements

- **DIST-01** — AppImage for distro-agnostic install — primary objective
- **DIST-02** — AUR / PKGBUILD for Arch / CachyOS — primary objective
- **UI-04** — first-run wizard (mic test, PTT bind, HD2 pack load) — primary objective; structural foundation done, runtime validation pending

## Scope

### In Scope

- Verifying AppImage runs on Debian 12, Fedora 39 (or RHEL-family), and Arch — recorded as artifacts under `docs/distribution-proofs/`
- Fixing whatever the verification surfaces (likely candidates: missing dependencies, GLIBC issues, FUSE handling, .desktop fields)
- AUR submission — final pkgver pin, namcap clean, AUR upload by maintainer
- README install section rewrite: AppImage download link, AUR command, first-run wizard walkthrough
- First-run wizard end-to-end UAT on each target distro — captured in `docs/distribution-proofs/`
- CI release workflow extension to produce tagged artifacts
- `tests/ui_distribution.rs` — packaging assertions (file presence, .desktop validity, AppRun executable bit)

### Out of Scope / Non-Goals

- Flatpak — out of scope
- `.deb` and `.rpm` formal packaging — out of scope; AppImage covers Debian/Fedora users; can be added later if demand exists
- Auto-update mechanism — out of scope
- Code signing — out of scope for a small release; can be added if/when distros require it
- Windows / macOS — explicitly future per project memory

## Technical Constraints

- All existing tests must pass
- `cargo clippy -D warnings` clean
- AppImage size budget — under 50 MB (currently ORT + sherpa + binary fits well within this)
- Reproducible build — `linuxdeploy` and `appimagetool` versions pinned in CI
- AUR package must build offline given `makepkg --offline` (no network during build)

## Integration Points

- GitHub Releases — artifact hosting
- huggingface.co — Whisper model download (wizard step)
- AUR — package distribution
- D-Bus session bus — tray works under AppImage FUSE mount (verify)
- evdev / uinput — must work from inside the AppImage (no special permissions; user must be in input group, surfaced by wizard)

## Testing Requirements

- Smoke test: AppImage runs `--version` on each target distro
- Smoke test: AppImage launches GUI and shows the first-run wizard on a clean install
- End-to-end UAT: clean install → wizard → fire stratagem by voice → recorded as transcript
- AUR test: `makepkg --offline` succeeds in a clean Arch container; `pacman -U *.pkg.tar.zst` installs cleanly; binary runs
- Wizard unit tests cover failure modes (mic unavailable, network error during model download, uinput permission denied)

## Acceptance Criteria

(Per-slice — to be refined during decomposition)

- AppImage produced by CI on tag push, downloadable from Releases
- AppImage runs on Debian 12, Fedora 39, Arch — three transcripts under `docs/distribution-proofs/`
- AUR package submitted and installable via `paru` / `yay`
- Wizard completes end-to-end on a clean install for each target distro
- README install section walks a new user from "I've never heard of this" to "stratagem fired by voice"
- `cargo test` passes including new packaging tests

## Open Questions

- Do we ship a `.deb` or `.rpm` in M010, or wait for user demand? — current thinking: AppImage + AUR cover the audience, defer
- AUR maintainer — who? Repo owner submits as `chaleyeah`? — current thinking: yes, repo owner is AUR maintainer; tracked in M010 summary
- Wizard model-download UX — is HTTPS to huggingface.co acceptable, or do we mirror the model on a project-controlled host? — current thinking: huggingface for v1; mirror only if huggingface rate-limits us
- Should the AppImage carry the bundled HD2 pack inside, or download on first run? — current thinking: bundle inside (~50KB); avoids first-run network dependency for the pack

## Suggested Slice Decomposition

- **S01** (medium risk, depends:[]): AppImage build verification — run `build.sh` on clean Debian 12, Fedora 39, Arch; fix what breaks; record proofs
- **S02** (medium risk, depends:[]): First-run wizard end-to-end UAT — run the existing wizard on a clean install per target distro; fix what breaks
- **S03** (low risk, depends:[S01]): Release CI workflow — tag-triggered AppImage + tarball + bundled `.hdpack`; pin `linuxdeploy` / `appimagetool` versions
- **S04** (medium risk, depends:[]): AUR PKGBUILD — fix `makedepends` (clang), pin pkgver, run `namcap`, submit
- **S05** (low risk, depends:[S01,S02,S04]): README install rewrite — AppImage download, AUR install, first-run walkthrough
- **S06** (low risk, depends:[S05]): Final UAT — fresh install on each target distro, full loop "download → wizard → fire stratagem", recorded under `docs/distribution-proofs/`
