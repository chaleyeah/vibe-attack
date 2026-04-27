---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Verify README.md accuracy

Read README.md line-by-line. Verify: (a) project name is vibe-attack throughout, (b) the architecture description matches src/lib.rs //! crate doc from S03, (c) build/run instructions work against current Cargo.toml (cargo build, cargo build --features gui, cargo run --bin vibe-attack), (d) configuration section references the actual fields in src/config.rs, (e) feature flags described match what's in Cargo.toml [features]. Update any drift.

## Inputs

- `README.md and current src/lib.rs (post-S03), src/config.rs, Cargo.toml`

## Expected Output

- `README.md updated to match current state`

## Verification

Manual review confirms README matches current code; running the documented build/run commands actually works
