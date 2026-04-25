# T03: Integration test — transcript to MacroCmd on channel

**Slice:** S03 — **Milestone:** M001

## Description

Add a test in `tests/dispatcher_logic.rs` that:

1. Constructs a `Dispatcher` with a known macro set (phrase + keys)
2. Calls `dispatcher.process("eagle airstrike")`
3. Asserts a `MacroCmd::Execute` arrives on the receiver with the expected key sequence

This proves the coordinator wiring end-to-end in-process: `process()` → `macro_tx.send()` → receiver sees the command.

Also add:
- A test that verifies `process()` with an unmatched transcript sends nothing to the macro channel (no spurious fires)
- A test that verifies flag/condition gating: macro only fires after its `if_flag` is set

The existing `test_dispatcher_conditional_reuse` and `test_dispatcher_negated_condition` in `dispatcher_logic.rs` already cover flag/condition logic — confirm they still pass; add the basic match→fire test which is currently missing.

## Files

- `tests/dispatcher_logic.rs`

## Verify

- `cargo test --test dispatcher_logic` passes
- All three test cases (match fires, no-match doesn't fire, flag gates fire) green
