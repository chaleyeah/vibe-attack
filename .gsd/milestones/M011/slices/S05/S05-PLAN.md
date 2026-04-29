# S05: Publish GitHub Release v1.0.0

**Goal:** Publish GitHub Release v1.0.0 with all four artifacts (AppImage, source tarball, .deb, .rpm) by pushing the v1.0.0 tag and confirming the release.yml workflow succeeds; then pin real sha256sums into packaging/PKGBUILD so the AUR submission is publishable.
**Demo:** GitHub Releases `v1.0.0` is live with all four artifacts; AUR PKGBUILD sha256sums pinned to real release hashes.

## Must-Haves

- After this slice: `gh release view v1.0.0` returns a non-draft release with at least 5 assets matching the planned globs; `curl -sI https://github.com/chaleyeah/vibe-attack/releases/latest/download/vibe-attack-x86_64.AppImage` returns 200/302 (not 404); `grep sha256sums packaging/PKGBUILD` shows two 64-char hex strings (no `'SKIP'`); `cargo test --test packaging` still reports 15 passed.

## Proof Level

- This slice proves: - This slice proves: final-assembly
- Real runtime required: yes (GitHub Actions runs the release workflow against a real tag)
- Human/UAT required: no (gh CLI is sufficient to verify; downstream final-distro UAT in M011/S02-T03 is human-bound but out of S05 scope)

## Integration Closure

- Upstream surfaces consumed: `.github/workflows/release.yml` (already complete from S04), `packaging/PKGBUILD` (sha256sums=SKIP from S04), `Cargo.toml` / `vibe-attack.spec` / `debian/changelog` / `CHANGELOG.md` (already at 1.0.0 from S04).
- New wiring introduced in this slice: a real git tag `v1.0.0` on origin/main triggers the release workflow for the first time on this repository; this is the first end-to-end exercise of the 4-job release pipeline.
- What remains before the milestone is truly usable end-to-end: M011/S02-T03 (final-distro UAT loop on debian13/ubuntu2604/fedora44/cachyos VMs) — human-bound, unblocked by S05's published release URL.

## Verification

- Runtime signals: GitHub Actions workflow run logs (per-job stdout/stderr); release workflow `fail_on_unmatched_files: true` produces a loud failure with the missing glob name if any build job's artifact doesn't match.
- Inspection surfaces: `gh run list --workflow=release.yml`, `gh run view <id> --log-failed`, `gh release view v1.0.0 --json assets`, `curl -sI <release-asset-url>`.
- Failure visibility: workflow run id, failed job name, failed step name, and the exact missing glob (from action-gh-release error output) are all surfaced by `gh run view --log-failed`.
- Redaction constraints: none — no secrets in release artifacts; GITHUB_TOKEN is automatic.

## Tasks

- [ ] **T01: Push v1.0.0 tag and verify GitHub Release publishes all artifacts** `est:20m`
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
  - Files: `.github/workflows/release.yml`, `Cargo.toml`, `packaging/vibe-attack.spec`, `packaging/debian/changelog`, `packaging/PKGBUILD`
  - Verify: git ls-remote --tags origin v1.0.0 prints a ref; gh release view v1.0.0 --json isDraft,assets --jq '.isDraft==false and (.assets|length)>=5' returns true; curl -sI -L https://github.com/chaleyeah/vibe-attack/releases/latest/download/vibe-attack-x86_64.AppImage | tail -1 contains '200'; gh run list --workflow=release.yml --limit=1 --json conclusion --jq '.[0].conclusion' returns 'success'.

- [ ] **T02: Pin packaging/PKGBUILD sha256sums to real v1.0.0 release hashes** `est:15m`
  Replace the two `'SKIP'` entries in `packaging/PKGBUILD`'s `sha256sums` array with real sha256 hex digests for source[0] (the project tarball at `https://github.com/chaleyeah/vibe-attack/archive/v1.0.0.tar.gz`) and source[1] (the sherpa-onnx 1.12.39 prebuilt linux-x64 shared-lib tarball at `https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.12.39/sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2`).

**Why:** AUR submissions cannot publish with `sha256sums=('SKIP','SKIP')` — `makepkg` requires real digests so downstream users get integrity verification on source fetch. This task closes the AUR-readiness gap left by S04 (which intentionally deferred this until a real tag was live, per MEM093). It is the final piece of M011's distribution-readiness goal.

**Pre-condition:** T01 must have succeeded — the release at `https://github.com/chaleyeah/vibe-attack/archive/v1.0.0.tar.gz` MUST return HTTP 200. Verify with `curl -sI -L https://github.com/chaleyeah/vibe-attack/archive/v1.0.0.tar.gz | grep -E '^HTTP/' | tail -1` — expect 200. If it returns 404, T01 has not actually completed; stop and report.

**Compute hashes:**
1. Project tarball: `curl -sL https://github.com/chaleyeah/vibe-attack/archive/v1.0.0.tar.gz | sha256sum` — capture the 64-char hex prefix. Note: GitHub's auto-generated `archive/v<tag>.tar.gz` is byte-deterministic per (commit-sha, format), so this hash is stable as long as the tag is not force-moved. (We will not force-move it.)
2. sherpa-onnx tarball: `curl -sL https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.12.39/sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2 | sha256sum` — capture the 64-char hex prefix. This is a fixed upstream artifact, sha256 is stable.

**Edit PKGBUILD:**
3. Open `packaging/PKGBUILD` lines 21-22:
   ```
   sha256sums=('SKIP'
               'SKIP')
   ```
   Replace with:
   ```
   sha256sums=('<project-hash>'
               '<sherpa-onnx-hash>')
   ```
   Preserve indentation and quoting style — `pacman` parses this as a bash array; quotes must remain single quotes; the second entry's leading whitespace must align under the first's opening quote per PKGBUILD convention. Match the exact column the existing `'SKIP'` entries occupy.
4. Do NOT change any other field in the file — `pkgname`, `pkgver`, `source`, etc. remain untouched.

**Verify:**
5. `grep -A1 '^sha256sums' packaging/PKGBUILD` — the two array entries must each be 64 hex chars (regex `^[0-9a-f]{64}$`), no `SKIP`, no truncation.
6. `cargo test --test packaging` — must continue to report 15 passed (S04 baseline). The packaging tests do not currently assert PKGBUILD sha256 content, so they should remain unaffected; this check guards against accidental damage to other PKGBUILD-related assertions.
7. (Optional sanity, not required for done) — re-run `curl -sL <url> | sha256sum` and compare; the digest must match what's now in PKGBUILD.

**Failure modes to handle:**
- Project archive curl returns 404 → T01 is not complete; stop.
- sherpa-onnx URL returns non-200 (upstream removed the asset) → unlikely; if it happens, report as a real blocker — pinning a fake hash would brick the AUR build.
- A future force-move of the v1.0.0 tag would invalidate the project hash. Document the assumption: tags are immutable post-publish (project convention); we are NOT planning to re-tag.

**Out of scope:** Actually submitting the PKGBUILD to AUR (`mkaurball`, `git push aur`) is operator runbook work documented in `docs/distribution-proofs/aur/README.md` and is M011/S02 (or later) territory, not S05. S05 only pins the hashes so the PKGBUILD is publishable.
  - Files: `packaging/PKGBUILD`
  - Verify: grep -E "^\s*'[0-9a-f]{64}'" packaging/PKGBUILD | wc -l reports 2; ! grep -q "'SKIP'" packaging/PKGBUILD; cargo test --test packaging reports 'test result: ok. 15 passed'.

## Files Likely Touched

- .github/workflows/release.yml
- Cargo.toml
- packaging/vibe-attack.spec
- packaging/debian/changelog
- packaging/PKGBUILD
