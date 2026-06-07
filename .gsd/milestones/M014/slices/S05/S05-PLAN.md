# S05: Tracker cleanup and v1.1.0 release

**Goal:** Bump version to 1.1.0, update CHANGELOG, mark PACK-05 and MCRO-04 validated in PROJECT.md.
**Demo:** cargo build --features gui and version string in UI footer shows 1.1.0.

## Must-Haves

- Complete the planned slice outcomes.

## Verification

- Run the task and slice verification checks for this slice.

## Tasks

- [x] **T01: Bumped version to 1.1.0 and updated all tracker documents** `est:15 min`
  Bump Cargo.toml to 1.1.0. Update debian/changelog with 1.1.0 entry. Update CHANGELOG.md with 1.1.0 section. Update PROJECT.md: add M014 to milestone table, move PACK-05 and MCRO-04 to Validated, remove them from Active.
  - Files: `Cargo.toml`, `packaging/debian/changelog`, `CHANGELOG.md`, `.gsd/PROJECT.md`
  - Verify: cargo build --features gui shows 'vibe-attack v1.1.0'. cargo test passes.

## Files Likely Touched

- Cargo.toml
- packaging/debian/changelog
- CHANGELOG.md
- .gsd/PROJECT.md
