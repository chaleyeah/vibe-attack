---
id: T01
parent: S03
milestone: M005
key_files:
  - packaging/debian/control
  - packaging/debian/rules
  - packaging/debian/changelog
  - packaging/debian/compat
  - packaging/debian/copyright
key_decisions:
  - (none)
duration: 
verification_result: untested
completed_at: 2026-04-27T00:40:13.632Z
blocker_discovered: false
---

# T01: packaging/debian/ created with control, rules, changelog, compat, copyright

**packaging/debian/ created with control, rules, changelog, compat, copyright**

## What Happened

Created all five required Debian packaging files. control: Section=games, Depends on libasound2, BuildDepends includes libasound2-dev and libclang-dev. rules: two cargo build steps (daemon + --features gui), installs both binaries and SVG icon. changelog, compat (13), copyright (AGPL-3.0-only) all present. rules is executable (0775).

## Verification

packaging/debian/control has Depends: libasound2; rules is executable; both binaries in override_dh_auto_install

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| — | No verification commands discovered | — | — | — |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `packaging/debian/control`
- `packaging/debian/rules`
- `packaging/debian/changelog`
- `packaging/debian/compat`
- `packaging/debian/copyright`
