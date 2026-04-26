---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Scaffold script with CLI parsing and step registry

Create scripts/setup.sh with bash strict mode (set -euo pipefail), a --help flag, a --yes/-y flag for non-interactive mode, and a --dry-run flag. Define a step registry as an ordered array of step names. Implement a run_step function that prints '[step] ...' status, checks if already satisfied, and either skips or executes. Make the script executable.

## Inputs

- `config.example.yaml (to know target path)`
- `docs/uinput-setup.md (uinput instructions to mirror)`

## Expected Output

- `scripts/setup.sh exists and is executable`
- `--help prints step list and flags`
- `bash -n scripts/setup.sh exits 0`

## Verification

bash scripts/setup.sh --help exits 0 and prints usage; bash -n scripts/setup.sh passes syntax check
