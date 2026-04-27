---
id: T02
parent: S05
milestone: M007
key_files:
  - CONTRIBUTING.md
key_decisions:
  - Listed libclang-dev as required dep for Debian/Ubuntu because sherpa-onnx-sys bindgen requires it (matches CI)
  - Expanded module list to reflect actual src/ layout so contributors know where to look for each subsystem
duration: 
verification_result: passed
completed_at: 2026-04-27T12:26:23.459Z
blocker_discovered: false
---

# T02: Updated CONTRIBUTING.md to add missing libclang-dev prerequisite, correct clippy flags to match CI (-D warnings), and expand module list to include stt, ui, vad, wake

**Updated CONTRIBUTING.md to add missing libclang-dev prerequisite, correct clippy flags to match CI (-D warnings), and expand module list to include stt, ui, vad, wake**

## What Happened

Read CONTRIBUTING.md line-by-line and cross-checked against the CI workflow (.github/workflows/ci.yml), Cargo.toml features, and actual src/ module layout.

Three concrete drifts were found and corrected:

1. **Missing libclang-dev prerequisite** — The CI installs `libclang-dev` (alongside `libasound2-dev`) because sherpa-onnx-sys requires it for its bindgen-based build. CONTRIBUTING.md only mentioned `libasound2-dev`. Updated to list both packages for Debian/Ubuntu, and added `clang` alongside `alsa-lib` for Arch.

2. **Clippy flags understated** — CONTRIBUTING.md said `cargo clippy` but the CI enforces `cargo clippy --all-targets -- -D warnings` (warnings-as-errors). Updated the PR Process step to match the actual enforcement level so contributors aren't surprised by CI failures after a local clippy pass.

3. **Module list incomplete** — The Coding Conventions section listed 7 modules (`audio`, `input`, `pipeline`, `pack`, `control`, `config`, `tui`) but the actual src/ tree has 11: `audio`, `vad`, `wake`, `stt`, `input`, `pipeline`, `pack`, `control`, `config`, `ui`, `tui`. Added the four missing modules in pipeline order (vad → wake → stt, then ui alongside tui).

Everything else in CONTRIBUTING.md was accurate: build commands (cargo build / --features gui / --features stt / --release), test invocation (cargo test), the architecture diagram (Microphone → VAD → STT → Dispatch → Key injection), the VAD/PTT description, the spawn_blocking convention for STT, anyhow/thiserror error handling conventions, and the PR workflow steps.

## Verification

Ran cargo test --lib (40 passed, 0 failed) and cargo test --test documentation (11 passed, 0 failed) to confirm no regressions. Clippy is not installed in this local environment (rustup is absent), but the CI workflow enforces it — the changes made are documentation-only and do not affect Rust source.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --lib` | 0 | ✅ pass | 3200ms |
| 2 | `cargo test --test documentation` | 0 | ✅ pass | 800ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `CONTRIBUTING.md`
