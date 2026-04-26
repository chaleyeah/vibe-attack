# S01: Setup script

**Goal:** Deliver a self-contained `scripts/setup.sh` that automates every manual prerequisite: copies config.example.yaml to the XDG config path, downloads the whisper model, loads the uinput kernel module, adds the user to the input group, and validates results. Supports --yes for non-interactive use and is idempotent on re-run.
**Demo:** Running `scripts/setup.sh` on a fresh system (or with --yes in a temp dir) completes all steps and exits 0; re-running is idempotent

## Must-Haves

- Script runs to completion with --yes against a temp XDG dir; each step is idempotent; --help exits 0; uinput and group steps are skipped when already satisfied; failed steps print a clear error naming the step and exit non-zero

## Proof Level

- This slice proves: Manual run with --yes in a scratch environment; automated check that --help exits 0

## Integration Closure

Script is the ground truth for setup logic; step names and exit codes will be referenced by S02 probe and S03 wizard

## Verification

- Each step prints a status line to stdout; failures include the step name and reason; --dry-run flag shows what would be done without executing

## Tasks

- [x] **T01: Scaffold script with CLI parsing and step registry** `est:30m`
  Create scripts/setup.sh with bash strict mode (set -euo pipefail), a --help flag, a --yes/-y flag for non-interactive mode, and a --dry-run flag. Define a step registry as an ordered array of step names. Implement a run_step function that prints '[step] ...' status, checks if already satisfied, and either skips or executes. Make the script executable.
  - Files: `scripts/setup.sh`
  - Verify: bash scripts/setup.sh --help exits 0 and prints usage; bash -n scripts/setup.sh passes syntax check

- [x] **T02: Implement copy-config step** `est:20m`
  Add the copy_config step: determine XDG config path ($XDG_CONFIG_HOME or ~/.config), target is $xdg_config/vibe-attack/config.yaml. If the file already exists, print 'already exists, skipping' and return 0. Otherwise mkdir -p the directory and cp config.example.yaml to the target. In non-interactive mode (--yes), proceed without prompting. In interactive mode, prompt 'Copy config to <path>? [Y/n]'.
  - Files: `scripts/setup.sh`
  - Verify: Run with XDG_CONFIG_HOME=$(mktemp -d) -- file appears at expected path; run again -- 'skipping' message, exit 0

- [x] **T03: Implement download-model step** `est:25m`
  Add the install_model step: target path is $XDG_DATA_HOME/vibe-attack/models/whisper/ggml-tiny.en.bin (or ~/.local/share/vibe-attack/... if XDG_DATA_HOME unset). If file exists and size > 0, skip. Otherwise mkdir -p and download with curl -L showing progress. Print the URL and destination before downloading. If curl is not available, print a manual download instruction and exit 1.
  - Files: `scripts/setup.sh`
  - Verify: With XDG_DATA_HOME=$(mktemp -d) and network access: model downloads to correct path; re-run skips; with curl missing: prints manual instruction and exits 1

- [x] **T04: Implement uinput setup step** `est:30m`
  Add the setup_uinput step: check if /dev/uinput exists (if not, run sudo modprobe uinput); check if current user is in the input group (getent group input | grep -q "$USER"); if not, run sudo usermod -aG input "$USER" and print a logout reminder. After both checks, test that /dev/uinput is readable/writable by the current user. In interactive mode, prompt before each sudo command. In --dry-run mode, print what would be run without executing.
  - Files: `scripts/setup.sh`
  - Verify: On a system with uinput loaded and user in input group: step reports already satisfied; --dry-run prints sudo commands without executing them

- [x] **T05: Implement validation step and final summary** `est:20m`
  Add a validate step as the final step: re-check all four conditions (config file exists, model file exists and non-empty, /dev/uinput accessible, ptt.key set in config -- use grep). Print a summary table with pass/fail for each check. Exit 0 only if all four pass. Exit 1 with a clear message listing which checks failed. Add a SETUP COMPLETE banner on success.
  - Files: `scripts/setup.sh`
  - Verify: Run full script with --yes in a temp env where all steps succeed: validation prints all-pass table and SETUP COMPLETE; manually break one condition (remove config file) and re-run validate alone: prints specific failure and exits 1

## Files Likely Touched

- scripts/setup.sh
