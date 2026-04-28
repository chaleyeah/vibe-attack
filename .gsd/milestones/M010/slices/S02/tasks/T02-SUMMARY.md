---
id: T02
parent: S02
milestone: M010
key_files:
  - src/bin/vibe-attack-config.rs
  - tests/ui_distribution.rs
key_decisions:
  - LD_LIBRARY_PATH set to binary's parent dir in spawned child processes so libsherpa-onnx-c-api.so resolves without system install
  - std::env::args() collected before tracing init so --help exits without setting up the log channel or GUI
duration: 
verification_result: passed
completed_at: 2026-04-28T04:00:43.595Z
blocker_discovered: false
---

# T02: Implement --skip-wizard and --help CLI flags in vibe-attack-config main(), wired through VibeAttackConfigApp::new(skip_wizard: bool), with two new regression tests asserting exit 0 and usage text

**Implement --skip-wizard and --help CLI flags in vibe-attack-config main(), wired through VibeAttackConfigApp::new(skip_wizard: bool), with two new regression tests asserting exit 0 and usage text**

## What Happened

VibeAttackConfigApp::new() previously took only log_rx and always called probe::run(). Added a skip_wizard: bool parameter. When true, the constructor substitutes FirstRunState::from_checks(true, true, true, true) in place of probe::run() and emits tracing::info!(skip_wizard = true, \"Wizard bypass via --skip-wizard flag\") so UAT transcripts can confirm the flag was honoured.

In main(), std::env::args() is collected before the log channel is set up. If --help or -h is present, a one-line usage message is printed to stdout (mentioning skip-wizard and --help) and the process exits 0 without opening a window. If --skip-wizard is present, the flag is captured as a bool and threaded into VibeAttackConfigApp::new() via a move closure passed to eframe::run_native.

Two new tests were added to tests/ui_distribution.rs:
- skip_wizard_help_exits_zero: spawns the binary with --help, asserts exit code 0, asserts stdout contains \"skip-wizard\".
- skip_wizard_flag_mentioned_in_help_output: asserts stdout contains both \"Usage:\" and \"--skip-wizard\".

Both tests resolve the binary path via env!(\"CARGO_BIN_EXE_vibe-attack-config\") and set LD_LIBRARY_PATH to the binary's parent directory (target/debug/) so the spawned process can find libsherpa-onnx-c-api.so. Both return early with an eprintln! if the binary is absent, keeping the suite green on cargo test --no-default-features runs.

All 18 tests in ui_distribution pass. The full verification command from the task plan (cargo test --test ui_distribution -- --test-threads=1 && cargo build --bin vibe-attack-config --features gui && LD_LIBRARY_PATH=target/debug target/debug/vibe-attack-config --help | grep -q 'skip-wizard') exits 0.

## Verification

Ran: cargo test --test ui_distribution -- --test-threads=1 (18/18 pass). Built binary with --features gui. Invoked target/debug/vibe-attack-config --help with LD_LIBRARY_PATH=target/debug — exits 0, stdout contains 'skip-wizard'. Full slice verification command passes.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test ui_distribution -- --test-threads=1` | 0 | ✅ pass | 860ms |
| 2 | `cargo build --bin vibe-attack-config --features gui` | 0 | ✅ pass | 110ms |
| 3 | `LD_LIBRARY_PATH=target/debug target/debug/vibe-attack-config --help | grep -q 'skip-wizard'` | 0 | ✅ pass | 30ms |

## Deviations

Tests set LD_LIBRARY_PATH explicitly in Command::env() rather than relying on ambient environment, which was not mentioned in the plan but is required because the binary links libsherpa-onnx-c-api.so dynamically and that library lives in target/debug/.

## Known Issues

none

## Files Created/Modified

- `src/bin/vibe-attack-config.rs`
- `tests/ui_distribution.rs`
