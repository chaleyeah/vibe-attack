# S06: GitHub Actions CI release pipeline

**Goal:** GitHub Actions workflow: on tag push build AppImage and upload as release asset
**Demo:** .github/workflows/release.yml committed; workflow passes act dry-run or manual inspection shows correct job/step structure

## Must-Haves

- Not provided.

## Proof Level

- This slice proves: Not provided.

## Integration Closure

Not provided.

## Verification

- Not provided.

## Tasks

- [x] **T01: Create .github/workflows/release.yml** `est:20m`
  Workflow triggers on v* tags, builds AppImage via build.sh, uploads via softprops/action-gh-release
  - Files: `.github/workflows/release.yml`
  - Verify: YAML parses without error; workflow has correct trigger, rust-cache, and upload steps

## Files Likely Touched

- .github/workflows/release.yml
