---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Remove sha2 dependency from Cargo.toml

Confirm via grep that sha2 is unused in src/ and tests/, then remove it from Cargo.toml [dependencies]. Run cargo check to confirm no transitive resolution failure. Run cargo test to confirm no regression. Commit Cargo.toml and the resulting Cargo.lock change.

## Inputs

- `Current Cargo.toml with sha2 = "0.10" present`

## Expected Output

- `Cargo.toml with sha2 line removed from [dependencies]`
- `Cargo.lock updated with sha2 and any sha2-only transitive deps removed`

## Verification

grep -rn 'use sha2\|sha2::' src/ tests/ returns no matches; cargo check succeeds; cargo test passes
