---
id: T02
parent: S01
milestone: M008
key_files:
  - src/pipeline/dispatcher.rs
  - src/pipeline/matcher.rs
key_decisions:
  - RwLock<PhraseMatcher> chosen over Arc<AtomicF32> because the threshold is consumed by PhraseMatcher::new() and the matcher is not separately shareable — replacing the whole inner matcher under write lock is the cleanest approach
  - NaN→0.0 via explicit is_nan() guard before clamp(), consistent with plan recommendation; negative→0.0 and >1.0→1.0 via f32::clamp
  - PhraseMatcher::threshold() getter added to matcher.rs (minimal addition) to support tracing old value in update_threshold without duplicating state
duration: 
verification_result: passed
completed_at: 2026-04-28T01:16:40.162Z
blocker_discovered: false
---

# T02: feat: wrap Dispatcher.matcher in RwLock<PhraseMatcher> and add update_threshold() with clamping, NaN guard, tracing::info!, and 4 unit tests

**feat: wrap Dispatcher.matcher in RwLock<PhraseMatcher> and add update_threshold() with clamping, NaN guard, tracing::info!, and 4 unit tests**

## What Happened

Wrapped the `matcher: PhraseMatcher` field in `Dispatcher` (`src/pipeline/dispatcher.rs`) with `RwLock<PhraseMatcher>`. The constructor now does `RwLock::new(PhraseMatcher::new(threshold))`. `process()` now calls `self.matcher.read().unwrap().find_best_match(...)` to acquire a read lock before matching. Added `pub fn update_threshold(&self, threshold: f32)` that guards against NaN (`threshold.is_nan()` → clamp to 0.0), clamps valid values to `[0.0, 1.0]` using `f32::clamp`, reads the old threshold via a new `PhraseMatcher::threshold()` getter added to `matcher.rs`, replaces the inner `PhraseMatcher` via `*self.matcher.write().unwrap() = PhraseMatcher::new(clamped)`, and emits `tracing::info!(old, new = clamped, "dispatcher threshold updated")`. Added `pub fn threshold(&self) -> f32` accessor to `PhraseMatcher` in `matcher.rs` so the old value can be read for the log. The existing `unsafe impl Send/Sync` blocks with their `// SAFETY:` comments were preserved intact (MEM010, MEM019 invariant). Added a `#[cfg(test)] mod tests` block in `dispatcher.rs` with 4 tests: `test_update_threshold_changes_match_behavior` (the primary happy-path/flip test: 0.99→NoMatch, then 0.5→Fired with "eagal airstrike"), `test_update_threshold_clamp_negative` (-0.5 clamps to 0.0, match fires), `test_update_threshold_clamp_above_one` (2.0 clamps to 1.0, fuzzy rejected but exact fires), and `test_update_threshold_nan_becomes_zero` (NaN→0.0, match fires). All 4 dispatcher tests pass; all 5 existing matcher tests remain green; all 17 control_protocol integration tests remain green; `cargo check --all-targets` is clean.

## Verification

cargo test --lib pipeline::dispatcher: 4 new tests pass. cargo test --lib pipeline::matcher: 5 existing tests pass unchanged. cargo test --test control_protocol: 17 pass. cargo check --all-targets: clean (no errors or warnings). Clippy binary absent from this toolchain per T01 precedent; cargo check used as substitute.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --lib pipeline::dispatcher 2>&1 | tail -8` | 0 | ✅ pass | 2050ms |
| 2 | `cargo test --lib pipeline::matcher 2>&1 | tail -8` | 0 | ✅ pass | 80ms |
| 3 | `cargo test --test control_protocol 2>&1 | tail -3` | 0 | ✅ pass | 110ms |
| 4 | `cargo check --all-targets 2>&1 | tail -3` | 0 | ✅ pass | 480ms |

## Deviations

Added `pub fn threshold(&self) -> f32` accessor to `PhraseMatcher` in `matcher.rs` — not mentioned in the plan but required to log the old threshold value as specified by the tracing::info! requirement. The getter is a pure read with no behavioral side effect.

## Known Issues

None.

## Files Created/Modified

- `src/pipeline/dispatcher.rs`
- `src/pipeline/matcher.rs`
