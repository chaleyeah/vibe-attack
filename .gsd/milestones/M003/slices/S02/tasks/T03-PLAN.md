---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T03: Wire probe::run() into vibe-attack-config.rs

Replace FirstRunState::from_checks(false, false, false, false) in src/bin/vibe-attack-config.rs with a call to vibe_attack::ui::probe::run(). Call probe::run() in VibeAttackConfigApp::new(). No other changes to the binary — wizard UI panels come in S03.

## Inputs

- `src/bin/vibe-attack-config.rs (current stub)`
- `src/ui/probe.rs (T01 output)`

## Expected Output

- `vibe-attack-config.rs uses probe::run() in new()`
- `cargo build --bin vibe-attack-config --features gui exits 0`

## Verification

cargo build --bin vibe-attack-config --features gui exits 0; no from_checks(false,false,false,false) remains in production code
