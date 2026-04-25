# Phase 2 Target Hardware Latency Proof — Results (Wake Word)

## Run ID

- Run folder name: `results-wake/`
- Date/time (UTC): 2026-04-24

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
- Config file path used: Wake word detection enabled
- Model artifacts used (paths + model name, e.g. `tiny.en`): whisper/ggml-tiny.en.bin

## Raw artifacts

- `transcript.jsonl` (stdout capture): yes
- `timing.log` (stderr capture): yes

## Results (computed from utterance JSONL)

Metric: `e2e_ms` (end-of-speech → transcript JSONL emit)

- Sample count (N): 8
- p50 (ms): 769.0
- p95 (ms): 1271.5
- p99 (ms): 1321.5

PASS/FAIL (Phase 2 target): **FAIL** (p95 = 1271.5ms, target < 500ms)

## Notes

- Wake word series produced only 8 utterances (limited sample size for statistical significance)
- Even minimum observed latency (523ms) exceeds Phase 2 target
- Latency profile similar to PTT series, suggesting model inference is primary bottleneck
- Longer tail latency in wake series may indicate additional wake-word detection overhead
- Additional test runs recommended with larger sample sizes and profiling to identify optimization opportunities
