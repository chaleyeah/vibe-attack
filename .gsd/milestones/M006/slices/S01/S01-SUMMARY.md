---
milestoneId: M006
sliceId: S01
title: "ci.yml — test + clippy on push/PR"
status: complete
completedAt: "2026-04-26"
commit: af4ccf8
---

# S01 Summary: ci.yml — test + clippy on push/PR

**One-liner:** GitHub Actions CI workflow delivering automated test + Clippy quality gates on every push and PR.

## Narrative

Inventoried the test suite to separate CI-safe tests from hardware-gated ones. Wrote a two-job `ci.yml`: the `test` job builds both feature sets and runs `--lib` plus nine safe integration tests by name; the `clippy` job lints both feature sets with `-D warnings`. Hardware/model tests (`uinput_smoke`, `macro_inject`, `stt_smoke`, `wake_word`, `daemon_headless`, `concurrency_stress`) are excluded from standard CI — they already carry `#[ignore]` guards. Pushed to main as commit `af4ccf8`.

## What was verified

- `ci.yml` committed and pushed
- Workflow triggers on the push (GitHub Actions picks up `on: push` events)
- `tags-ignore: ["v*"]` ensures release tag pushes don't double-trigger

## Patterns established

- CI-safe integration tests are enumerated explicitly; adding a new hardware test requires also adding it to the exclusion rationale
- Clippy `-D warnings` enforced from first commit to prevent lint debt
