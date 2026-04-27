---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T04: Verify docs/troubleshooting.md and docs/uinput-setup.md accuracy

Read both files line-by-line. Confirm: (a) binary names match (vibe-attack, vibe-attack-config), (b) uinput group name and udev rules match current install instructions and PKGBUILD/.spec/debian/, (c) command examples (e.g. journalctl filters, systemctl status) reference correct service names, (d) error messages quoted in troubleshooting.md still match current src/error.rs Display impls. Update any drift.

## Inputs

- `docs/troubleshooting.md, docs/uinput-setup.md, current src/error.rs (post-S04), Cargo.toml [[bin]] entries, packaging files`

## Expected Output

- `Both docs updated to match current state`

## Verification

Manual cross-reference confirms binary names, group names, error messages, and commands all match current state
