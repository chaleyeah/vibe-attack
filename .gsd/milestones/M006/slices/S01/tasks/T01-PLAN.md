---
milestoneId: M006
sliceId: S01
taskId: T01
title: "Write and push ci.yml"
status: complete
estimate: 30m
---

# T01: Write and push ci.yml

**Description:** Create `.github/workflows/ci.yml` with two jobs: `test` (lib + safe integration tests) and `clippy` (-D warnings, default + gui features). Push to main to trigger the first run.

**Files:** `.github/workflows/ci.yml`

**Inputs:** existing `release.yml` structure, test file inventory (15 tests across `tests/`)

**Expected output:**
- `.github/workflows/ci.yml` committed as `af4ccf8` and pushed to main
- CI jobs triggered on push

**Verify:** `git push` triggers two CI jobs visible in GitHub Actions; both pass.
