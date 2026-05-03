---
estimated_steps: 22
estimated_files: 2
skills_used: []
---

# T02: Pushed v1.0.1-test tag and triggered CI+Release workflows; CI passed but Release failed on RPM (hyphen in Version field) and Debian (dpkg build-dep check blocks rustup-installed Rust)

Create and push a disposable test tag `v1.0.1-test` to the `chaleyeah/vibe-attack` GitHub remote. This triggers both `.github/workflows/ci.yml` and `.github/workflows/release.yml` (both watch `tags: ['v*']`). Monitor both workflow runs and confirm every job succeeds. Do NOT delete the tag in this task — T03 needs it live to inspect release assets.

Why `v1.0.1-test`: Debian upstream-version strings allow alphanumerics; `-test` is safer than `rc.1` (which the research warned could be ambiguous for dpkg) and clearly signals disposability. The `${GITHUB_REF_NAME#v}` strip yields `1.0.1-test` for spec/changelog/tarball stamping.

Steps:
1. From repo root with the T01 commit on HEAD of `main`, create an annotated tag: `git tag -a v1.0.1-test -m 'CI/Release pipeline smoke test'`.
2. Push the tag: `git push origin v1.0.1-test`.
3. Wait ~15 seconds for GitHub to register both workflow runs, then list them: `gh run list --repo chaleyeah/vibe-attack --event push --limit 5`. Capture the two run IDs (one for `CI`, one for `Release`).
4. Watch each run to completion. Use `gh run watch <RUN_ID> --repo chaleyeah/vibe-attack --exit-status` for each — this blocks until the run finishes and exits non-zero on failure. The Release run is the longer one (RPM build was 10m16s on v1.0.0). Allow up to 25 minutes total.
5. After both runs report success, run `gh run view <CI_RUN_ID> --repo chaleyeah/vibe-attack --json jobs --jq '.jobs[] | {name, conclusion}'` for the CI run — confirm `Test`, `Clippy`, and `Validate AUR PKGBUILD` all show `conclusion: success`. Repeat for the Release run — confirm `Build AppImage`, `Build Debian package`, `Build RPM package`, and `Publish GitHub Release` all show `conclusion: success`.
6. If any job fails: do NOT delete the tag. Capture the failing job's logs via `gh run view <RUN_ID> --repo chaleyeah/vibe-attack --log-failed`, write a short failure note to `.gsd/milestones/M013/slices/S03/T02-FAILURE.md`, and stop. The slice will need replanning.

Must-haves:
- A new annotated tag `v1.0.1-test` exists locally and on `origin`.
- Both workflow runs (CI and Release) for the tag finish with `conclusion: success`.
- All 7 jobs across both workflows individually report `conclusion: success` (not `skipped`, not `cancelled`).
- The tag is NOT deleted in this task.

Failure modes:
- Push fails due to remote rejection — confirm `git remote -v` points at the correct fork and the user has push permissions.
- A workflow times out at the GitHub default of 6h — both workflows have no custom `timeout-minutes` set; if Rust cache is cold, the RPM job may take 12-15 min but will not hit 6h.
- The Release job fails because `softprops/action-gh-release@v2` rejects an existing draft for the tag — there should not be one; if there is, the v1.0.0 release for tag `v1.0.0` is unrelated and should not interfere.
- Network blip causes `gh run watch` to disconnect — if the run is otherwise progressing, re-issue `gh run watch` with the same run ID.

Negative tests: N/A — this is a real-world pipeline run, not a code-path test.

Load profile: One-shot execution. The two workflow runs together consume ~25 GitHub Actions minutes per attempt. Avoid retries unless a fix is needed.

Observability impact: GitHub Actions logs and gh CLI are the only inspection surfaces. The status of each job is captured by step 5's JSON jq query and is the verification artifact.

## Inputs

- ``.github/workflows/ci.yml``
- ``.github/workflows/release.yml``
- ``packaging/vibe-attack.spec``
- ``packaging/debian/changelog``
- ``packaging/PKGBUILD``

## Expected Output

- ``.gsd/milestones/M013/slices/S03/T02-RESULT.md``

## Verification

gh run list --repo chaleyeah/vibe-attack --event push --limit 2 --json conclusion,name --jq 'map(select(.name=="CI" or .name=="Release")) | length' | grep -qx 2 && gh run list --repo chaleyeah/vibe-attack --event push --limit 2 --json conclusion --jq 'all(.[]; .conclusion=="success")' | grep -qx true && git ls-remote --tags origin v1.0.1-test | grep -q refs/tags/v1.0.1-test

## Observability Impact

Verification leans entirely on GitHub Actions per-job conclusion fields exposed via `gh run view --json jobs`. On failure, capture `gh run view --log-failed` output to `T02-FAILURE.md` so the next planner has the failing step's stderr without re-fetching.
