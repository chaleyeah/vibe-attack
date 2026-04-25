# Phase 2 Target Hardware Latency Proof — Results (PTT)

## Run ID

- Run folder name: `results-ptt/`
- Date/time (UTC): 2026-04-23

## Machine metadata

- Hostname / machine label: CHADMIN-BS
- CPU model: AMD Ryzen 9 9950X3D 16-Core Processor
- RAM: 64GB
- OS + kernel: Linux 7.0.0-14-generic #14-Ubuntu SMP PREEMPT_DYNAMIC Mon Apr 13 11:09:53 UTC 2026
- Power governor / performance mode (if known): Not specified
- Microphone device / audio backend notes (if relevant): PipeWire Sound Server, 44100 Hz native rate (resampled to 16 kHz)

## Build + config

- Repo git commit (short hash): b758c46
- Build profile: `--release`
- Binary path used: Not specified
- CLI args used: Not specified
- Config file path used: ptt_key=KEY_LEFTCTRL dwell_ms=50 gap_ms=30
- Model artifacts used (paths + model name, e.g. `tiny.en`): whisper/ggml-tiny.en.bin

## Raw artifacts

- `transcript.jsonl` (stdout capture): yes
- `timing.log` (stderr capture): yes

## Results (computed from utterance JSONL)

Metric: `e2e_ms` (end-of-speech → transcript JSONL emit)

- Sample count (N): 49
- p50 (ms): 2169.0
- p95 (ms): 4697.5
- p99 (ms): 8443.5

PASS/FAIL (Phase 2 target): **FAIL** (p95 = 4697.5ms, target < 500ms)

## Notes

- PTT series produced 49 utterances with significant latency variance
- Tail latency (p95, p99) exceeds Phase 2 target by 9x
- Minimum observed latency (1109ms) already exceeds target threshold
- May indicate model inference time dominates end-to-end latency (tiny.en model on CPU)
