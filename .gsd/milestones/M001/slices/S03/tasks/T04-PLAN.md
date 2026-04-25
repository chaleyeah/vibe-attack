# T04: JSONL schema tests for no_match and dispatch events

**Slice:** S03 — **Milestone:** M001

## Description

Add tests in `tests/jsonl_schema.rs` that assert the stable serialization contract for the two new event types:

```
{"type":"no_match","utterance_id":0,"transcript":"orbital strike","wall_time_ms":...,"mono_ms":...}
{"type":"dispatch","utterance_id":0,"macro_id":"eagle_airstrike","score":1.0,"wall_time_ms":...,"mono_ms":...}
```

Tests should:
1. Construct each event variant directly
2. Serialize to JSON string
3. Parse the JSON and assert `type` field equals the expected string
4. Assert all expected fields are present and have correct types

This guards against accidental rename/restructuring breaking external tooling that parses the JSONL stream.

## Files

- `tests/jsonl_schema.rs`

## Verify

- `cargo test --test jsonl_schema` passes
- Both new event types have stable `type` field assertions
