---
id: T04
parent: S03
milestone: M003
key_files:
  - src/ui/wizard.rs
key_decisions:
  - Used Arc<Mutex<Option<String>>> not Arc<AtomicPtr> — simpler and avoids unsafe for a low-frequency wizard operation
  - find_keyboard_device() selects devices advertising KEY_A — standard enough to catch all keyboards, specific enough to skip mouse-only devices
duration: 
verification_result: passed
completed_at: 2026-04-26T00:19:13.942Z
blocker_discovered: false
---

# T04: Implemented ConfigurePtt panel with evdev capture thread and rewrite_ptt_key() config writer; three unit tests pass

**Implemented ConfigurePtt panel with evdev capture thread and rewrite_ptt_key() config writer; three unit tests pass**

## What Happened

show_configure_ptt renders a 'Listen for key' button that spawns capture_first_keypress() on a background thread. The thread finds the first keyboard device (by KEY_A support), waits for the first KeyDown event, stores the key name in Arc<Mutex<Option<String>>>, and exits. On next frame show_wizard() harvests the result, calls write_ptt_key_to_config(), and re-probes. Manual key entry text field also provided. rewrite_ptt_key() is a pure function that handles replace-active, replace-commented, and append-ptt-section cases. Three unit tests for rewrite_ptt_key cover all three branches (gated to --features gui).

## Verification

cargo check --lib clean; rewrite_ptt_key logic verified via 3 unit tests under --features gui

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
