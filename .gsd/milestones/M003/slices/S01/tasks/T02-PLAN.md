---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T02: Implement copy-config step

Add the copy_config step: determine XDG config path ($XDG_CONFIG_HOME or ~/.config), target is $xdg_config/vibe-attack/config.yaml. If the file already exists, print 'already exists, skipping' and return 0. Otherwise mkdir -p the directory and cp config.example.yaml to the target. In non-interactive mode (--yes), proceed without prompting. In interactive mode, prompt 'Copy config to <path>? [Y/n]'.

## Inputs

- `config.example.yaml`

## Expected Output

- `Config file copied to $XDG_CONFIG_HOME/vibe-attack/config.yaml on first run`
- `Second run prints skip message and exits 0`
- `Interactive mode shows prompt when --yes not passed`

## Verification

Run with XDG_CONFIG_HOME=$(mktemp -d) -- file appears at expected path; run again -- 'skipping' message, exit 0
