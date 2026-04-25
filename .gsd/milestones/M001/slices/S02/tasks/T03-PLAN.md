# T03: 02-pipeline-core 03

**Slice:** S02 — **Milestone:** M001

## Description

Wire the full Phase 2 pipeline end-to-end: drain audio from the existing RT-safe ringbuffer, run wake-word + VAD on an OS thread, push bounded utterance jobs (drop-oldest) to a dedicated blocking STT OS thread (whisper-rs), and emit final transcript JSONL events to stdout with stage timing instrumentation to stderr.

Purpose: Prove the pipeline meets on-device + <500ms budget constraints and establishes the output contract consumed by later phases.
Output: `stt` and `wake` modules, plus `main.rs` wiring that honors all thread/IO constraints.

## Must-Haves

- [ ] "Speaking a short phrase while PTT is held produces a final transcript JSONL event on stdout."
- [ ] "Wake word triggers LISTENING for ~5s and emits a trigger/status event on stderr while keeping stdout clean."
- [ ] "Stage timing instrumentation exists (AudioCapture → VAD → STT → output) and can be used to confirm the <500ms budget on target hardware."
- [ ] "STT runs on a dedicated blocking OS thread; audio RT callback remains allocation-free and never blocks."

## Files

- `src/main.rs`
- `src/pipeline/mod.rs`
- `src/stt/mod.rs`
- `src/wake/mod.rs`
- `src/lib.rs`
- `tests/stt_smoke.rs`
- `tests/wake_word.rs`
