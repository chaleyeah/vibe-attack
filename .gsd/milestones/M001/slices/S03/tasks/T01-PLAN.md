# T01: Add no_match and dispatch JSONL event variants

**Slice:** S03 — **Milestone:** M001

## Description

Add two new variants to `JsonlEvent` in `src/pipeline/jsonl.rs`:

- `NoMatch { utterance_id, transcript, wall_time_ms, mono_ms }` — emitted when `Dispatcher::process()` finds no phrase above threshold
- `Dispatch { utterance_id, macro_id, score, wall_time_ms, mono_ms }` — emitted as a second JSONL line after `utterance` when a macro fires

Both variants must serialize with `type: "no_match"` and `type: "dispatch"` respectively (via `#[serde(tag = "type")]`).

## Files

- `src/pipeline/jsonl.rs`

## Verify

- `cargo check` passes
- `cargo test -p hd-linux-voice jsonl` passes
- New variants appear in `JsonlEvent` with correct serde output verified by a unit test
