# Phase 2 Target Hardware Latency Proof — Results Template

Copy this file to a run directory as `RESULTS.md` and fill it in.

This document exists to make Phase 2 target-hardware proof runs attributable and reviewable (mitigates repudiation by capturing machine + build identity alongside the computed results).

## Run ID

- Run folder name: `run-20260425-HHMM-{hostname-or-machine}/`
- Date/time (UTC):

## Machine metadata

- Hostname / machine label:
- CPU model:
- RAM:
- OS + kernel:
- Power governor / performance mode (if known):
- Microphone device / audio backend notes (if relevant):

## Build + config

- Repo git commit (short hash):
- Build profile: `--release`
- Binary path used:
- CLI args used:
- Config file path used:
- Model artifacts used (paths + model name, e.g. `tiny.en`):

## Raw artifacts

- `transcript.jsonl` (stdout capture): yes/no
- `timing.log` (stderr capture): yes/no

## Results (computed from utterance JSONL)

Metric: `e2e_ms` (end-of-speech → transcript JSONL emit)

- Sample count \(N\):
- p50 (ms):
- p95 (ms):
- p99 (ms):

PASS/FAIL (Phase 2 target): **PASS if p95 < 500ms**

## Notes

- Environmental noise / interruptions:
- Anomalies (dropouts, stalls, unexpected logs):
- Anything that might inflate tail latency:
