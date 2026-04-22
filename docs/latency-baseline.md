# Phase 2 Latency Baseline (End-of-speech → transcript emit)

This document defines a **repeatable “Phase 2 latency proof”** procedure to measure the Phase 2 budget on **target hardware**.

## What we measure in Phase 2 (proxy for STT-04)

`STT-04` in `.planning/REQUIREMENTS.md` describes latency as:

- **end of speech → first key event**

In **Phase 2** there is **no dispatch** yet, so the measurable proxy is:

- **end of speech → transcript JSONL emit**

We keep the same **< 500ms** budget proxy here. The “→ first key event” portion is validated in **Phase 3 (dispatch)** while keeping this Phase 2 measurement identical.

## Baseline assumptions

- **Reference model:** Phase 2 proof uses **`tiny.en`** as the baseline model.
  - The **model path remains config-driven** (see `stt.model_path` in your config).
- **Output contract:** transcripts are emitted as **JSONL on stdout** (one JSON object per line), while timing/instrumentation logs go to **stderr**.

## One-time setup (target machine)

Build a release binary:

```bash
cargo build --release
```

Pick (or create) a config file with:

- working audio capture + PTT (or wake word)
- `stt.model_path` pointing at a local **`tiny.en`** whisper.cpp model file

## Run procedure (collect artifacts)

From the repo root:

```bash
./target/release/hd-linux-voice --config /path/to/config.yaml -v \
  > transcript.jsonl \
  2> timing.log
```

Logging verbosity guidance:

- `-v` (DEBUG): recommended for baseline runs
- `-vv` (TRACE): use only if you need deeper per-stage detail (more stderr noise)

### Test script (N short phrases)

Speak **N short phrases** while PTT is held (or after wake word). Suggested:

- \(N = 50\) short phrases, 1–3 seconds each
- consistent distance to mic
- minimal background noise

Stop the daemon (Ctrl+C) after collecting samples.

Artifacts produced:

- `transcript.jsonl` — machine-readable transcript stream (stdout)
- `timing.log` — instrumentation / timing logs (stderr)

## Pass/Fail checklist (Phase 2 success criteria alignment)

This checklist is aligned to Phase 2 success criteria in `.planning/ROADMAP.md`:

- **JSONL cleanliness**: `transcript.jsonl` contains **one JSON object per line** (no log contamination).
- **Per-utterance summary present**: each utterance emits a summary JSON that includes:
  - **per-stage monotonic durations** (non-negative)
  - an **explicit end-to-end duration** field **`e2e_ms`** for the Phase 2 metric (**end-of-speech → transcript JSONL emit**)
- **Latency acceptance (target hardware)**:
  - Compute **p95** of the end-to-end duration across the N phrases
  - **PASS if p95 < 500ms**, **FAIL otherwise**

## How to interpret results (mapping to roadmap budgets)

The per-stage duration fields should map to Phase 2 budget targets from the roadmap:

- **VAD compute cost**: `vad_ms` target **≤ 50ms**
- **STT compute cost**: `stt_ms` target **≤ 200ms** (reference `tiny.en`, CPU-only)
- **Total (end-of-speech → transcript emit)**: target **p95 < 500ms**

Field definitions (from the stable stdout JSONL utterance schema):

- `e2e_ms = output_done_ms - vad_done_ms` (monotonic ms)
  - `vad_done_ms`: monotonic marker at end-of-speech detection / utterance cut
  - `output_done_ms`: monotonic marker immediately before emitting the utterance JSONL line

If total latency fails but individual stage targets look fine, investigate:

- output thread stalls / stdout buffering
- queue contention (bounded channel behavior)
- CPU scheduling / background load

## Known caveats

- **CPU load matters**: background compilation, browsers, game overlays, or power-saving governors can inflate p95.
- **Microphone devices differ**: sample rate conversion, driver buffering, and USB audio latency can shift the end-of-speech boundary.
- **Background noise affects VAD**: noisy environments can delay end-of-speech detection (more “silence” required to cut).

