---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T03: Run final verification — test, clippy, doc, audit script

Run cargo test, cargo test --features gui, cargo clippy --all-targets -- -D warnings, cargo clippy --all-targets --features gui -- -D warnings, cargo doc --no-deps, and the S03 audit script. All must pass and audit must report 0 undocumented public items in src/.

## Inputs

- `S04 T01 and T02 complete`

## Expected Output

- `Verification log captured for slice summary`

## Verification

All cargo invocations exit 0; audit script reports 0
