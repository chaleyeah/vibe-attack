# T02: Emit events in Dispatcher::process()

**Slice:** S03 — **Milestone:** M001

## Description

Thread a `JsonlWriter` (or a lightweight event sender) into `Dispatcher` so `process()` can emit:

1. `dispatch` event immediately after `macro_tx.send()` succeeds
2. `no_match` event when `find_best_match` returns `None`

The coordinator already owns the only stdout `JsonlWriter`. To avoid a second writer on stdout, pass an `mpsc::Sender<JsonlEvent<'static>>` (or an owned struct) into `Dispatcher::new()`. The output thread drains this alongside the utterance channel.

Alternatively — and simpler — `Dispatcher` owns a `Mutex<Box<dyn Write + Send>>` injected at construction so tests can inject a `Vec<u8>` writer and production code injects `stdout`. Choose whichever avoids the lifetime complexity.

Emit `utterance_id: 0` for now (dispatcher doesn't receive the utterance_id from the STT result yet — that wiring is a future task). Add `utterance_id` to `SttResult` if straightforward; otherwise `0` is acceptable for S03.

## Files

- `src/pipeline/dispatcher.rs`
- `src/pipeline/coordinator.rs` (thread in the writer or sender)

## Verify

- `cargo check` passes
- `cargo test` passes
- Manual: run with `RUST_LOG=info` and observe `dispatch`/`no_match` lines in stdout JSONL during a test run
