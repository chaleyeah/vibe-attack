# M006: CI test pipeline

**Vision:** Every push and pull request to the vibe-attack repo gets a fast, automated quality signal: library tests, safe integration tests, and Clippy lints run in GitHub Actions without requiring hardware access.

## Success Criteria

- CI workflow triggers on every push (non-tag) and every pull request
- cargo test --lib and safe integration tests pass in CI
- Clippy runs with -D warnings on both default and gui feature sets
- Hardware-gated tests (uinput, STT, KWS) are excluded from the standard CI run

## Slices

- [x] **S01: ci.yml — test + clippy on push/PR** `risk:low` `depends:[]`
  > After this: Push commit triggers two green CI jobs (test, clippy) visible in GitHub Actions

## Boundary Map

Not provided.
