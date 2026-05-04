---
id: S03
parent: M013
milestone: M013
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - [".github/workflows/release.yml", "src/ui/pack_editor.rs", ".gsd/milestones/M013/slices/S03/T05-RESULT.md"]
key_decisions:
  - ["Use tr '-' '~' (not bash parameter substitution) for RPM_VERSION to avoid bash tilde home-directory expansion", "Use RPM_VERSION (not raw TAG) for both git archive --prefix and -o filename so Source0 %{version} resolves to the correct tarball", "dpkg-buildpackage -d skips build-dep check — standard escape hatch when deps managed outside dpkg (e.g. rustup)", "GitHub release asset filenames normalize ~ to . — documented behavior, not a defect"]
patterns_established:
  - ["RPM Version tilde substitution pattern: TAG var (raw, hyphen) for git operations; RPM_VERSION var (tilde) for spec Version and tarball name", "CI smoke-test workflow: push disposable vX.Y.Z-testN tag, monitor runs, verify assets, delete tag+release"]
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-03T22:35:21.776Z
blocker_discovered: false
---

# S03: S03: End-to-end CI + Release pipeline smoke test

**CI and Release workflows run all-green end-to-end after fixing RPM Version tilde substitution and Debian dpkg build-dep check; all 5 release assets confirmed; all test tags cleaned up**

## What Happened

S03 validated the full CI + Release pipeline after the S01/S02 changes. Five tasks spanned three days of iterative debugging:

**T01** fixed a clippy `empty_line_after_doc_comments` lint in `pack_editor.rs` (/// → //!) that would have blocked CI.

**T02** pushed v1.0.1-test and discovered two Release workflow failures: (1) RPM `Version:` field cannot contain hyphens — `1.0.1-test` is invalid in RPM spec, and (2) `dpkg-buildpackage` rejects rustup-installed Rust because it checks apt's package index for build dependencies.

**T03** applied two surgical fixes to `release.yml`: added `-d` to `dpkg-buildpackage` to skip build-dep check, and computed `RPM_VERSION` using sed substitution. Cleaned up the stale v1.0.1-test tag.

**T04** required three additional fix iterations before reaching all-green:
- v1.0.1-test2: `${TAG//-/~}` bash parameter expansion replaces `~` with the home directory — fixed with `tr '-' '~'`
- v1.0.1-test3: `Source0: ...%{version}...` in the RPM spec looked for a tarball named with the tilde version, but git archive was still using the raw TAG (hyphen) — fixed by using `RPM_VERSION` for both `--prefix` and `-o` in `git archive`
- v1.0.1-test4: all 7 jobs across CI (Clippy, Test, Validate AUR PKGBUILD) and Release (Build AppImage, Build Debian package, Build RPM package, Publish GitHub Release) ran green

**T05** confirmed all 5 expected assets on the v1.0.1-test4 release (AppImage, tar.gz, hdpack, deb, rpm). Noted that GitHub normalizes `~` to `.` in asset filenames (RPM shows as `vibe-attack-1.0.1.test4-1.x86_64.rpm`), though internal RPM metadata retains the tilde. Deleted the GitHub Release and all four test tags from origin and locally.

## Verification

GitHub Actions: all 7 jobs (Validate AUR PKGBUILD, Clippy, Test, Build AppImage, Build Debian package, Build RPM package, Publish GitHub Release) showed conclusion:success for the v1.0.1-test4 run. Asset verification via `gh release view` confirmed all 5 expected files present with correct version-stamped names. All test tags removed from origin and locally — `git ls-remote --tags origin | grep v1.0.1-test` returns empty.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

Four test tag iterations required instead of one (v1.0.1-test through v1.0.1-test4). Two bugs discovered post-T03: bash tilde expansion in parameter substitution, and Source0/tarball name mismatch when RPM_VERSION differs from TAG. All fixed within S03 scope.

## Known Limitations

None.

## Follow-ups

S04: Add CI/Release status badges to README. Future: note that GitHub normalizes ~ to . in release asset filenames for pre-release RPM packages — may want to document in README or release notes.

## Files Created/Modified

None.
