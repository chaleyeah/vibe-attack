---
id: T05
parent: S03
milestone: M013
key_files:
  - .gsd/milestones/M013/slices/S03/T05-RESULT.md
key_decisions:
  - Adapted T05 to target v1.0.1-test4 (final successful tag) instead of v1.0.1-test2 as planned — T04 required two extra fix iterations
  - GitHub release asset filenames normalize ~ to . — documented in T05-RESULT.md rather than treating as a failure; internal RPM metadata retains tilde
  - Cleaned up all four test tags (test, test2, test3, test4) not just the planned test2
duration: 
verification_result: passed
completed_at: 2026-05-03T22:34:49.823Z
blocker_discovered: false
---

# T05: Verified all 5 release assets on v1.0.1-test4 and cleaned up all four test tags (v1.0.1-test, test2, test3, test4) from origin and locally

**Verified all 5 release assets on v1.0.1-test4 and cleaned up all four test tags (v1.0.1-test, test2, test3, test4) from origin and locally**

## What Happened

T05 was written for v1.0.1-test2 but T04 deviated — two additional bug fixes were required, resulting in v1.0.1-test4 as the final successful tag. This task adapted accordingly.

**Asset verification for v1.0.1-test4:**
`gh release view v1.0.1-test4 --repo chaleyeah/vibe-attack --json assets --jq '.assets[].name'` returned all 5 expected assets:
- `hd2-v1.0.1-test4.hdpack` (1780 bytes)
- `vibe-attack-1.0.1.test4-1.x86_64.rpm` (11.9 MB)
- `vibe-attack-v1.0.1-test4-x86_64.AppImage` (20.7 MB)
- `vibe-attack-v1.0.1-test4.tar.gz` (181.9 MB)
- `vibe-attack_1.0.1-test4-1_amd64.deb` (8.1 MB)

**RPM tilde normalization:** The RPM was built as `vibe-attack-1.0.1~test4-1.x86_64.rpm` (confirmed via `Wrote:` line in the rpmbuild log), but GitHub's release asset upload normalizes `~` to `.` in the download filename, yielding `vibe-attack-1.0.1.test4-1.x86_64.rpm` as the visible asset name. The internal RPM `Version:` metadata retains `1.0.1~test4` which is correct per RPM spec.

**Cleanup:**
- `gh release delete v1.0.1-test4 --repo chaleyeah/vibe-attack --yes` — succeeded (no output = success)
- `git push origin :refs/tags/v1.0.1-test2 :refs/tags/v1.0.1-test3 :refs/tags/v1.0.1-test4` — all three deleted from origin in one push
- `git tag -d v1.0.1-test2 v1.0.1-test3 v1.0.1-test4` — all three deleted locally
- v1.0.1-test was already removed by T03

Post-cleanup `git ls-remote --tags origin | grep v1.0.1-test` returned empty. All test tags gone.

## Verification

Asset verification: all 5 expected assets confirmed present on GitHub Release v1.0.1-test4 via gh CLI. RPM filename uses `.` instead of `~` (GitHub normalization); internal RPM metadata retains tilde (correct). T05-RESULT.md written to .gsd/milestones/M013/slices/S03/T05-RESULT.md with asset list and cleanup status. Origin tag check: `git ls-remote --tags origin | grep v1.0.1-test` returns empty. Local tag check: `git tag -l v1.0.1-test*` returns empty.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `gh release view v1.0.1-test4 --repo chaleyeah/vibe-attack --json assets --jq '.assets[].name'` | 0 | ✅ pass — 5 assets present: hdpack, rpm, AppImage, tar.gz, deb | 1800ms |
| 2 | `gh release delete v1.0.1-test4 --repo chaleyeah/vibe-attack --yes` | 0 | ✅ pass — GitHub Release deleted | 1200ms |
| 3 | `git push origin :refs/tags/v1.0.1-test2 :refs/tags/v1.0.1-test3 :refs/tags/v1.0.1-test4` | 0 | ✅ pass — all 3 test tags deleted from origin | 2100ms |
| 4 | `git tag -d v1.0.1-test2 v1.0.1-test3 v1.0.1-test4` | 0 | ✅ pass — all 3 test tags deleted locally | 100ms |
| 5 | `git ls-remote --tags origin | grep v1.0.1-test` | 1 | ✅ pass — no test tags on origin | 1500ms |
| 6 | `test -f .gsd/milestones/M013/slices/S03/T05-RESULT.md && grep -q 'cleanup confirmed'` | 0 | ✅ pass — T05-RESULT.md written with cleanup confirmed | 50ms |

## Deviations

Plan targeted v1.0.1-test2; actual final tag was v1.0.1-test4 due to T04 fix iterations. Cleaned up test2/test3/test4 instead of just test2. RPM filename on GitHub uses . not ~ (GitHub normalization) — documented but not treated as a failure since internal RPM metadata is correct.

## Known Issues

GitHub normalizes ~ to . in release asset filenames. The release.yml glob `vibe-attack-*.x86_64.rpm` still matches the normalized filename. Future releases with pre-release tags will show `.` in the GitHub asset filename even though the RPM internally uses `~`.

## Files Created/Modified

- `.gsd/milestones/M013/slices/S03/T05-RESULT.md`
