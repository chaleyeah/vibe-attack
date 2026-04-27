---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T03: Document src/control/ public items

Add /// doc comments to every undocumented pub item in src/control/: mod.rs (DaemonHandle::new/state/status), client.rs (send_command/query_status/is_daemon_running). protocol.rs is already well-documented per research — verify and skip if so.

## Inputs

- `src/control/ with ~8 undocumented public items per research`

## Expected Output

- `All pub items in src/control/ have /// docs`

## Verification

Audit script reports 0 undocumented pub items under src/control/; cargo doc renders cleanly
