# T05 Result

Verified at: 2026-05-03T23:00:00Z

## Note on tag deviation

The T05 plan was written for `v1.0.1-test2`. Due to two additional fix iterations in T04, the final successful tag is `v1.0.1-test4`. This result file documents the verified assets for `v1.0.1-test4` and cleanup of all test tags (v1.0.1-test2, v1.0.1-test3, v1.0.1-test4).

## Release assets for v1.0.1-test4

hd2-v1.0.1-test4.hdpack
vibe-attack-1.0.1.test4-1.x86_64.rpm
vibe-attack-v1.0.1-test4-x86_64.AppImage
vibe-attack-v1.0.1-test4.tar.gz
vibe-attack_1.0.1-test4-1_amd64.deb

## Note on RPM filename tilde normalization

The RPM was built internally as `vibe-attack-1.0.1~test4-1.x86_64.rpm` (tilde, confirmed in rpmbuild log: `Wrote: .../vibe-attack-1.0.1~test4-1.x86_64.rpm`). GitHub's release asset upload normalizes `~` to `.` in the download filename, resulting in `vibe-attack-1.0.1.test4-1.x86_64.rpm` as the release asset name. The RPM package metadata (`Version: 1.0.1~test4`) retains the tilde and is correct per RPM spec.

## Verification

- [x] vibe-attack-v1.0.1-test4-x86_64.AppImage
- [x] vibe-attack-v1.0.1-test4.tar.gz
- [x] hd2-v1.0.1-test4.hdpack
- [x] vibe-attack_1.0.1-test4-1_amd64.deb
- [x] vibe-attack-1.0.1.test4-1.x86_64.rpm (GitHub normalizes ~ to . in asset filenames; internal RPM metadata uses 1.0.1~test4)

All 5 expected assets present with correct version-stamped filenames.

## Cleanup

- GitHub Release v1.0.1-test4 deleted: yes
- Tags deleted from origin: v1.0.1-test2, v1.0.1-test3, v1.0.1-test4
- Tags deleted locally: v1.0.1-test2, v1.0.1-test3, v1.0.1-test4

cleanup confirmed
