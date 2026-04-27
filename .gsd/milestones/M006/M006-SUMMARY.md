---
milestoneId: M006
title: "CI test pipeline"
status: complete
completedAt: "2026-04-26"
commit: af4ccf8
---

# M006 Summary: CI test pipeline

**One-liner:** Automated quality gate — every push and PR now triggers `cargo test` + Clippy in GitHub Actions.

## Narrative

M006 was a single-slice milestone. The test suite already had clean hardware/model separation via `#[ignore]` guards, which made CI-safe test selection straightforward. The workflow delivers two jobs: a `test` job covering lib tests and 9 safe integration tests across both feature sets, and a `clippy` job enforcing zero warnings with `-D warnings`.

The `tags-ignore: ["v*"]` trigger guard prevents double-runs when release tags are pushed (the release workflow covers those). Clippy `-D warnings` is enforced from day one.

## Success criteria results

- CI workflow triggers on every push (non-tag) and every PR ✓
- `cargo test --lib` and safe integration tests run in CI ✓
- Clippy runs with `-D warnings` on both default and gui feature sets ✓
- Hardware-gated tests excluded from standard CI run ✓

## Key files

- `.github/workflows/ci.yml` — two-job CI workflow

## Key decisions

- Explicit per-test invocation to exclude hardware tests without suppressing entire test binary
- `-D warnings` from first commit to prevent lint debt

## Follow-ups

- Add a `cargo test --test concurrency_stress` job gated behind a matrix label if stress coverage is needed
- If Clippy warnings surface on first CI run, address them before merging any new PRs
