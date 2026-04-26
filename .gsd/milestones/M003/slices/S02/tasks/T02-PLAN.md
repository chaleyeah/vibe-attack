---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T02: Add libc dependency and write hermetic unit tests

Add libc to Cargo.toml dependencies (needed for O_NONBLOCK in probe). Write #[cfg(test)] tests in src/ui/probe.rs covering: check_config returns false when config missing, true when present; check_model returns false when model missing or empty, true when non-empty file present; check_ptt returns false when config has no PTT key, true when 'key: KEY_LEFTCTRL' is present. All tests use tempdir and set XDG_CONFIG_HOME / XDG_DATA_HOME for hermetic isolation. Use unsafe { std::env::set_var } with the established pattern from existing tests.

## Inputs

- `tests/config_parse.rs (XDG isolation pattern)`
- `src/ui/first_run.rs`

## Expected Output

- `libc in Cargo.toml [dependencies]`
- `At least 6 unit tests in probe.rs #[cfg(test)]`
- `cargo test --lib exits 0`

## Verification

cargo test --lib ui::probe exits 0 with all tests passing
