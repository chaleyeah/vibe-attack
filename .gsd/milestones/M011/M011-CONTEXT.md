# M011: v1.0 Release

## Vision

Ship vibe-attack v1.0: replace stale VM proof transcripts with fresh runs on current distros, apply UI polish surfaced during those runs, version-bump all release artifacts to 1.0.0, and publish AppImage + .deb + .rpm + source tarball to GitHub Releases.

## Scope

### 1. VM Proof Runs (prerequisite for UI polish)

Replace the three existing proof transcript directories (`debian12`, `fedora39`, `arch`) across all three proof trees (`appimage/`, `wizard/`, `final/`) with directories for the new target distros:

- Debian 13
- Ubuntu 26.04
- Fedora 44
- CachyOS

VM runs are executed manually. The new directories and transcripts replace the old ones entirely — no legacy directories are kept.

**Test impact:** `tests/distribution_proofs.rs` hardcodes the old distro names — these references must be updated to match the new directory names before the tests can pass.

READMEs and reproduction notes in each proof directory get updated to reflect the new distro set.

### 2. UI Polish

Wizard flow, config screen, and tray menu need layout, wording, and UX improvements. No empirical basis exists yet (the software hasn't been exercised end-to-end on real hardware), so VM proof runs are a natural prerequisite — real UX failures will surface during those runs. Polish work follows proof runs.

### 3. v1.0 Release Artifacts

- Promote `[Unreleased]` → `[1.0.0]` in `CHANGELOG.md`
- Bump version from `0.1.0` to `1.0.0` in:
  - `Cargo.toml`
  - `packaging/vibe-attack.spec`
  - `packaging/PKGBUILD`
- Add `.deb` build job (via `dpkg-deb` or `cargo-deb`) to `release.yml`
- Add `.rpm` build job (via `rpmbuild`) to `release.yml`
- Update AUR `PKGBUILD` for the 1.0.0 release
- GitHub Releases publishes: AppImage + `.deb` + `.rpm` + source tarball

The `.deb` and `.rpm` packaging skeletons already exist (`packaging/debian/` and `packaging/vibe-attack.spec`); CI only needs the build jobs wired up.

## Out of Scope

**Error message improvements** — deferred until post-v1.0 when real-hardware usage reveals what's actually confusing.

## Slice Order

1. Rename/create proof directories + update test harness (unblocks everything)
2. Run VM proofs (manually — agent scaffolds transcripts, human runs VMs)
3. UI polish from proof-run findings
4. Version bump + release CI (.deb/.rpm jobs)
5. Publish GitHub Release

## Key Constraints

- Distribution proof tests use exact string matching on directory names — any rename must be reflected in `distribution_proofs.rs` simultaneously
- `--test-threads=1` required for distribution proof tests (pre-existing flake)
- AUR sha256sums use `SKIP` during development; real hashes pinned at release time per existing convention
