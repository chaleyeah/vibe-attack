---
estimated_steps: 5
estimated_files: 1
skills_used: []
---

# T01: Add Dispatcher::fire_named with unit tests

Add `pub fn fire_named(&self, name: &str) -> Result<DispatchOutcome, String>` to `Dispatcher` in `src/pipeline/dispatcher.rs`. The method skips phrase matching: it acquires a read lock on `self.macros`, finds the first MacroConfig whose `name` equals the given name, plays the optional sound (same code path as `process()`), builds a `Vec<KeyStep>` via `KeyStep::from_config`, and sends `MacroCmd::Execute { keys, default_dwell_ms, default_gap_ms }` over `self.macro_tx`. On found-and-sent return `Ok(DispatchOutcome::Fired { macro_id: name.into(), score: 1.0 })` (score 1.0 marks a direct trigger as a deliberate convention). On not-found return `Err(format!("macro not found: {name}"))`. On `macro_tx.send` failure (receiver dropped) return `Err(format!("injection channel closed: {e}"))`. Add tracing::info!(macro_name=name, "Firing macro (direct)") before the send so journalctl distinguishes direct triggers from phrase matches.

Add two unit tests to the existing `mod tests` block, mirroring the `make_dispatcher` helper pattern:
- `fire_named_found_emits_execute`: build dispatcher with a known macro carrying two KeyAction entries; capture the receiver end of the macro channel; call `fire_named("eagle_airstrike")`; assert Ok(Fired { macro_id, score }) where macro_id == "eagle_airstrike" and (score - 1.0).abs() < 1e-6; assert exactly one `MacroCmd::Execute` was sent and its keys vec length matches the configured KeyAction count.
- `fire_named_missing_returns_err`: build dispatcher with one macro; call `fire_named("does_not_exist")`; assert Err whose Display contains "macro not found"; assert receiver got zero MacroCmd messages (use try_recv() and expect Err(TryRecvError::Empty)).

Do NOT touch `process()` or any other dispatcher behavior. Do NOT add a `category` argument — name lookup over the flat registry is sufficient.

## Inputs

- ``src/pipeline/dispatcher.rs` — existing Dispatcher struct, macro_tx Sender<MacroCmd>, macros Arc<RwLock<Vec<MacroConfig>>>, default_dwell_ms/default_gap_ms fields, and the `make_dispatcher` test helper to reuse`
- ``src/input/inject.rs` — MacroCmd::Execute variant (keys: Vec<KeyStep>, default_dwell_ms, default_gap_ms) and KeyStep::from_config(&KeyAction) constructor`
- ``src/config.rs` — MacroConfig struct (fields: name, phrase, if_flag, set_flag, sound, keys: Vec<KeyAction>) used by fire_named's lookup`

## Expected Output

- ``src/pipeline/dispatcher.rs` — new `pub fn fire_named` method on Dispatcher plus two new unit tests `fire_named_found_emits_execute` and `fire_named_missing_returns_err` in the existing `mod tests` block`

## Verification

cargo test --lib pipeline::dispatcher -- --test-threads=1 && RUSTFLAGS="-D warnings" cargo check --all-targets
