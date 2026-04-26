---
id: T02
parent: S03
milestone: M003
key_files:
  - src/ui/wizard.rs
key_decisions:
  - (none)
duration: 
verification_result: passed
completed_at: 2026-04-26T00:19:01.758Z
blocker_discovered: false
---

# T02: Implemented show_create_config (button copies file + re-probes) and show_install_model (monospace curl command + Re-check button)

**Implemented show_create_config (button copies file + re-probes) and show_install_model (monospace curl command + Re-check button)**

## What Happened

show_create_config: heading, XDG target path label, 'Copy example config' button that mkdir_p + copies config.example.yaml then calls probe::run(). show_install_model: heading, model path label, scrollable horizontal monospace block with the full curl command, Re-check button. Both re-probe on action to update FirstRunState.

## Verification

cargo check --lib clean; panels are dispatched correctly by show_wizard()

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check --lib` | 0 | pass | 80ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/ui/wizard.rs`
