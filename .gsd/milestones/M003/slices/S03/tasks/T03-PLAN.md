---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T03: Implement SetupUinput panel

In wizard.rs, implement show_setup_uinput(ui). Heading 'Set up uinput access'. Shows two code blocks: 'sudo modprobe uinput' and 'sudo usermod -aG input $USER'. Shows the systemd v258+/CachyOS note. Shows a 'Re-check' button that calls probe::run() to refresh state. No sudo commands are run by the app itself — the panel is informational with re-probe.

## Inputs

- `docs/uinput-setup.md`

## Expected Output

- `show_setup_uinput renders two code blocks and note`
- `Re-check button refreshes state`

## Verification

Panel renders without panic; Re-check button calls probe::run()
