# Phase 2: Pipeline Core - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `02-CONTEXT.md` — this log preserves the alternatives considered.

**Date:** 2026-04-22
**Phase:** 2 - Pipeline Core
**Areas discussed:** Pipeline shape + buffering, VAD behavior, STT integration, Latency/stdout contract, Wake word

---

## Pipeline shape + buffering

| Decision | Options considered | Selected |
|---|---|---|
| Frame size | 20ms frames \| 30ms frames \| 100ms+ chunks | ✓ 20ms |
| Thread topology | single thread VAD+STT \| **two-stage VAD + blocking STT thread** \| worker pool | ✓ two-stage |
| Backpressure | drop-oldest \| drop-new \| block | ✓ drop-oldest |
| PTT vs VAD | VAD inside PTT \| PTT-only utterance \| hybrid | ✓ VAD inside PTT |
| Pacing | short sleep + poll \| busy-spin \| callback notify | ✓ short sleep + poll |
| Pre-roll | ~100–200ms \| none \| configurable now | ✓ ~100–200ms |
| Post-roll | ~100–200ms \| none \| configurable now | ✓ ~100–200ms |
| Utterance cap | ~10s \| ~5s \| unlimited | ✓ ~10s |

---

## VAD behavior (Silero)

| Decision | Options considered | Selected |
|---|---|---|
| VAD cadence | 20ms \| 100ms batch \| adaptive | ✓ 20ms |
| Speech start stability | single threshold \| **hysteresis** \| N-of-M vote | ✓ hysteresis |
| End-of-speech silence | 250ms \| **400ms** \| 600ms | ✓ 400ms |
| Min speech before commit | **100ms** \| none \| 300ms | ✓ 100ms |

---

## STT integration (whisper.cpp)

| Decision | Options considered | Selected |
|---|---|---|
| Binding approach | **`whisper-rs`** \| lower-level sys/FFI \| shell out to CLI | ✓ `whisper-rs` |
| Transcript style | **final-only** \| partial streaming \| per-segment | ✓ final-only |
| Model location | **config path** \| bundled \| auto-download | ✓ config path |
| Compute target | **CPU-only** \| CPU + optional GPU \| GPU-first | ✓ CPU-only |

---

## Latency + stdout contract

| Decision | Options considered | Selected |
|---|---|---|
| Output format | **JSONL** \| human-readable \| both | ✓ JSONL |
| stdout/stderr split | **stdout transcript + stderr instrumentation** \| all stdout \| all stderr | ✓ split |
| Timestamp basis | **both wall-clock + monotonic** \| wall-clock only \| monotonic only | ✓ both |
| Event granularity | one summary \| multi-event \| **hybrid** | ✓ hybrid |

---

## Wake word (ACT-02)

| Decision | Options considered | Selected |
|---|---|---|
| Wake word backend | **sherpa-onnx keyword spotter** \| Porcupine \| other | ✓ sherpa-onnx |
| LISTENING window | **~5s window** \| until speech \| manual cancel | ✓ ~5s |

---

## Claude's Discretion

- Threshold numeric tuning inside the chosen hysteresis model.
- Exact JSON field naming (consistent + stable).
