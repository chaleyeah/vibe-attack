# S04: README badges and documentation cleanup

**Goal:** Surface CI and Release pipeline status in the README
**Demo:** README shows two rendered badge images linking to the correct Actions workflow runs

## Must-Haves

- Both badges render with correct status after a tag push; URLs reference correct repo and workflow filenames

## Proof Level

- This slice proves: visual inspection of rendered README

## Integration Closure

README-only change

## Verification

- Public build status visible to all repo visitors

## Tasks

- [x] **T01: Add CI and Release badges to README** `est:5m`
  Add GitHub Actions badge markdown for ci.yml and release.yml below the h1 title
  - Files: `README.md`
  - Verify: README.md contains two badge image links pointing to ci.yml and release.yml workflow runs

## Files Likely Touched

- README.md
