---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T02: Verify CONTRIBUTING.md accuracy

Read CONTRIBUTING.md line-by-line. Verify dev setup commands, test invocations (including hardware-gated ones), code style references, and PR workflow notes match current reality (CI workflow, clippy enforcement). Update any drift.

## Inputs

- `CONTRIBUTING.md and current .github/workflows/ci.yml, Cargo.toml`

## Expected Output

- `CONTRIBUTING.md updated to match current state`

## Verification

Manual review; running the documented dev setup steps works on a fresh clone
