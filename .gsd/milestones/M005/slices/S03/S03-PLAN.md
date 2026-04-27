# S03: Debian package

**Goal:** Debian package directory with control, rules, changelog, compat, copyright
**Demo:** dpkg-deb or dpkg-buildpackage produces vibe-attack_0.1.0_amd64.deb; dpkg --info shows correct metadata

## Must-Haves

- Not provided.

## Proof Level

- This slice proves: Not provided.

## Integration Closure

Not provided.

## Verification

- Not provided.

## Tasks

- [x] **T01: Create packaging/debian/ directory** `est:20m`
  control, rules, changelog, compat, copyright files for dpkg-buildpackage
  - Files: `packaging/debian/control`, `packaging/debian/rules`, `packaging/debian/changelog`, `packaging/debian/compat`, `packaging/debian/copyright`
  - Verify: packaging/debian/control has correct Depends; rules is executable

## Files Likely Touched

- packaging/debian/control
- packaging/debian/rules
- packaging/debian/changelog
- packaging/debian/compat
- packaging/debian/copyright
