---
status: partial
phase: 01-foundation
source: [01-VERIFICATION.md]
started: 2026-04-21
updated: 2026-04-21
---

## Current Test

[awaiting human testing]

## Tests

### 1. PTT gates audio on real hardware
expected: Run daemon with `-vv`, hold PTT key while a fullscreen game is foregrounded; TRACE logs appear in daemon terminal showing audio gate open/close.
result: [pending]

### 2. Key sequences inject into focused Wayland window
expected: Run with /dev/uinput access and a text editor in focus; arrow keys (or any configured key sequence) register in the target window. Alternatively: `RUN_PRIVILEGED_TESTS=1 cargo test --test macro_inject -- --include-ignored` exits 0.
result: [pending]

### 3. No focus disruption with live fullscreen game
expected: Start a fullscreen game (e.g. Helldivers 2 via Steam/Proton), then start daemon; game does not minimize or lose focus.
result: [pending]

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps
