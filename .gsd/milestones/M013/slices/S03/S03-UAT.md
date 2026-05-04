# S03: S03: End-to-end CI + Release pipeline smoke test — UAT

**Milestone:** M013
**Written:** 2026-05-03T22:35:21.776Z

# S03 UAT: CI + Release Pipeline End-to-End Verification

## Outcome

All 7 GitHub Actions jobs ran green for tag v1.0.1-test4:

**CI workflow:**
- Validate AUR PKGBUILD: success (14s)
- Clippy: success
- Test: success (2m24s)

**Release workflow:**
- Build AppImage: success (9m55s)
- Build Debian package: success (11m23s)
- Build RPM package: success (10m36s)
- Publish GitHub Release: success (17s)

## Assets Verified on GitHub Release v1.0.1-test4

| Asset | Present | Size |
|-------|---------|------|
| vibe-attack-v1.0.1-test4-x86_64.AppImage | ✅ | 20.7 MB |
| vibe-attack-v1.0.1-test4.tar.gz | ✅ | 181.9 MB |
| hd2-v1.0.1-test4.hdpack | ✅ | 1780 bytes |
| vibe-attack_1.0.1-test4-1_amd64.deb | ✅ | 8.1 MB |
| vibe-attack-1.0.1.test4-1.x86_64.rpm | ✅ | 11.9 MB |

Note: GitHub normalizes `~` → `.` in RPM asset filename; internal RPM metadata retains `Version: 1.0.1~test4`.

## Cleanup

All test tags (v1.0.1-test, v1.0.1-test2, v1.0.1-test3, v1.0.1-test4) removed from origin and locally. GitHub Release deleted.

