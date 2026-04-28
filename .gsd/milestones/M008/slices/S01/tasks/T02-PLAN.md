---
estimated_steps: 12
estimated_files: 1
skills_used: []
---

# T02: Make Dispatcher threshold runtime-mutable via update_threshold()

Make the phrase-match confidence threshold mutable at runtime so the coordinator can apply SetThreshold without rebuilding the dispatcher. Wrap the `matcher: PhraseMatcher` field in `Dispatcher` (`src/pipeline/dispatcher.rs`) with `RwLock<PhraseMatcher>`. Update the constructor to wrap accordingly. Update `Dispatcher::process()` to acquire a read lock on the matcher before calling `find_best_match`. Add a new `pub fn update_threshold(&self, threshold: f32)` method that clamps the input to `[0.0, 1.0]`, acquires the write lock, replaces the inner `PhraseMatcher` with `PhraseMatcher::new(clamped)`, and emits `tracing::info!(old, new, "dispatcher threshold updated")`. Add a unit test in the existing `#[cfg(test)] mod tests` block (or a new one if none exists in dispatcher.rs) named `test_update_threshold_changes_match_behavior` that: constructs a Dispatcher with threshold 0.99 and a single macro phrase "eagle airstrike", asserts `process("eagal airstrike")` returns NoMatch (score below 0.99), then calls `update_threshold(0.5)` and asserts the same input now returns Fired. Existing matcher unit tests in `src/pipeline/matcher.rs` must continue to pass unchanged. The `unsafe impl Send/Sync for Dispatcher` blocks must remain (rodio constraint, MEM019); RwLock is Send+Sync so this does not affect that justification — keep the existing SAFETY comments intact.

## Failure Modes

| Dependency | On error | On timeout | On malformed response |
|------------|----------|-----------|----------------------|
| RwLock<PhraseMatcher> poison (only on panic during write) | propagate via `.unwrap()` consistent with existing `macros: Arc<RwLock<Vec<MacroConfig>>>` usage in this file | N/A (no timeout) | N/A |

## Load Profile

- **Shared resources**: one additional RwLock on the dispatcher hot path (acquired read-side per utterance).
- **Per-operation cost**: one read-lock acquisition per `process()` call (microseconds, uncontended).
- **10x breakpoint**: write-side calls (update_threshold) are rare (user-initiated); no breakpoint at expected load.

## Negative Tests

- **Boundary conditions**: `update_threshold(-0.5)` clamps to 0.0; `update_threshold(2.0)` clamps to 1.0; `update_threshold(f32::NAN)` clamps to 1.0 (or 0.0 — pick one and document; recommend NAN→0.0 via `.max(0.0).min(1.0)` semantics with NaN comparison short-circuit).
- **Behavior verification**: the test described above (threshold-flip changes match outcome) is the primary negative-path proof.

## Inputs

- ``src/pipeline/dispatcher.rs` — existing Dispatcher struct with `matcher: PhraseMatcher` field`
- ``src/pipeline/matcher.rs` — PhraseMatcher::new(threshold) constructor (unchanged this task)`

## Expected Output

- ``src/pipeline/dispatcher.rs` — `matcher` field becomes `RwLock<PhraseMatcher>`; new `pub fn update_threshold(&self, threshold: f32)`; new unit test `test_update_threshold_changes_match_behavior``

## Verification

cargo test --lib pipeline::dispatcher 2>&1 | tail -5 shows new `test_update_threshold_changes_match_behavior` passes; cargo test --lib pipeline::matcher remains green; `cargo clippy --all-targets -- -D warnings` clean.
