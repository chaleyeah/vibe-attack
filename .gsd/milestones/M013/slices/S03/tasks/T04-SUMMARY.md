---
id: T04
parent: S03
milestone: M013
key_files:
  - .github/workflows/release.yml
key_decisions:
  - Used tr '-' '~' instead of bash ${TAG//-/~} to produce RPM-safe version — bash tilde expansion inside parameter substitution replaces literal ~ with home directory
  - Used RPM_VERSION for both git archive --prefix and -o filename so Source0 %{version} reference resolves to the correct tarball
  - Pushed v1.0.1-test4 (not v1.0.1-test2 as planned) after two additional fix iterations — test tags v1.0.1-test2 and v1.0.1-test3 were intermediate failure runs
duration: 
verification_result: passed
completed_at: 2026-05-03T22:32:18.452Z
blocker_discovered: false
---

# T04: Pushed v1.0.1-test4 tag (after fixing two additional RPM bugs found during test runs) and confirmed all 7 CI+Release jobs green; GitHub Release created with AppImage, deb, and RPM assets

**Pushed v1.0.1-test4 tag (after fixing two additional RPM bugs found during test runs) and confirmed all 7 CI+Release jobs green; GitHub Release created with AppImage, deb, and RPM assets**

## What Happened

T04 required three additional fix iterations beyond the T03 changes before the Release workflow ran all-green:

**Round 1 — v1.0.1-test2 (bash tilde expansion bug):** Pushed `v1.0.1-test2` after T03's commit. The Release run immediately failed on `Create source tarball for rpmbuild` with `sed: -e expression #1, char 37: unknown option to 's'`. Investigation revealed that `${TAG//-/~}` in bash expands the `~` replacement string to the runner's home directory (`/home/runner`), making `RPM_VERSION=1.0.1/home/runnertest2` — not the intended `1.0.1~test2`. This is a bash tilde-expansion behavior inside parameter substitution. Fix: replaced `RPM_VERSION="${TAG//-/~}"` with `RPM_VERSION="$(echo "$TAG" | tr '-' '~')"`.

**Round 2 — v1.0.1-test3 (Source0 tarball name mismatch):** Pushed `v1.0.1-test3`. The `Create source tarball` step now passed (RPM_VERSION correct), but `Build RPM package` failed with `Bad source: vibe-attack-1.0.1~test3.tar.gz: No such file or directory`. The RPM spec's `Source0` field uses `%{version}` to locate the tarball, so once `Version: 1.0.1~test3` was stamped, rpmbuild looked for a file named with `~`. But the tarball was still named with `-` (using raw TAG). Fix: updated both `--prefix` and `-o` arguments in `git archive` to use `${RPM_VERSION}` instead of `${TAG}`, so the tarball filename matches what `Source0: ...%{version}...` resolves to. AppImage and Debian both succeeded in this run.

**Round 3 — v1.0.1-test4 (all green):** Pushed `v1.0.1-test4`. All 7 jobs succeeded:
- CI: Validate AUR PKGBUILD (14s), Clippy (success), Test (2m24s)
- Release: Build AppImage (9m55s), Build Debian package (11m23s), Build RPM package (10m36s), Publish GitHub Release (17s)

A GitHub Release for `v1.0.1-test4` was created with all artifacts uploaded. The tag is live on origin for T05 to inspect asset filenames and metadata.

## Verification

CI run 25292448147: all 3 jobs (Validate AUR PKGBUILD, Clippy, Test) = success via `gh run view --json jobs --jq '.jobs[] | {name, conclusion}'`. Release run 25292448158: all 4 jobs (Build AppImage, Build Debian package, Build RPM package, Publish GitHub Release) = success. Tag confirmed on origin: `git ls-remote --tags origin v1.0.1-test4` returned the tag SHA. Two extra fix commits required (bash tilde expansion + Source0 tarball naming) and two additional test tags (v1.0.1-test2, v1.0.1-test3) were needed before v1.0.1-test4 ran clean.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `git push origin v1.0.1-test4` | 0 | ✅ pass — tag v1.0.1-test4 created on origin | 2600ms |
| 2 | `gh run view 25292448147 --json jobs --jq '.jobs[] | {name, conclusion}' (CI)` | 0 | ✅ pass — Validate AUR PKGBUILD: success, Clippy: success, Test: success | 175000ms |
| 3 | `gh run watch 25292448158 --exit-status (Release)` | 0 | ✅ pass — all 4 Release jobs completed successfully | 726000ms |
| 4 | `gh run view 25292448158 --json jobs --jq '.jobs[] | {name, conclusion}'` | 0 | ✅ pass — Build AppImage: success, Build Debian package: success, Build RPM package: success, Publish GitHub Release: success | 1800ms |
| 5 | `git ls-remote --tags origin v1.0.1-test4` | 0 | ✅ pass — tag SHA 0eb3e09e confirmed on origin | 1500ms |

## Deviations

Plan specified v1.0.1-test2 as the final smoke-test tag. Two additional bugs were discovered and fixed during execution (bash tilde expansion, Source0 tarball name mismatch), requiring two more test tags (v1.0.1-test3, v1.0.1-test4). The final successful tag is v1.0.1-test4. Tags v1.0.1-test2 and v1.0.1-test3 are on origin with failed runs; they can be cleaned up by T05. Two additional commits were added to main: 'fix(ci): use tr for RPM tilde substitution to avoid bash tilde expansion bug' and 'fix(ci): use RPM_VERSION for tarball name and prefix to match Source0'.

## Known Issues

Tags v1.0.1-test2 and v1.0.1-test3 remain on origin with failed Release runs. T05 should clean these up along with v1.0.1-test4 (or adjust its scope to clean all three test tags). The spec's Source0 URL uses %{version} which will now contain ~ for pre-release tags — the URL will have a tilde in the archive path, which is valid but differs from what the GitHub URL actually uses (hyphen). This only matters if someone tries to download the source from the URL; the tarball is packaged correctly inside the RPM.

## Files Created/Modified

- `.github/workflows/release.yml`
