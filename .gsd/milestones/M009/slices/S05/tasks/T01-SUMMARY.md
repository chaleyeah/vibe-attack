---
id: T01
parent: S05
milestone: M009
key_files:
  - src/pipeline/dispatcher.rs
key_decisions:
  - score: 1.0 used as a deliberate convention for direct (non-phrase) macro triggers to distinguish them from fuzzy-matched phrases in downstream JSONL events
  - make_dispatcher_with_keys test helper returns the Receiver<MacroCmd> so tests can assert on channel messages, unlike the existing make_dispatcher which discards _rx
duration: 
verification_result: passed
completed_at: 2026-04-28T03:14:41.758Z
blocker_discovered: false
---

# T01: Added Dispatcher::fire_named to bypass phrase matching and trigger macros by name directly, with two unit tests covering the found and missing cases

**Added Dispatcher::fire_named to bypass phrase matching and trigger macros by name directly, with two unit tests covering the found and missing cases**

## What Happened

Added `pub fn fire_named(&self, name: &str) -> Result<DispatchOutcome, String>` to `Dispatcher` in `src/pipeline/dispatcher.rs`. The method acquires a read lock on `self.macros`, finds the first entry whose `name` equals the argument, plays the optional sound (same path as `process()`), builds a `Vec<KeyStep>` via `KeyStep::from_config`, and sends `MacroCmd::Execute` over `self.macro_tx`. On success it returns `Ok(DispatchOutcome::Fired { macro_id: name.into(), score: 1.0 })` — score 1.0 is a deliberate convention marking direct (non-phrase) triggers. On name not found it returns `Err(format!("macro not found: {name}"))`. On send failure it returns `Err(format!("injection channel closed: {e}"))`. A `tracing::info!(macro_name = name, "Firing macro (direct)")` log before the send distinguishes direct triggers from phrase-matched ones in journalctl.\n\nAdded a `make_dispatcher_with_keys` helper in the test module that returns both the dispatcher and the receiver end of the macro channel (unlike the existing `make_dispatcher` which discards `_rx`). Added two unit tests:\n- `fire_named_found_emits_execute`: configures a macro with two `KeyAction` entries, calls `fire_named(\"eagle_airstrike\")`, asserts `Ok(Fired { macro_id == \"eagle_airstrike\", score ≈ 1.0 })`, asserts exactly one `MacroCmd::Execute` was sent with `keys.len() == 2`, asserts no further messages on the receiver.\n- `fire_named_missing_returns_err`: calls `fire_named(\"does_not_exist\")` on a dispatcher that only has `\"eagle_airstrike\"`, asserts `Err` containing `\"macro not found\"`, asserts zero `MacroCmd` messages were sent via `try_recv()` → `TryRecvError::Empty`.\n\nNo changes were made to `process()` or any other existing behavior. No category argument was added per plan.

## Verification

Ran `cargo test --lib pipeline::dispatcher -- --test-threads=1`: all 6 tests pass (4 pre-existing threshold tests + 2 new fire_named tests). Ran `RUSTFLAGS=\"-D warnings\" cargo check --all-targets`: finished with no warnings or errors.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --lib pipeline::dispatcher -- --test-threads=1` | 0 | ✅ pass — 6/6 tests passed | 1520ms |
| 2 | `RUSTFLAGS="-D warnings" cargo check --all-targets` | 0 | ✅ pass — no warnings | 820ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/pipeline/dispatcher.rs`
