---
estimated_steps: 18
estimated_files: 5
skills_used: []
---

# T01: Push v1.0.0 tag and verify GitHub Release publishes all artifacts

Push annotated tag `v1.0.0` to origin to trigger `.github/workflows/release.yml`, monitor the run to completion, and verify the resulting GitHub Release contains all five expected artifacts.

**Why:** This is the only action that fires the 4-job release pipeline (build-appimage, build-deb, build-rpm → release). Until the tag is on origin and the workflow succeeds, no v1.0.0 release exists, the AUR PKGBUILD source[0] URL returns 404, and M011/S02-T03's `releases/latest/download/` URL is unreachable.

**Pre-flight:** Confirm working tree is in a clean publishable state. The `.gsd/` directory is gitignored — do NOT stage or commit it. Run `git status --porcelain | grep -v '^?? \.gsd/' | grep -v '^.. \.gsd/'` and confirm zero non-`.gsd` lines (or commit any real source changes first). Confirm `git tag -l v1.0.0` is empty (must not pre-exist locally) and `git ls-remote --tags origin v1.0.0` is empty (must not pre-exist on remote). Confirm `gh auth status` shows `workflow` scope (research already verified this).

**Tag push:**
1. `git tag -a v1.0.0 -m "Release v1.0.0"` (annotated, on current main HEAD).
2. `git push origin v1.0.0`. This MUST succeed — if it fails, do NOT delete the tag without diagnosing; report the error and stop. The push triggers `release.yml` because the workflow's trigger is `push: tags: [v*]`.

**Monitor run:**
3. Wait ~10 seconds, then `gh run list --workflow=release.yml --limit=3 --json databaseId,status,conclusion,headBranch,headSha,event` and identify the run whose `headSha` matches the tagged commit and `event=push`.
4. Watch with `gh run watch <run-id> --exit-status` (or poll `gh run view <run-id> --json status,conclusion` every 60s). Each build job is ~5-10min; release job runs after all three complete. Total expected wall time: 10-15 min.
5. If any build job fails, run `gh run view <run-id> --log-failed | tail -200` to extract the failure, decide whether the failure is (a) transient (re-run via `gh run rerun <run-id>`) or (b) a real defect (file as a blocker; do NOT delete the tag — leave it so the failing run is preserved for forensics). The `release` job is gated by `needs: [build-appimage, build-deb, build-rpm]` so it will not run if any build job fails.

**Verify release published:**
6. `gh release view v1.0.0 --json tagName,isDraft,assets --jq '{tag: .tagName, draft: .isDraft, count: (.assets | length), names: [.assets[].name]}'` — assert `tag==v1.0.0`, `draft==false`, `count >= 5`. The five expected asset name patterns are: `vibe-attack-v1.0.0-x86_64.AppImage`, `vibe-attack-v1.0.0.tar.gz`, `hd2-v1.0.0.hdpack`, `vibe-attack_1.0.0-1_amd64.deb`, `vibe-attack-1.0.0-1.x86_64.rpm` (deb/rpm exact filenames depend on dpkg-buildpackage / rpmbuild output and may differ slightly — match by glob).
7. `curl -sI -L https://github.com/chaleyeah/vibe-attack/releases/latest/download/vibe-attack-x86_64.AppImage | grep -E '^HTTP/'` — expect a final `HTTP/2 200` after redirects (GitHub Releases issues 302→CDN, then 200). HTTP 404 is failure. The trailing `-x86_64.AppImage` suffix matches via GitHub's name-suffix resolver despite the version in the actual asset name.

**Failure modes to handle:**
- `git push origin v1.0.0` rejected for missing `workflow` scope → report and halt; user must re-auth `gh auth refresh -s workflow`.
- Workflow run fails on `fail_on_unmatched_files: true` → exact missing glob is in the action-gh-release step log; this means a build job produced a differently-named artifact than the release-job glob expects. Diagnose and report; do not retry blindly.
- `gh release view v1.0.0` shows `draft: true` → the `softprops/action-gh-release@v2` step ran but defaulted to draft; check workflow inputs and report.

**Note on HEAD cleanliness:** the planning notification flagged untracked `.gsd/` files and a deleted `.gsd/safety/evidence-M011-S04-T03.json`. These are all under `.gsd/` and gitignored — they will not affect what the tag points to. Do NOT `git add .gsd/`. Do NOT `git rm .gsd/...`. Leave them untouched.

## Inputs

- ``.github/workflows/release.yml``
- ``Cargo.toml``
- ``packaging/vibe-attack.spec``
- ``packaging/PKGBUILD``
- ``packaging/debian/changelog``
- ``CHANGELOG.md``

## Expected Output

- ``refs/tags/v1.0.0` (remote git ref on origin)`
- ``gh release view v1.0.0` (live, non-draft, 5+ assets)`
- ``https://github.com/chaleyeah/vibe-attack/releases/latest/download/vibe-attack-x86_64.AppImage` (HTTP 200 via redirect)`

## Verification

git ls-remote --tags origin v1.0.0 prints a ref; gh release view v1.0.0 --json isDraft,assets --jq '.isDraft==false and (.assets|length)>=5' returns true; curl -sI -L https://github.com/chaleyeah/vibe-attack/releases/latest/download/vibe-attack-x86_64.AppImage | tail -1 contains '200'; gh run list --workflow=release.yml --limit=1 --json conclusion --jq '.[0].conclusion' returns 'success'.

## Observability Impact

Touches the GitHub Actions runtime boundary and the GitHub Releases API. Failure signals: `gh run view <id> --log-failed` produces the failing job/step/log lines; `fail_on_unmatched_files: true` (per S04 MEM086) names the missing artifact glob in the release-step log. Inspection: `gh run list --workflow=release.yml` enumerates runs by conclusion; `gh release view v1.0.0 --json assets` enumerates published asset names. No new code-level observability surfaces are added in this task — workflow-level signals are sufficient.
