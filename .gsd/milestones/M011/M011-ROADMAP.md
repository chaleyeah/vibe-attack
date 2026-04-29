# M011: M011

**Vision:** 

## Slices

- [x] **S01: S01** `risk:high` `depends:[]`
  > After this: `cargo test --test distribution_proofs --test-threads=1` passes with the new four-distro names; old three-distro directories are removed; test function names and README per-distro sections updated.

- [x] **S02: S02** `risk:high` `depends:[]`
  > After this: all 12 transcripts (appimage + wizard + final × 4 distros) carry `STATUS: ok`; proof trees are complete.

- [x] **S03: S03** `risk:medium` `depends:[]`
  > After this: wizard flow, config screen, and tray menu issues found during VM runs are fixed; changes verified in the four distro environments.

- [ ] **S04: S04** `risk:medium` `depends:[]`
  > After this: `Cargo.toml`, `vibe-attack.spec`, and `PKGBUILD` read `1.0.0`; `CHANGELOG.md` has a dated `[1.0.0]` block; `release.yml` builds and uploads AppImage + .deb + .rpm + source tarball on a real test-tag push.

- [ ] **S05: Publish GitHub Release v1.0.0** `risk:low` `depends:[S02,S03,S04]`
  > After this: GitHub Releases `v1.0.0` is live with all four artifacts; AUR PKGBUILD sha256sums pinned to real release hashes.
