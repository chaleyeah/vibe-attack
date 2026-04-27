---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T05: Run full verification — test, clippy, success-criteria grep

Run cargo test, cargo test --features gui, cargo clippy --all-targets -- -D warnings, cargo clippy --all-targets --features gui -- -D warnings, and the success-criteria grep (`grep -rn 'hd.linux.voice\|hd_linux_voice\|hd2_linux\|TODO\|FIXME\|HACK\|dead_code\|allow(unused' src/`). All must pass or have explicit justification (the known control/mod.rs TODO about CancellationToken is the only acceptable remaining hit).

## Inputs

- `All preceding tasks complete`

## Expected Output

- `Verification log captured for the slice summary; clean test/clippy runs on both feature sets; grep audit recorded`

## Verification

All four cargo invocations exit 0; grep returns at most one hit (the documented control/mod.rs TODO); record the grep output in the slice summary
