---
estimated_steps: 1
estimated_files: 4
skills_used: []
---

# T01: Bumped version to 1.1.0 and updated all tracker documents

Bump Cargo.toml to 1.1.0. Update debian/changelog with 1.1.0 entry. Update CHANGELOG.md with 1.1.0 section. Update PROJECT.md: add M014 to milestone table, move PACK-05 and MCRO-04 to Validated, remove them from Active.

## Inputs

- `Cargo.toml`
- `CHANGELOG.md`
- `.gsd/PROJECT.md`

## Expected Output

- `Cargo.toml`
- `packaging/debian/changelog`
- `CHANGELOG.md`
- `.gsd/PROJECT.md`

## Verification

cargo build --features gui shows 'vibe-attack v1.1.0'. cargo test passes.

## Observability Impact

Version string visible in config window UI footer.
