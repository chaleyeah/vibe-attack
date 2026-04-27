---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T05: Run full verification — test, clippy, audit grep

Run cargo test, cargo test --features gui, cargo clippy --all-targets -- -D warnings, cargo clippy --all-targets --features gui -- -D warnings, and `grep -rn '#\[allow(\|unsafe impl\|unsafe fn' src/` to confirm every match has an adjacent justifying comment. All cargo invocations must exit 0.

## Inputs

- `All preceding S02 tasks complete`

## Expected Output

- `Verification log for slice summary; audit grep output captured`

## Verification

All four cargo invocations exit 0; manual review of grep output confirms each unsafe/allow has a comment
