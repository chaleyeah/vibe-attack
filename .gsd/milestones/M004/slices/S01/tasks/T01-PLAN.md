---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T01: Control protocol + DaemonHandle

Extend protocol.rs with STATUS/MUTE/UNMUTE commands and DaemonState enum; implement DaemonHandle in client.rs

## Inputs

- `existing control socket protocol`

## Expected Output

- `DaemonHandle with query_status/mute/unmute`
- `STATUS/MUTE/UNMUTE in protocol`

## Verification

cargo test control:: passes
