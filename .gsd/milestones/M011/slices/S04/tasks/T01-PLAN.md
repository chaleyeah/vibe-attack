---
estimated_steps: 2
estimated_files: 5
skills_used: []
---

# T01: Bump version to 1.0.0 across Cargo.toml, spec, PKGBUILD, debian/changelog, and CHANGELOG.md

Atomic version bump across all packaging manifests. Today's date is 2026-04-28 — use that date for the spec %changelog entry, debian/changelog timestamp, and the CHANGELOG.md `## [1.0.0]` heading. The rule is: `grep -rn "0\.1\.0" Cargo.toml packaging/ CHANGELOG.md` must return zero hits afterward. PKGBUILD's `sha256sums=('SKIP', 'SKIP')` MUST remain unchanged per MEM093 — sha256sums are pinned only at AUR submission time, not at version bump time. Do not modify packaging/debian/control (it does not embed a version). For CHANGELOG.md, create a new `## [1.0.0] - 2026-04-28` block above the existing `## [Unreleased]`, and move the current Added/Fixed/Changed entries from Unreleased into the 1.0.0 block; leave `## [Unreleased]` as an empty header for future work.

Why: every other task in this slice and S05 depends on the manifests reading 1.0.0; otherwise release artifact filenames will be wrong (vibe-attack_0.1.0-1_amd64.deb vs the expected 1.0.0) and AppImage rename will pick up the wrong tag.

## Inputs

- ``Cargo.toml``
- ``packaging/vibe-attack.spec``
- ``packaging/PKGBUILD``
- ``packaging/debian/changelog``
- ``CHANGELOG.md``

## Expected Output

- ``Cargo.toml``
- ``packaging/vibe-attack.spec``
- ``packaging/PKGBUILD``
- ``packaging/debian/changelog``
- ``CHANGELOG.md``

## Verification

grep -rn "0\.1\.0" Cargo.toml packaging/ CHANGELOG.md | grep -v 'sherpa-onnx' | grep -v 'silero' ; test $? -ne 0 && grep -q '^version = "1.0.0"' Cargo.toml && grep -q '^Version:        1.0.0' packaging/vibe-attack.spec && grep -q '^pkgver=1.0.0' packaging/PKGBUILD && head -1 packaging/debian/changelog | grep -q '1.0.0-1' && grep -q '## \[1.0.0\] - 2026-04-28' CHANGELOG.md
