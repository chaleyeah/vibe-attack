---
estimated_steps: 56
estimated_files: 1
skills_used: []
---

# T03: Fix RPM hyphen and Debian build-dep blockers in release.yml; clean up stale v1.0.1-test tag/release

Apply two surgical fixes to `.github/workflows/release.yml` to unblock the Release workflow, then remove the stale `v1.0.1-test` tag and any partial release left over from the failed T02 run.

## Fix 1: RPM Version field cannot contain hyphens

In the `Build RPM package` job (the `Create source tarball for rpmbuild` step around lines 182-188), compute an RPM-safe version by substituting `-` with `~` (RPM's pre-release comparator). The tarball prefix and source filename must continue to use the raw `${GITHUB_REF_NAME#v}` (with hyphen) since `git archive --prefix` and the Source0 reference use the literal tag-derived name. Only the spec `Version:` field needs the substitution.

Replace the existing step body with:
```yaml
      - name: Create source tarball for rpmbuild
        run: |
          TAG="${GITHUB_REF_NAME#v}"
          RPM_VERSION="${TAG//-/~}"
          git archive --format=tar.gz --prefix="vibe-attack-${TAG}/" HEAD \
            -o ~/rpmbuild/SOURCES/vibe-attack-${TAG}.tar.gz
          sed "s/^Version:.*/Version:        ${RPM_VERSION}/" packaging/vibe-attack.spec \
            > ~/rpmbuild/SPECS/vibe-attack.spec
```

The `${TAG//-/~}` bash parameter expansion replaces ALL hyphens with tildes. For a clean release tag (e.g. `v1.0.2`), TAG=`1.0.2` has no hyphens and RPM_VERSION=`1.0.2` — no behavior change. For pre-release tags like `v1.0.1-test`, RPM_VERSION becomes `1.0.1~test`, which RPM accepts.

Note: the resulting RPM filename will be `vibe-attack-1.0.1~test2-1.x86_64.rpm` (tilde) instead of `vibe-attack-1.0.1-test2-1.x86_64.rpm` (hyphen). The release.yml glob `vibe-attack-*.x86_64.rpm` already matches both forms. T05 must verify the tilde form.

## Fix 2: Debian dpkg-buildpackage rejects rustup-installed Rust

In the `Build .deb package` step (around lines 134-138), add the `-d` flag to `dpkg-buildpackage` to skip `dpkg-checkbuilddeps`. The workflow already installs Rust via the `dtolnay/rust-toolchain@stable` action; there is no value in dpkg re-checking against apt's package index.

Replace:
```yaml
          dpkg-buildpackage -uc -us -b
```
with:
```yaml
          dpkg-buildpackage -uc -us -b -d
```

The `-d` flag (per `man dpkg-buildpackage`) is documented as: 'Do not check build dependencies and conflicts.' This is the standard escape hatch when build deps are managed outside dpkg.

## Cleanup of stale T02 artifacts

The v1.0.1-test tag from T02 is still live on origin (per T02 protocol). The Release workflow ran far enough to upload AppImage+deb+rpm artifacts to the workflow run, but the `release` job (Publish GitHub Release) requires `needs: [build-appimage, build-deb, build-rpm]` — since build-deb and build-rpm both failed, the release job was skipped and no GitHub Release was created. Verify this and clean up.

## Steps

1. Read `.github/workflows/release.yml` lines 119-138 (deb job) and lines 182-192 (rpm job) to confirm the current state matches what's described above.
2. Apply the RPM fix via Edit on the `Create source tarball for rpmbuild` step. The new step adds two lines (`TAG=` and `RPM_VERSION=`) and changes the sed expression to use `${RPM_VERSION}`.
3. Apply the Debian fix via Edit, changing `dpkg-buildpackage -uc -us -b` to `dpkg-buildpackage -uc -us -b -d`.
4. Verify the file parses as valid YAML (no syntax errors): `python3 -c 'import yaml; yaml.safe_load(open(".github/workflows/release.yml"))' && echo OK`.
5. Check that no GitHub Release exists for v1.0.1-test: `gh release view v1.0.1-test --repo chaleyeah/vibe-attack 2>&1 | head -5`. Expected: 'release not found' (release job was skipped). If a release does exist, delete it: `gh release delete v1.0.1-test --repo chaleyeah/vibe-attack --yes`.
6. Delete the stale tag from origin: `git push origin :refs/tags/v1.0.1-test`.
7. Delete the stale tag locally: `git tag -d v1.0.1-test`.
8. Stage and commit the workflow fix: `git add .github/workflows/release.yml && git commit -m 'fix(ci): use tilde for RPM version and skip dpkg build-dep check'`.

## Must-haves

- `.github/workflows/release.yml` `Create source tarball for rpmbuild` step computes `RPM_VERSION="${TAG//-/~}"` and passes it via the sed substitution to the spec Version field.
- `.github/workflows/release.yml` `Build .deb package` step calls `dpkg-buildpackage -uc -us -b -d`.
- The file is valid YAML.
- `git ls-remote --tags origin v1.0.1-test` returns empty (tag removed from origin).
- `git tag -l v1.0.1-test` returns empty (tag removed locally).
- One new commit on `main` with both fixes.

## Failure modes

- Bash parameter expansion `${TAG//-/~}` is bash-specific. The workflow `run:` block defaults to `bash` on ubuntu-latest, so this is fine. Do NOT switch to `sh`.
- The sed expression must NOT escape the `~` — RPM accepts it literally in the Version field.
- If `gh release delete` returns non-zero because no release exists, treat that as success and proceed.
- If `git push origin :refs/tags/v1.0.1-test` fails because the tag was already deleted, treat as success.

## Negative tests

N/A — this is a workflow config change. The real test is T04 pushing a fresh tag.

## Load profile

Local edits + a few `gh`/`git` cleanup calls. Negligible.

## Observability impact

None at runtime — these are CI workflow changes. The cleanup is durable: stale tag is removed so T04 can push a fresh tag without name collision.

## Inputs

- `.github/workflows/release.yml`
- `.gsd/milestones/M013/slices/S03/tasks/T02-SUMMARY.md`

## Expected Output

- `Updated .github/workflows/release.yml with RPM tilde substitution and dpkg -d flag`
- `Stale v1.0.1-test tag removed from origin and local`
- `One new commit on main`

## Verification

grep -q 'RPM_VERSION="\${TAG//-/~}"' .github/workflows/release.yml && grep -q 'dpkg-buildpackage -uc -us -b -d' .github/workflows/release.yml && python3 -c 'import yaml; yaml.safe_load(open(".github/workflows/release.yml"))' && ! git ls-remote --tags origin v1.0.1-test 2>/dev/null | grep -q refs/tags/v1.0.1-test && ! git tag -l v1.0.1-test | grep -q v1.0.1-test
