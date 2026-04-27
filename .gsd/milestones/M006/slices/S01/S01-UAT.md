---
milestoneId: M006
sliceId: S01
title: "ci.yml — test + clippy on push/PR"
---

# S01 UAT: ci.yml — test + clippy on push/PR

## Acceptance criteria

- [ ] Push to any branch (non-tag) triggers the CI workflow
- [ ] `test` job: both `cargo build` steps succeed (default + gui features)
- [ ] `test` job: `cargo test --lib` passes
- [ ] `test` job: all 9 safe integration tests pass
- [ ] `clippy` job: no warnings with `-D warnings` on default features
- [ ] `clippy` job: no warnings with `-D warnings` on gui features
- [ ] A pull request shows CI status checks from both jobs

## How to verify

1. Open https://github.com/chaleyeah/vibe-attack/actions
2. Find the run triggered by commit `af4ccf8` on `main`
3. Confirm both "Test" and "Clippy" jobs show green
