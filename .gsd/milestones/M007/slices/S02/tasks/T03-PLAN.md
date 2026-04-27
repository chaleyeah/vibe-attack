---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T03: Document the dual get_socket_path functions in control/

In src/control/mod.rs and src/control/client.rs, add a comment on each private get_socket_path function explaining the intentional difference: mod.rs uses place_runtime_file (creating the parent dir), client.rs uses find_runtime_file (read-only lookup). The comment should reference its counterpart so a reader searching for one finds the other.

## Inputs

- `src/control/mod.rs and src/control/client.rs each containing a private get_socket_path with no cross-reference`

## Expected Output

- `Both files have a comment on get_socket_path explaining the intentional split between place vs find runtime-file behavior`

## Verification

grep -B3 'fn get_socket_path' src/control/ shows an explanatory comment in each location; cargo check passes
