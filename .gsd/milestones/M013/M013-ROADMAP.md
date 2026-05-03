# M013: CI Build Revamp & Package Distribution

**Vision:** Make the release pipeline reliable end-to-end: fix versioning bugs, validate every packaging format in CI, and confirm AUR, AppImage, .deb, and .rpm artifacts can be produced from a clean tag push without manual intervention.

## Success Criteria

- Pushing a vN.N.N tag produces all four package artifacts (AppImage, .deb, .rpm, AUR-ready tarball) with the correct version embedded in each
- RPM spec Version field matches the git tag — no hardcoded 1.0.0
- Debian changelog version matches the git tag
- CI validate-pkgbuild job catches PKGBUILD syntax and field regressions on every tag push
- README shows live CI and Release status badges

## Slices

- [x] **S01: S01** `risk:low` `depends:[]`
  > After this: Show release.yml diff: RPM tarball prefix uses GITHUB_REF_NAME, deb changelog stamped with tag version, dh-cargo removed

- [x] **S02: S02** `risk:low` `depends:[]`
  > After this: ci.yml validate-pkgbuild job runs and passes, showing all required fields and function definitions present

- [ ] **S03: S03** `risk:medium` `depends:[]`
  > After this: GitHub Actions run shows all jobs green; downloaded artifacts have correct version in filename and package metadata

- [x] **S04: S04** `risk:low` `depends:[]`
  > After this: README shows two rendered badge images linking to the correct Actions workflow runs

## Boundary Map

Not provided.
