# M005: Distribution Packaging

**Vision:** Ship vibe-attack as installable packages for all three target Linux distributions (Arch, Debian, Red Hat) and as a portable AppImage, with a GitHub Actions CI pipeline that builds and publishes the AppImage on every tagged release.

## Success Criteria

- AppImage runs on a clean Ubuntu/Debian machine without additional deps installed
- PKGBUILD installs cleanly via makepkg on Arch
- debian/ control file produces a .deb installable via dpkg -i
- RPM .spec file produces an .rpm installable via rpm -i on Fedora
- GitHub Actions workflow triggers on git tag push and uploads AppImage as a release asset
- All packages install both vibe-attack and vibe-attack-config binaries
- vibe-attack.svg icon is present and referenced correctly in all .desktop and packaging files

## Slices

- [x] **S01: S01** `risk:low` `depends:[]`
  > After this: assets/vibe-attack.svg exists; build.sh copies it into AppDir without errors

- [x] **S02: S02** `risk:medium` `depends:[]`
  > After this: ./packaging/appimage/build.sh exits 0; produced AppImage passes --appimage-extract sanity check showing both binaries and .so files

- [x] **S03: S03** `risk:medium` `depends:[]`
  > After this: dpkg-deb or dpkg-buildpackage produces vibe-attack_0.1.0_amd64.deb; dpkg --info shows correct metadata

- [x] **S04: S04** `risk:low` `depends:[]`
  > After this: namcap packaging/PKGBUILD outputs no errors or warnings beyond expected AUR-only advisory

- [x] **S05: S05** `risk:low` `depends:[]`
  > After this: rpmbuild -bs vibe-attack.spec produces a .src.rpm without error (source RPM proves spec is syntactically valid)

- [x] **S06: S06** `risk:medium` `depends:[]`
  > After this: .github/workflows/release.yml committed; workflow passes act dry-run or manual inspection shows correct job/step structure

## Boundary Map

Not provided.
