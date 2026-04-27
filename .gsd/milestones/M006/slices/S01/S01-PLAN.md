---
milestoneId: M006
sliceId: S01
title: "ci.yml — test + clippy on push/PR"
status: complete
---

# S01: ci.yml — test + clippy on push/PR

**Goal:** Add `.github/workflows/ci.yml` that runs library tests, safe integration tests, and Clippy on every push and PR.

**Success criteria:** Both test and clippy jobs complete successfully on the main branch push.

**Proof level:** CI run observable in GitHub Actions at github.com/chaleyeah/vibe-attack/actions.

**Integration closure:** Workflow is self-contained; no other system changes required.

**Observability impact:** GitHub Actions status visible on every push and PR.

## Tasks

- [x] **T01: Write and push ci.yml** `est:30m`
