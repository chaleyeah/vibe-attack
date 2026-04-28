---
id: T01
parent: S04
milestone: M009
key_files:
  - Cargo.toml
key_decisions:
  - Used rfd 0.17.2 (latest 0.17 patch) as optional dep listed under gui feature only — keeps default/headless build free of file-dialog backend
duration: 
verification_result: passed
completed_at: 2026-04-28T03:01:46.536Z
blocker_discovered: false
---

# T01: Added rfd 0.17 as optional gui-feature dependency; both default and gui builds compile cleanly with zero warnings

**Added rfd 0.17 as optional gui-feature dependency; both default and gui builds compile cleanly with zero warnings**

## What Happened

Added `rfd = { version = "0.17", optional = true }` to the `[dependencies]` section of `Cargo.toml` and appended `"dep:rfd"` to the `gui` feature array. Cargo resolved rfd 0.17.2 (latest patch) along with its transitive dep pollster 0.4.0. The default build (`cargo build`) completed in 1.71s with no warnings — rfd is not compiled because it is optional. The gui build (`cargo build --features gui`) compiled rfd 0.17.2 and pollster 0.4.0 and finished in 8.50s with no warnings. `cargo metadata` confirmed rfd is present in the dependency graph.

## Verification

Ran `cargo build` (default, no features) → exit 0, zero warnings. Ran `cargo build --features gui` → exit 0, zero warnings. Ran `cargo metadata --format-version 1 | grep -q '"name":"rfd"'` → found rfd in metadata.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build` | 0 | ✅ pass | 1710ms |
| 2 | `cargo build --features gui` | 0 | ✅ pass | 8500ms |
| 3 | `cargo metadata --format-version 1 | grep -q '"name":"rfd"'` | 0 | ✅ pass | 200ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `Cargo.toml`
