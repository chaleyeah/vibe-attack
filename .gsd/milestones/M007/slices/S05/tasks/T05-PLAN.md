---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T05: Run final M007 milestone verification

Run the complete M007 verification gate: cargo test, cargo test --features gui, cargo clippy --all-targets -- -D warnings, cargo clippy --all-targets --features gui -- -D warnings, cargo doc --no-deps, the M007-RESEARCH.md Python audit script (must report 0), and the success-criteria grep `grep -rn 'hd.linux.voice\|hd_linux_voice\|hd2_linux\|TODO\|FIXME\|HACK\|dead_code\|allow(unused' src/` (must return 0 unjustified hits; the control/mod.rs CancellationToken TODO is the only acceptable remaining hit if not yet addressed). Capture all output for the milestone summary.

## Inputs

- `All preceding S05 tasks complete; S01–S04 complete`

## Expected Output

- `Full milestone verification log for M007-SUMMARY.md and M007-VALIDATION.md`

## Verification

All cargo invocations exit 0; audit script reports 0; grep returns 0 or only the documented control/mod.rs TODO
