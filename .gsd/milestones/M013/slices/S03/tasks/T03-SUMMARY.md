---
id: T03
parent: S03
milestone: M013
key_files:
  - github/workflows/release.yml
key_decisions:
  - Used bash parameter expansion ${TAG//-/~} (not tr or sed) for tilde substitution — it is bash-specific but release.yml runs on ubuntu-latest with bash as default shell, so this is safe and avoids a subprocess
  - Tilde goes only into the spec Version field; tarball prefix and Source0 filename continue using the raw hyphen form to match the git archive output
  - dpkg -d flag is the canonical escape hatch when build-deps are managed outside apt — no changes to debian/control needed
duration: 
verification_result: passed
completed_at: 2026-05-03T21:53:35.276Z
blocker_discovered: false
---

# T03: Fixed RPM version hyphen (tilde substitution) and Debian dpkg build-dep check (-d flag) in release.yml; removed stale v1.0.1-test tag from origin and locally

**Fixed RPM version hyphen (tilde substitution) and Debian dpkg build-dep check (-d flag) in release.yml; removed stale v1.0.1-test tag from origin and locally**

## What Happened

Applied two surgical fixes to `.github/workflows/release.yml` to unblock the Release workflow after the T02 failures:

**Fix 1 — RPM Version hyphen:** In the `Create source tarball for rpmbuild` step (lines 182-188), added `RPM_VERSION="${TAG//-/~}"` after the TAG assignment, then changed the sed expression to substitute `${RPM_VERSION}` instead of `${TAG}` into the spec's Version field. For pre-release tags like `v1.0.1-test2`, this produces `Version: 1.0.1~test2`, which RPM accepts. For clean semver tags like `v1.0.2`, TAG has no hyphens so RPM_VERSION is identical to TAG — no behavior change for production releases.

**Fix 2 — Debian dpkg-checkbuilddeps:** Changed `dpkg-buildpackage -uc -us -b` to `dpkg-buildpackage -uc -us -b -d` in the `Build .deb package` step. The `-d` flag skips dpkg's build-dependency check, which was rejecting the workflow because `cargo`/`rustc` are installed via rustup rather than apt. The workflow already manages Rust installation correctly via `dtolnay/rust-toolchain@stable`.

**Stale tag cleanup:** Confirmed no GitHub Release was created for v1.0.1-test (the release job was skipped because its build-deb and build-rpm dependencies failed). Deleted the tag from origin (`git push origin :refs/tags/v1.0.1-test`) and locally (`git tag -d v1.0.1-test`).

**Commit:** `512e7d0 fix(ci): use tilde for RPM version and skip dpkg build-dep check` — 3 insertions, 2 deletions in release.yml.

## Verification

Ran full T03 verification suite: RPM_VERSION line present in release.yml (`grep -q 'RPM_VERSION="${TAG//-/~}"'` → pass), Debian -d flag present (`grep -q 'dpkg-buildpackage -uc -us -b -d'` → pass), YAML parses cleanly (`python3 yaml.safe_load` → YAML OK), origin tag absent (`git ls-remote --tags origin v1.0.1-test` → empty), local tag absent (`git tag -l v1.0.1-test` → empty). All 5 must-haves satisfied.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `grep -q 'RPM_VERSION="${TAG//-/~}"' .github/workflows/release.yml` | 0 | ✅ pass — RPM tilde substitution line present | 50ms |
| 2 | `grep -q 'dpkg-buildpackage -uc -us -b -d' .github/workflows/release.yml` | 0 | ✅ pass — Debian -d flag present | 40ms |
| 3 | `python3 -c 'import yaml; yaml.safe_load(open(".github/workflows/release.yml"))' && echo YAML OK` | 0 | ✅ pass — YAML parses without error | 120ms |
| 4 | `git push origin :refs/tags/v1.0.1-test` | 0 | ✅ pass — stale tag deleted from origin | 2800ms |
| 5 | `git tag -d v1.0.1-test && git tag -l v1.0.1-test` | 0 | ✅ pass — stale tag deleted locally, tag list empty | 80ms |
| 6 | `gh release view v1.0.1-test --repo chaleyeah/vibe-attack 2>&1 | head -5` | 1 | ✅ expected — 'release not found'; release job was skipped so nothing to delete | 1200ms |

## Deviations

none

## Known Issues

RPM artifact filename for test tags will contain a tilde (e.g. vibe-attack-1.0.1~test2-1.x86_64.rpm). The release.yml glob `vibe-attack-*.x86_64.rpm` matches this form. T05 must verify the tilde filename is present on the GitHub Release asset list.

## Files Created/Modified

- `github/workflows/release.yml`
