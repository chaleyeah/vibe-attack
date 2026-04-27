# S01: Control Socket — Daemon Status Query

**Goal:** Add STATUS/MUTE/UNMUTE to control protocol and wrap socket client in DaemonHandle
**Demo:** cargo test control:: passes; a CLI one-liner (echo STATUS | nc -U /run/...) returns a JSON status line from a running daemon

## Must-Haves

- Not provided.

## Proof Level

- This slice proves: Not provided.

## Integration Closure

Not provided.

## Verification

- Not provided.

## Tasks

- [x] **T01: Control protocol + DaemonHandle** `est:1h`
  Extend protocol.rs with STATUS/MUTE/UNMUTE commands and DaemonState enum; implement DaemonHandle in client.rs
  - Files: `src/control/protocol.rs`, `src/control/client.rs`, `src/control/mod.rs`
  - Verify: cargo test control:: passes

## Files Likely Touched

- src/control/protocol.rs
- src/control/client.rs
- src/control/mod.rs
