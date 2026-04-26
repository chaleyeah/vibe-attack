---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T05: Implement validation step and final summary

Add a validate step as the final step: re-check all four conditions (config file exists, model file exists and non-empty, /dev/uinput accessible, ptt.key set in config -- use grep). Print a summary table with pass/fail for each check. Exit 0 only if all four pass. Exit 1 with a clear message listing which checks failed. Add a SETUP COMPLETE banner on success.

## Inputs

- `scripts/setup.sh (T01-T04 output)`

## Expected Output

- `Validation table shows pass/fail per check`
- `Exit 0 with SETUP COMPLETE when all pass`
- `Exit 1 with named failures when any check fails`
- `ptt.key check uses grep on the config file`

## Verification

Run full script with --yes in a temp env where all steps succeed: validation prints all-pass table and SETUP COMPLETE; manually break one condition (remove config file) and re-run validate alone: prints specific failure and exits 1
