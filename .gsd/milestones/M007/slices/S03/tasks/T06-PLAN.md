---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T06: Run audit script and quality spot-check

Run the M007-RESEARCH.md Python audit script against src/ — must report 0 undocumented public items. Then randomly select 10 newly-documented pub items, read each /// comment, and confirm it explains the item's purpose (why) not just restates the name (what). If any are superficial, revise. Run cargo test, cargo test --features gui, cargo clippy --all-targets -- -D warnings, cargo clippy --all-targets --features gui -- -D warnings, cargo doc --no-deps. All must pass.

## Inputs

- `All preceding S03 tasks complete`

## Expected Output

- `Audit script output recorded as evidence; spot-check notes for 10 items captured in slice summary`

## Verification

Audit script output is '0 undocumented public items'; spot-check log (10 items reviewed) is captured in slice summary; all cargo invocations exit 0
