---
estimated_steps: 9
estimated_files: 2
skills_used: []
---

# T02: Implement --skip-wizard CLI flag in vibe-attack-config main()

The M010 roadmap specifies a `--skip-wizard` CLI flag so users (and UAT testers) can bypass the wizard regardless of probe state. The flag is currently NOT implemented — `src/bin/vibe-attack-config.rs` does no CLI argument parsing at all.

Minimal implementation per S02-RESEARCH.md recommendation:
- In `main()`, before constructing `VibeAttackConfigApp`, check `std::env::args().any(|a| a == "--skip-wizard")`.
- Pass a bool through `VibeAttackConfigApp::new(log_rx, skip_wizard)` (or via a constructor variant).
- When skip_wizard is true, substitute `FirstRunState::from_checks(true, true, true, true)` instead of calling `probe::run()`.
- Log the skip via `tracing::info!(skip_wizard = true, "Wizard bypass via --skip-wizard flag")` so transcripts show the flag was honored.
- Also handle `--help` / `-h` to print a one-line usage and exit 0 (so `vibe-attack-config --help` works without launching a window).

Add two unit tests in `tests/ui_distribution.rs` (or `tests/cli.rs`) that EXEC the built binary with `--help` and assert exit 0 + usage text presence. Use `env!("CARGO_BIN_EXE_vibe-attack-config")` to locate the binary. Skip the test gracefully if the binary is not built (return early with a log line) rather than panicking — this keeps the suite green on `cargo test --no-default-features` runs.

Do NOT add a full clap dependency for this — keep it std::env only, per project minimalism convention.

## Inputs

- ``src/bin/vibe-attack-config.rs` — current main() has no CLI parsing; VibeAttackConfigApp::new takes only log_rx`
- ``src/ui/first_run.rs` — FirstRunState::from_checks(config, model, uinput, ptt) is the substitution target`
- ``src/ui/probe.rs` — probe::run() is what we are bypassing when skip_wizard is true`

## Expected Output

- ``src/bin/vibe-attack-config.rs` — main() parses --skip-wizard and --help flags; passes skip_wizard bool to constructor; tracing::info! logs the bypass`
- ``tests/ui_distribution.rs` — new tests asserting --help exit code 0 and usage line includes 'skip-wizard'`

## Verification

cargo test --test ui_distribution -- --test-threads=1 && cargo build --bin vibe-attack-config --features gui && target/debug/vibe-attack-config --help | grep -q 'skip-wizard'
