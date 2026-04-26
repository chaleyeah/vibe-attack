---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T04: Implement uinput setup step

Add the setup_uinput step: check if /dev/uinput exists (if not, run sudo modprobe uinput); check if current user is in the input group (getent group input | grep -q "$USER"); if not, run sudo usermod -aG input "$USER" and print a logout reminder. After both checks, test that /dev/uinput is readable/writable by the current user. In interactive mode, prompt before each sudo command. In --dry-run mode, print what would be run without executing.

## Inputs

- `docs/uinput-setup.md`

## Expected Output

- `Step skips modprobe when /dev/uinput already exists`
- `Step skips usermod when user already in input group`
- `--dry-run prints commands without executing`
- `Interactive prompt fires before sudo on interactive run`

## Verification

On a system with uinput loaded and user in input group: step reports already satisfied; --dry-run prints sudo commands without executing them
