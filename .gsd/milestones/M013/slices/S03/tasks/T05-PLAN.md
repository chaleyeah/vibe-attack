---
estimated_steps: 60
estimated_files: 1
skills_used: []
---

# T05: Verify v1.0.1-test2 release assets and clean up the test tag

Confirm the GitHub Release created by the `v1.0.1-test2` tag contains all five expected assets with correct version-stamped filenames. Note the RPM filename uses `~` (tilde) instead of `-` because of the T03 RPM_VERSION substitution. Then delete the test tag and the release from both origin and locally. This task closes the slice.

## Expected asset names

Per `release.yml`:
- `vibe-attack-v1.0.1-test2-x86_64.AppImage` (uses `${GITHUB_REF_NAME}` with v prefix)
- `vibe-attack-v1.0.1-test2.tar.gz` (uses `${GITHUB_REF_NAME}` with v prefix)
- `hd2-v1.0.1-test2.hdpack` (uses `${GITHUB_REF_NAME}` with v prefix)
- `vibe-attack_1.0.1-test2-1_amd64.deb` (debian uses `${GITHUB_REF_NAME#v}` with hyphen — debian allows hyphens)
- `vibe-attack-1.0.1~test2-1.x86_64.rpm` (RPM uses tilde-substituted RPM_VERSION — note `~` not `-` between `1.0.1` and `test2`)

The asymmetry is intentional: AppImage/tarball/hdpack use the raw tag (with `v`), Debian strips `v` (hyphens fine), RPM strips `v` AND substitutes `-`→`~` (Version field constraint).

## Steps

1. Fetch release asset metadata: `gh release view v1.0.1-test2 --repo chaleyeah/vibe-attack --json assets --jq '.assets[].name'`.
2. Write the result and a timestamp to `.gsd/milestones/M013/slices/S03/T05-RESULT.md`. Format:
   ```
   # T05 Result
   
   Verified at: <ISO timestamp>
   
   ## Release assets for v1.0.1-test2
   
   <one filename per line>
   
   ## Verification
   
   - [x] vibe-attack-v1.0.1-test2-x86_64.AppImage
   - [x] vibe-attack-v1.0.1-test2.tar.gz
   - [x] hd2-v1.0.1-test2.hdpack
   - [x] vibe-attack_1.0.1-test2-1_amd64.deb
   - [x] vibe-attack-1.0.1~test2-1.x86_64.rpm
   
   ## Cleanup
   
   - GitHub Release deleted: yes
   - Tag deleted from origin: yes
   - Tag deleted locally: yes
   
   cleanup confirmed
   ```
3. Confirm all five expected filenames are present (exact string match). If any missing or misnamed, do NOT proceed to cleanup — record the discrepancy in T05-RESULT.md and stop. The slice will need replanning.
4. Cleanup (only after step 3 passes):
   a. `gh release delete v1.0.1-test2 --repo chaleyeah/vibe-attack --yes` — deletes the GitHub Release entry.
   b. `git push origin :refs/tags/v1.0.1-test2` — deletes the remote tag.
   c. `git tag -d v1.0.1-test2` — deletes the local tag.
5. Verify cleanup: `git ls-remote --tags origin v1.0.1-test2` should return empty; `gh release view v1.0.1-test2 --repo chaleyeah/vibe-attack 2>&1 | head -1` should report 'release not found' or non-zero exit.

## Must-haves

- All 5 expected asset filenames present on the release (exact string match — note the tilde in the RPM name).
- Release deleted from GitHub.
- Tag deleted from origin and from local.
- `T05-RESULT.md` documents the verified asset list and contains the literal text `cleanup confirmed`.

## Failure modes

- An asset is missing — `softprops/action-gh-release@v2` has `fail_on_unmatched_files: true`, so this should have failed the release job in T04. If it shows up here, something raced. Capture the full asset list to T05-RESULT.md and stop.
- An asset has the wrong version (e.g. `1.0.0` slipped through) — indicates an S01 or T03 fix regressed; flag in T05-RESULT.md and do NOT delete the release so the artifact can be inspected.
- The RPM filename has `-` instead of `~` between the version and `test2` — the T03 RPM_VERSION substitution didn't take effect. Inspect release.yml on the merged commit and check the build log.
- `gh release delete` returns 404 — release was already deleted; treat as success and proceed.
- Tag delete fails because the tag was never pushed — re-check `git tag -l v1.0.1-test2`; if absent, T04 did not actually push.

## Negative tests

N/A — this is verification + cleanup.

## Load profile

A few `gh` API calls and three delete operations. Negligible.

## Observability impact

`T05-RESULT.md` is the durable artifact recording the verified asset names and cleanup state. This is the slice's final verification record.

## Inputs

- `.github/workflows/release.yml`

## Expected Output

- `T05-RESULT.md created with verified asset list`
- `v1.0.1-test2 release deleted from GitHub`
- `v1.0.1-test2 tag deleted from origin and locally`

## Verification

test -f .gsd/milestones/M013/slices/S03/T05-RESULT.md && grep -q 'vibe-attack-v1.0.1-test2-x86_64.AppImage' .gsd/milestones/M013/slices/S03/T05-RESULT.md && grep -q 'vibe-attack-1.0.1~test2-1.x86_64.rpm' .gsd/milestones/M013/slices/S03/T05-RESULT.md && grep -q 'vibe-attack_1.0.1-test2-1_amd64.deb' .gsd/milestones/M013/slices/S03/T05-RESULT.md && grep -q 'cleanup confirmed' .gsd/milestones/M013/slices/S03/T05-RESULT.md && ! git ls-remote --tags origin v1.0.1-test2 2>/dev/null | grep -q refs/tags/v1.0.1-test2
