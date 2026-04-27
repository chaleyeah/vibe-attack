---
milestoneId: M006
sliceId: S01
taskId: T01
title: "Write and push ci.yml"
status: complete
completedAt: "2026-04-26"
commit: af4ccf8
---

# T01 Summary: Write and push ci.yml

**One-liner:** Created `.github/workflows/ci.yml` with test + clippy jobs and pushed to main, triggering the first CI run.

## What happened

Surveyed the 15 integration tests in `tests/` to identify which are CI-safe (no hardware, no models, no `/dev/uinput`). Hardware-gated tests already use `#[ignore]` with env-var guards (`RUN_PRIVILEGED_TESTS`, `RUN_STT_TESTS`, `RUN_KWS_TESTS`), and `macro_inject.rs` explicitly documents the CI strategy.

Wrote a two-job workflow:
- **test**: `cargo build` (default + gui), `cargo test --lib`, then explicit `cargo test --test <name>` for each of the 9 safe integration tests
- **clippy**: `cargo clippy --all-targets -- -D warnings` for both feature sets

Committed as `af4ccf8` and pushed to main.

## Key decisions

- Explicit per-test invocation (not `cargo test --tests`) to avoid running `uinput_smoke`, `macro_inject`, `stt_smoke`, `wake_word`, `daemon_headless`, `concurrency_stress` which need hardware or the built binary
- `tags-ignore: ["v*"]` on the push trigger so release tag pushes don't double-run (release.yml handles those)
- `-D warnings` on Clippy from day one to avoid lint debt accumulation

## Files modified

- `.github/workflows/ci.yml` — created
