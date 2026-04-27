# S05: RPM spec file

**Goal:** RPM spec file for Fedora/RHEL with correct metadata and both binaries
**Demo:** rpmbuild -bs vibe-attack.spec produces a .src.rpm without error (source RPM proves spec is syntactically valid)

## Must-Haves

- Not provided.

## Proof Level

- This slice proves: Not provided.

## Integration Closure

Not provided.

## Verification

- Not provided.

## Tasks

- [x] **T01: Create packaging/vibe-attack.spec** `est:15m`
  RPM spec with Name/Version/License/URL, both binaries, icon, .desktop, %changelog
  - Files: `packaging/vibe-attack.spec`
  - Verify: spec file has correct Name/Version/License/URL and both binaries in %files

## Files Likely Touched

- packaging/vibe-attack.spec
