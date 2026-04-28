---
estimated_steps: 7
estimated_files: 1
skills_used: []
---

# T01: Add rfd 0.17 dependency under gui feature

Add the rfd (Rusty File Dialog) crate to Cargo.toml under the `gui` feature so the editor panel can open native file pickers. Confirm both default and gui builds compile cleanly. This is a dependency-only change — no code changes yet.

Why: rfd is the standard egui-ecosystem file picker. It calls the native portal on Wayland and zenity/kdialog on X11. Without it, the next two tasks cannot reference rfd::FileDialog. Adding it as optional + listed under the `gui` feature keeps the default (no-features) build untouched.

Key constraints:
- rfd MUST be added with `optional = true` and listed under the `gui` feature dep array. Do not add it to default deps — that would force the file-dialog backend into the headless daemon binary.
- Use rfd version `0.17` (research-confirmed current stable; 0.17.2 is fine).
- Do not import rfd anywhere yet — that happens in T04. This task only touches Cargo.toml.
- Distribution targets are Debian, Red Hat, and Arch (per project memory); rfd's xdg-portal backend covers all three. No extra system deps required at build time.

## Inputs

- ``Cargo.toml``

## Expected Output

- ``Cargo.toml``

## Verification

cargo build && cargo build --features gui — both must exit 0 with zero rustc warnings; cargo metadata --format-version 1 | grep -q '"name":"rfd"' to confirm the crate is in the dependency graph.
