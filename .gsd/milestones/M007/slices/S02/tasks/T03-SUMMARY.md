---
id: T03
parent: S02
milestone: M007
key_files:
  - src/control/mod.rs
  - src/control/client.rs
key_decisions:
  - Placed the comment as a plain code comment (// ...) directly above the fn, consistent with the SAFETY: and allow-justification comment style established in T01 and T02
  - Each comment names the specific XDG call used and references the counterpart file by path, so a reader doesn't need to search
duration: 
verification_result: passed
completed_at: 2026-04-27T11:43:03.461Z
blocker_discovered: false
---

# T03: Documented the dual get_socket_path functions in control/mod.rs and control/client.rs with cross-referencing comments explaining place vs find runtime-file semantics

**Documented the dual get_socket_path functions in control/mod.rs and control/client.rs with cross-referencing comments explaining place vs find runtime-file semantics**

## What Happened

Both `src/control/mod.rs` and `src/control/client.rs` contain a private `get_socket_path` function. They are intentionally different: the server-side function in `mod.rs` uses `xdg.place_runtime_file`, which creates the XDG runtime directory if it doesn't exist (needed when the daemon starts and must own the socket path), while the client-side function in `client.rs` uses `xdg.find_runtime_file`, which is a read-only lookup that returns an error if the socket isn't present (the correct behavior for a client probing whether the daemon is running).

Added a two-line comment immediately above each function that explains which XDG call it uses, what that implies (create vs. read-only), and explicitly points the reader to the counterpart file. This satisfies the cross-reference requirement so a reader following either function can immediately find the other.

## Verification

grep -B3 'fn get_socket_path' src/control/mod.rs src/control/client.rs — confirmed explanatory comment present above each function; cargo check — finished with no errors or warnings.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `grep -B3 'fn get_socket_path' src/control/mod.rs src/control/client.rs` | 0 | ✅ pass — comment with cross-reference visible above each function | 50ms |
| 2 | `cargo check` | 0 | ✅ pass — Finished dev profile, no errors | 340ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/control/mod.rs`
- `src/control/client.rs`
