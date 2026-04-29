---
id: T02
parent: S05
milestone: M011
key_files:
  - packaging/PKGBUILD
key_decisions:
  - v1.0.0 tag is treated as immutable post-publish; force-moving it would invalidate the pinned source[0] sha256 hash — project convention is no re-tagging
duration: 
verification_result: passed
completed_at: 2026-04-29T11:43:47.965Z
blocker_discovered: false
---

# T02: Pinned packaging/PKGBUILD sha256sums to real v1.0.0 release hashes (project tarball + sherpa-onnx prebuilt), making the AUR PKGBUILD publishable

**Pinned packaging/PKGBUILD sha256sums to real v1.0.0 release hashes (project tarball + sherpa-onnx prebuilt), making the AUR PKGBUILD publishable**

## What Happened

T01 had already pushed the v1.0.0 tag and published the GitHub Release with all five artifacts. T02 computed sha256 digests for both source entries and replaced the two 'SKIP' placeholders in packaging/PKGBUILD.

Source[0] (https://github.com/chaleyeah/vibe-attack/archive/v1.0.0.tar.gz) hashed to `da0a2427d4812c274ec5fbaf4fa5dd7e13d4fb0030a484f4e06753b8ff6f4c6c`. Source[1] (https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.12.39/sherpa-onnx-v1.12.39-linux-x64-shared-lib.tar.bz2) hashed to `1b95e49f889dee65310cab832d6181db619ea3ac77ecd60fe8b301028145781c`. Both were substituted into the sha256sums array with the exact quoting and indentation expected by makepkg.

The prior session had already performed this edit and the PKGBUILD on disk already contained both real hashes — no re-editing was needed on resume. Verification confirmed both 64-char hex digests present, no SKIP remaining, and all 15 packaging tests still passing.

Assumption documented per task plan: the v1.0.0 tag is immutable post-publish; a force-move would invalidate source[0]'s hash.

## Verification

1. `grep -oE "'[0-9a-f]{64}'" packaging/PKGBUILD | wc -l` → 2 (both hashes pinned)
2. `! grep -q "'SKIP'" packaging/PKGBUILD` → exit 0 (no SKIP remains)
3. `cargo test --test packaging` → test result: ok. 15 passed; 0 failed
4. `gh release view v1.0.0` → 5 assets, draft: false, tag: v1.0.0

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `grep -oE "'[0-9a-f]{64}'" packaging/PKGBUILD | wc -l` | 0 | ✅ pass — 2 hashes pinned | 50ms |
| 2 | `! grep -q "'SKIP'" packaging/PKGBUILD && echo 'No SKIP found'` | 0 | ✅ pass — no SKIP entries remain | 30ms |
| 3 | `cargo test --test packaging` | 0 | ✅ pass — 15 passed, 0 failed | 450ms |
| 4 | `gh release view v1.0.0 --json tagName,isDraft,assets` | 0 | ✅ pass — 5 assets, non-draft, tag v1.0.0 | 1200ms |

## Deviations

none

## Known Issues

none — AUR submission itself (mkaurball, git push aur) is operator runbook work outside S05 scope

## Files Created/Modified

- `packaging/PKGBUILD`
