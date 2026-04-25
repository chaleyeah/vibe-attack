# T02: 02-pipeline-core 02

**Slice:** S02 — **Milestone:** M001

## Description

Create the core pipeline contracts and VAD segmentation implementation: stable JSONL event schema (stdout), timing capture utilities (monotonic + wall clock), and Silero-based utterance detection/assembly that yields bounded, measurable jobs for STT.

Purpose: Make the pipeline observable and bounded so end-to-end latency work is measurable and resilient under load.
Output: `pipeline` module (jsonl + timing + coordinator skeleton) and `vad` module that produces utterance jobs with stage timing hooks.

## Must-Haves

- [ ] "When speech is detected and cut into an utterance, the daemon can emit stable JSONL events with timing fields."
- [ ] "VAD segmentation respects pre-roll, tail, minimum speech, end-of-speech silence, and max utterance length decisions."

## Files

- `src/lib.rs`
- `src/pipeline/mod.rs`
- `src/pipeline/jsonl.rs`
- `src/pipeline/timing.rs`
- `src/vad/mod.rs`
- `tests/jsonl_schema.rs`
- `tests/drop_oldest_queue.rs`
