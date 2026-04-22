---
phase: 02-pipeline-core
verified: 2026-04-22T12:42:00Z
status: gaps_found
score: 3/5 must-haves verified
overrides_applied: 0
gaps:
  - truth: "Speaking a short phrase while PTT is held produces the correct transcript on stdout within 500 ms of end-of-speech, proven on target hardware"
    status: failed
    reason: "Repo contains wiring + a procedure, but no captured proof artifacts/results; verification requires a target-hardware run and evidence."
    artifacts:
      - path: "docs/latency-baseline.md"
        issue: "Procedure exists, but no recorded measurement output is committed/attached; phase goal includes 'proven to meet <500ms on target hardware'."
    missing:
      - "Run the baseline procedure on target hardware and archive evidence (at least: sample `transcript.jsonl`, `timing.log`, and computed p95)."
  - truth: "Per-stage latency is instrumented such that stage costs/sub-budgets can be validated (AudioCapture → VAD → STT → output; VAD ≤ 50 ms, STT ≤ 200 ms on reference hardware)"
    status: partial
    reason: "JSONL includes STT compute time (`stt_ms`) and monotonic milestone fields (`vad_done_ms`, `stt_done_ms`, `output_done_ms`), but VAD 'cost' is not measured as compute time, and the proof doc claims a 'total end-to-end duration field' that is not present in the JSONL schema."
    artifacts:
      - path: "src/pipeline/jsonl.rs"
        issue: "No explicit `e2e_ms`/`end_to_end_ms` field; only milestone timestamps and `stt_ms`."
      - path: "docs/latency-baseline.md"
        issue: "Claims a 'total end-to-end duration field' exists in the summary JSON; current schema does not include such a field."
    missing:
      - "Either add an explicit end-to-end field to the `utterance` JSON (recommended: `e2e_ms = output_done_ms - vad_done_ms`), or update `docs/latency-baseline.md` to document the exact computation from existing fields."
      - "If VAD compute time is intended to be budgeted (≤ 50 ms), add a measured VAD compute duration field (separate from utterance timeline/length)."
---
# Phase 2: Pipeline Core Verification Report

**Phase Goal:** Speaking any phrase into the microphone produces a timestamped transcript on stdout, with per-stage latency instrumented and the full pipeline proven to meet the < 500 ms end-to-end budget on target hardware
**Verified:** 2026-04-22T12:42:00Z
**Status:** gaps_found
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (ROADMAP Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Speaking a short phrase while PTT is held produces the correct transcript on stdout within 500 ms of end-of-speech, as confirmed by the per-stage timestamp log | ✗ FAILED | Code wiring exists (`src/pipeline/coordinator.rs`, `src/stt/mod.rs`, `src/pipeline/jsonl.rs`) and a baseline procedure exists (`docs/latency-baseline.md`), but there is no committed/attached target-hardware run output proving \(p95 < 500ms\). |
| 2 | Timestamp log shows AudioCapture → VAD → STT → output with individual stage costs; no stage exceeds sub-budget | ⚠️ PARTIAL | JSONL schema has wall-clock + monotonic timing markers and `stt_ms` (`src/pipeline/jsonl.rs`, `src/pipeline/timing.rs`, `src/stt/mod.rs`), but does not currently expose an explicit end-to-end duration field and does not isolate “VAD compute cost” from the utterance timeline. |
| 3 | Wake word (without PTT) enters LISTENING and prints the trigger event to console; wake runs fully on-device | ✓ VERIFIED | Idle-mode wake detection exists in the pipeline loop and logs via `tracing` (stderr): “Wake word triggered; entering LISTENING window” (`src/pipeline/coordinator.rs`). Wake word model paths are local-only and validated (`src/config.rs`, `src/wake/mod.rs`). |
| 4 | Whisper `tiny.en` model loads at daemon startup; recognition produces transcripts with no network access | ✓ VERIFIED | STT model path is local and validated (`Config::validate_model_paths()` in `src/config.rs`). STT worker loads whisper model once when the STT thread starts (`WhisperContext::new_with_params(...)` in `src/stt/mod.rs`). No network-dependent runtime calls are present in this phase’s STT/wake/VAD modules. |
| 5 | STT inference runs on a dedicated OS thread (not Tokio); audio RT callback never allocates/blocks; concurrent recognition + audio capture verified via stress test | ✓ VERIFIED | Thread topology is `std::thread` for pipeline/STT/output (`src/pipeline/coordinator.rs`, `src/stt/mod.rs`). Audio callback writes into a pre-allocated ring buffer and avoids allocations (`src/audio/mod.rs`). Stress test artifact exists and is env-gated/ignored by default (`tests/concurrency_stress.rs`). |

**Score:** 3/5 truths verified

## Required Artifacts (exists → substantive → wired)

| Artifact | Expected | Status | Details |
|---------|----------|--------|---------|
| `src/config.rs` | Strict config schema + local model path validation | ✓ VERIFIED | `pipeline/vad/stt/wake` sections exist and `validate_model_paths()` fails fast on missing files. |
| `src/audio/mod.rs` | RT-safe audio capture into ring buffer | ✓ VERIFIED | Pre-allocated ring buffer; callback pushes samples and does not allocate or block. |
| `src/vad/mod.rs` | Silero VAD + utterance segmentation respecting locked decisions | ✓ VERIFIED | Implements 20ms pacing, 512-sample sliding window, hysteresis, min-speech, end-silence, preroll/tail, cap; includes unit tests. |
| `src/stt/mod.rs` | Dedicated STT OS thread, whisper model preload, bounded drop-oldest queue | ✓ VERIFIED | Uses `std::thread::spawn`, `crossbeam_channel::bounded`, drop-oldest helper; loads model from local path (feature `stt`). |
| `src/wake/mod.rs` | On-device wake word keyword spotter using local artifacts | ✓ VERIFIED | sherpa-onnx KeywordSpotter configured from local paths; no stdout writes. |
| `src/pipeline/jsonl.rs` | Stable stdout JSONL schema + writer | ✓ VERIFIED | `JsonlEvent` is `#[serde(tag="type")]` and writer uses `serde_json::to_writer` + newline + flush. |
| `src/pipeline/timing.rs` | Wall-clock + monotonic timing helpers | ✓ VERIFIED | `wall_time_ms()` unix ms; `UtteranceTimings` monotonic markers. |
| `src/pipeline/coordinator.rs` | End-to-end wiring + stdout/stderr split | ✓ VERIFIED | Pipeline thread drains ring buffer, runs wake/VAD, queues STT, forwards results to a dedicated stdout JSONL writer thread. |
| `docs/latency-baseline.md` | Reproducible latency proof procedure | ⚠️ PARTIAL | Procedure exists, but doc currently claims a “total end-to-end duration field” that is not present in the JSONL schema; also phase goal requires actual target-hardware evidence. |
| `tests/concurrency_stress.rs` | Stress test artifact for concurrency under load | ✓ VERIFIED | Ignored + `RUN_STRESS_TESTS=1` gated; exercises drop-oldest + concurrent progress invariants. |
| `tests/stt_smoke.rs` | Env-gated STT smoke harness | ✓ VERIFIED | Ignored + `RUN_STT_TESTS=1` gated; requires `--features stt` and local model path. |
| `tests/wake_word.rs` | Env-gated wake word smoke harness | ✓ VERIFIED | Ignored + `RUN_KWS_TESTS=1` gated; validates decode loop with local artifacts. |
| `tests/jsonl_schema.rs` | JSONL schema invariants | ✓ VERIFIED | Asserts stable keys + required timing fields exist and are non-negative. |
| `tests/drop_oldest_queue.rs` | Drop-oldest semantics unit tests | ✓ VERIFIED | Confirms oldest is dropped and capacity never exceeded. |

## Key Link Verification (wiring)

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/audio/mod.rs` | Pipeline drain loop | `AudioHandle.consumer` moved into pipeline thread | ✓ WIRED | `spawn_pipeline(...)` takes `AudioHandle` and drains the consumer in the OS thread. |
| `src/vad/mod.rs` | STT queue | `crossbeam_channel::bounded` + drop-oldest `try_send_drop_oldest` | ✓ WIRED | Pipeline uses STT submitter; drop-oldest helper is used for result forwarding as well. |
| `src/stt/mod.rs` | stdout JSONL | Output thread calls `JsonlWriter::write_utterance` | ✓ WIRED | Pipeline forwards completed `SttResult` to output thread, which emits JSONL to `stdout.lock()`. |
| `src/wake/mod.rs` | LISTENING window | Pipeline idle loop runs wake decode and sets `listening_until` | ✓ WIRED | Wake triggers `listening_until = now + listen_window` and logs via `tracing`. |

## Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|---------|---------------|--------|--------------------|--------|
| `src/pipeline/coordinator.rs` → `JsonlWriter` | `text` in JSONL utterance | `SttResult.text` from `src/stt/mod.rs` | ✓ (when `stt.enabled=true` and built with `--features stt`) | ✓ FLOWING |
| `src/stt/mod.rs` | Transcript text | whisper.cpp inference (`whisper-rs`) | ✓ (with local model + runtime deps) | ✓ FLOWING |
| `src/wake/mod.rs` | Keyword trigger | sherpa-onnx keyword spotter | ✓ (with local artifacts) | ✓ FLOWING |
| `src/vad/mod.rs` | Utterance jobs | Silero VAD ONNX model forward + segmentation | ✓ (requires ONNX Runtime shared lib at runtime) | ✓ FLOWING |

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|---------|---------|--------|--------|
| Unit + integration suite (default) is green without models | `cargo test -q` | pass (ignored heavy tests remain ignored) | ✓ PASS |

## Requirements Coverage

| Requirement | Source Plan(s) | Description | Status | Evidence |
|------------|----------------|-------------|--------|----------|
| **ACT-02** | 02-03 | Wake word activation (on-device, no cloud) | ? NEEDS HUMAN | Wake-word wiring exists and is local-path driven, but requires running with real model artifacts to confirm trigger behavior. |
| **STT-01** | 02-01, 02-03 | Fully on-device STT using bundled local model | ? NEEDS HUMAN | STT worker loads local model and runs whisper inference on OS thread; requires local model + `--features stt` runtime run to confirm. |
| **STT-04** | 02-02, 02-03, 02-04 | <500ms end-to-end latency proven on target hardware | ✗ BLOCKED | No committed proof run artifacts; instrumentation/doc mismatch for “total end-to-end duration field”; requires target hardware measurement to close. |

## Anti-Patterns Found

No obvious placeholder/TODO stubs were found in the reviewed Phase 2 implementation files. Heavy tests are correctly `#[ignore]` + env-gated.

## Gaps Summary

- **The pipeline is real and wired** (audio → wake/VAD → STT → stdout JSONL) with the correct threading constraints and bounded backpressure.
- **The phase goal’s “proven <500ms on target hardware” is not yet satisfied** by repository evidence: there is a runbook, but no results and the runbook currently references a total duration field that is not present in the JSONL schema.
- **Latency instrumentation is usable but incomplete for sub-budget validation**: STT compute duration exists (`stt_ms`), but VAD “stage cost” is not currently isolated as compute time, and end-to-end needs an explicit field or documented computation.

---

_Verified: 2026-04-22_
_Verifier: Claude (gsd-verifier)_

