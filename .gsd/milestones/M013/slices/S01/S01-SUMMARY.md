---
id: S01
parent: M013
milestone: M013
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - (none)
key_decisions:
  - ["sed rewrites spec into SPECS/ dir rather than modifying source, keeping packaging/vibe-attack.spec clean", "changelog prepend preserves history for local builds"]
patterns_established:
  - (none)
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-03T20:46:12.576Z
blocker_discovered: false
---

# S01: Fix release versioning bugs

**All hardcoded 1.0.0 version strings removed from release pipeline; RPM and deb now derive version from git tag**

## What Happened

Two versioning bugs fixed in release.yml. RPM: git archive prefix and SOURCES filename now use TAG=${GITHUB_REF_NAME#v}; spec Version: field rewritten via sed before rpmbuild. DEB: a changelog prepend step stamps the correct version before dpkg-buildpackage runs; dh-cargo removed as it was never exercised by the custom rules overrides.

## Verification

Diff inspection; grep confirms no literal 1.0.0 remains in release workflow steps.

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

None.

## Known Limitations

None.

## Follow-ups

None.

## Files Created/Modified

- `.github/workflows/release.yml` — RPM tarball version fix, deb changelog stamp, dh-cargo removal
