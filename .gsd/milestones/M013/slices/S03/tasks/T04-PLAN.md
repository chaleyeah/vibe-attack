---
estimated_steps: 26
estimated_files: 2
skills_used: []
---

# T04: Push v1.0.1-test2 tag and confirm CI + Release workflows finish all-green

Create and push a fresh disposable test tag `v1.0.1-test2` to the `chaleyeah/vibe-attack` GitHub remote. Using `-test2` rather than reusing `-test` ensures no GitHub Actions cache state, artifact name collision, or release-name collision from the prior failed T02 run. The new tag triggers both `.github/workflows/ci.yml` and the now-fixed `.github/workflows/release.yml`. Monitor both runs and confirm every job succeeds.

## Steps

1. From repo root with the T03 commit on HEAD of `main`, create an annotated tag: `git tag -a v1.0.1-test2 -m 'CI/Release pipeline smoke test (round 2 after release.yml fixes)'`.
2. Push the tag: `git push origin v1.0.1-test2`.
3. Wait ~15s for GitHub to register both runs, then list: `gh run list --repo chaleyeah/vibe-attack --event push --limit 5 --json databaseId,name,headBranch,event,status`. The two runs for `refs/tags/v1.0.1-test2` are the targets — capture both run IDs (one for `CI`, one for `Release`).
4. Watch each run to completion sequentially: `gh run watch <CI_RUN_ID> --repo chaleyeah/vibe-attack --exit-status` then `gh run watch <RELEASE_RUN_ID> --repo chaleyeah/vibe-attack --exit-status`. Each blocks until done and exits non-zero on failure. Allow up to 30 minutes total (RPM build was 10m on v1.0.0 with cold cache).
5. After both succeed, confirm per-job status:
   - `gh run view <CI_RUN_ID> --repo chaleyeah/vibe-attack --json jobs --jq '.jobs[] | {name, conclusion}'` → expect `Test`, `Clippy`, `Validate AUR PKGBUILD` all `success`.
   - `gh run view <RELEASE_RUN_ID> --repo chaleyeah/vibe-attack --json jobs --jq '.jobs[] | {name, conclusion}'` → expect `Build AppImage`, `Build Debian package`, `Build RPM package`, `Publish GitHub Release` all `success`.
6. If any job fails: do NOT delete the tag. Capture failing job logs via `gh run view <RUN_ID> --repo chaleyeah/vibe-attack --log-failed`, write a failure note to `.gsd/milestones/M013/slices/S03/T04-FAILURE.md`, and stop. The slice will need replanning.

## Must-haves

- A new annotated tag `v1.0.1-test2` exists locally and on `origin`.
- Both workflow runs (CI and Release) for the tag finish with `conclusion: success`.
- All 7 jobs across both workflows individually report `conclusion: success` (not `skipped`, not `cancelled`).
- The tag is NOT deleted in this task — T05 needs it live to inspect release assets.

## Failure modes

- If RPM build still fails on Version: the bash `${TAG//-/~}` expansion didn't fire — verify the workflow YAML actually contains the RPM_VERSION line and that the run picked up the new commit (check the workflow run's HEAD commit SHA matches).
- If Debian build still fails on Unmet build dependencies: the `-d` flag didn't make it into the dpkg-buildpackage call — check the workflow log for the exact command echoed by the shell.
- If `softprops/action-gh-release@v2` reports `Already_exists` for the tag: a stale release for `v1.0.1-test2` exists from a prior attempt — delete it manually and re-push.
- Network blip during `gh run watch` — re-issue with the same run ID; it picks back up.

## Negative tests

N/A — this is a real-world pipeline run.

## Load profile

One-shot. ~25 GitHub Actions minutes total. Avoid retries unless a fix is needed.

## Observability impact

GH Actions logs and gh CLI introspection are the only inspection surfaces. Per-job conclusion JSON from step 5 is the verification artifact.

## Inputs

- `.github/workflows/ci.yml`
- `.github/workflows/release.yml`

## Expected Output

- `Annotated tag v1.0.1-test2 pushed to origin`
- `CI run all jobs success`
- `Release run all jobs success`

## Verification

git ls-remote --tags origin v1.0.1-test2 | grep -q refs/tags/v1.0.1-test2 && gh run list --repo chaleyeah/vibe-attack --event push --limit 10 --json headBranch,name,conclusion --jq '[.[] | select(.headBranch=="v1.0.1-test2")] | length' | grep -qx 2 && gh run list --repo chaleyeah/vibe-attack --event push --limit 10 --json headBranch,name,conclusion --jq '[.[] | select(.headBranch=="v1.0.1-test2") | .conclusion] | all(. == "success")' | grep -qx true
