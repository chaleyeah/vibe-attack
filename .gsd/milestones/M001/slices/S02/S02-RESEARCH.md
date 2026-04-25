# Phase 2: Pipeline Core - Research

**Researched:** 2026-04-22  
**Domain:** On-device voice pipeline (VAD + wake word + STT) with real-time/threading constraints  
**Confidence:** MEDIUM (core APIs verified; performance/latency still requires empirical measurement on target hardware)

<user_constraints>
## User Constraints (from `02-CONTEXT.md`)

### Locked Decisions

## Implementation Decisions

### Pipeline shape + buffering

- **D-01:** Frame audio as **fixed 20ms frames** (320 samples @ 16kHz).
- **D-02:** Use a **2-stage thread topology**: VAD on one thread; STT on a **separate blocking OS thread**, connected via a **bounded queue**.
- **D-03:** Backpressure policy is **drop-oldest** on the bounded queue to preserve responsiveness under load.
- **D-04:** While PTT is held, **still run VAD** to trim leading/trailing silence (PTT gates capture; VAD gates utterance boundaries).
- **D-05:** Pipeline pacing: **short sleep + poll** when insufficient samples (no busy-spin; avoid complex callback notifications).
- **D-06:** Keep a rolling **pre-roll buffer (~100–200ms)** and prepend it when speech starts to avoid clipping.
- **D-07:** Append a **tail (~100–200ms)** after end-of-speech to avoid truncating endings.
- **D-08:** **Cap utterance length at ~10 seconds**; force a cut/flush and log a warning if exceeded.

### VAD behavior (Silero)

- **D-09:** Run VAD scoring **every 20ms frame**.
- **D-10:** Use **hysteresis thresholds** for stability (start threshold > stop threshold).
- **D-11:** End-of-speech requires **~400ms of silence** before cutting the utterance.
- **D-12:** Require **~100ms minimum speech** before committing to an utterance (noise spike protection).

### STT integration (whisper.cpp)

- **D-13:** Use **`whisper-rs`** as the Rust integration layer for whisper.cpp.
- **D-14:** Print **final transcript only** per utterance (no streaming partials by default).
- **D-15:** Model path is **config-driven** (e.g., `stt.model_path`), with a documented default location.
- **D-16:** Phase 2 baseline targets **CPU-only** inference (no GPU requirement).

### Wake word (ACT-02)

- **D-17:** Wake word backend: **sherpa-onnx keyword spotter** (fully on-device).
- **D-18:** After wake word trigger, enter LISTENING for a **fixed ~5s window**; return to idle if no speech is detected.

### Latency + stdout contract

- **D-19:** Transcript output format is **JSONL**.
- **D-20:** Stream split: **transcript JSONL on stdout**; instrumentation/status/log events on **stderr**.
- **D-21:** Include **both** wall-clock time (RFC3339 or unix ms) **and** monotonic duration fields in JSON events.
- **D-22:** Hybrid event model: always emit a **single summary “utterance” event**, and allow **optional detailed stage events** behind higher verbosity.

### Claude's Discretion

- Exact numeric values for hysteresis thresholds (within the chosen hysteresis approach) as long as they meet latency/accuracy goals.
- Exact JSON field names, as long as the contract remains stable and parseable.

### Deferred Ideas (OUT OF SCOPE)

## Deferred Ideas

None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ACT-02 | Wake word activation via configurable wake word (on-device model, no cloud) | `sherpa-onnx` keyword spotter crate + documented keyword file format and Rust example; keep CPU-only inference and local model artifacts. [VERIFIED: crates.io via `cargo info sherpa-onnx`], [CITED: https://k2-fsa.github.io/sherpa/onnx/kws/index.html], [CITED: https://raw.githubusercontent.com/k2-fsa/sherpa-onnx/master/rust-api-examples/examples/keyword_spotter.rs] |
| STT-01 | STT runs fully on-device using bundled local model (whisper.cpp or equivalent) | `whisper-rs` crate loads model from path and runs `WhisperState::full()` on local PCM f32 audio. [VERIFIED: crates.io via `cargo info whisper-rs`], [CITED: https://docs.rs/whisper-rs/latest/whisper_rs/], [CITED: https://codeberg.org/tazz4843/whisper-rs/raw/branch/master/README.md] |
| STT-04 | End-to-end latency < 500ms (end of speech → first key event) | Instrumentation contract: monotonic stage durations + wall clock timestamps; enforce bounded queues + drop-oldest; cap utterance length; pre-load models at startup; validate with a latency-focused integration test that prints stage timings. [ASSUMED] |
</phase_requirements>

## Summary

Phase 2 is mostly about **correct thread topology + buffering** and making the pipeline **measurable**. The codebase already provides an allocation-free audio RT callback that pushes 16kHz mono f32 samples into a pre-allocated ring buffer (`AudioHandle.consumer`) and a clear precedent that latency-sensitive work should live on **dedicated OS threads**, not Tokio. [VERIFIED: local code `src/audio/mod.rs`, `src/main.rs`]

The critical integration facts for planning:
- **Silero VAD (`silero-vad-rust`) is ONNX Runtime–backed and expects an ONNX Runtime shared library at runtime**; its own windowing logic uses **512 samples at 16kHz**. This creates a planning requirement: keep the system’s “20ms pacing” decision, but internally accumulate/slide into 512-sample windows for VAD inference while still emitting 20ms-resolution decisions (e.g., score each 20ms by running VAD on the latest 512-sample window). [VERIFIED: crates.io via `cargo info silero-vad-rust`], [CITED: https://raw.githubusercontent.com/sheldonix/silero-vad-rust/master/README.md], [CITED: https://docs.rs/silero-vad-rust/latest/src/silero_vad_rust/silero_vad/utils_vad.rs.html]
- **whisper.cpp integration via `whisper-rs`** provides a straightforward “load model once → create state per job → run `state.full()`” API and supports redirecting whisper.cpp logs. Build commonly relies on system tooling (notably `cmake`). [VERIFIED: crates.io via `cargo info whisper-rs`], [CITED: https://docs.rs/whisper-rs/latest/whisper_rs/], [CITED: https://codeberg.org/tazz4843/whisper-rs/raw/branch/master/README.md]
- **Wake word via `sherpa-onnx` keyword spotter** is a local model path + tokens + keyword file driven system, with a Rust example that demonstrates stream-based decoding and keyword result JSON. [VERIFIED: crates.io via `cargo info sherpa-onnx`], [CITED: https://k2-fsa.github.io/sherpa/onnx/kws/index.html], [CITED: https://raw.githubusercontent.com/k2-fsa/sherpa-onnx/master/rust-api-examples/examples/keyword_spotter.rs]

**Primary recommendation:** Plan around a **3-thread pipeline** (Audio RT → VAD/Wake thread → STT blocking thread) with a bounded drop-oldest queue between VAD and STT, and treat “<500ms” as a *measured artifact* (JSONL stdout + structured stderr stage timings), not an emergent property.

## Project Constraints (from `.cursor/rules/`)

No `.cursor/rules/` directory found in this repo. [VERIFIED: local glob]

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Audio ingest (16kHz f32), RT-safe buffering | Audio RT thread (CPAL callback) | VAD thread | Callback must remain allocation-free; ring buffer is already pre-allocated. [VERIFIED: local code `src/audio/mod.rs`] |
| Wake word detection | VAD/Wake OS thread | — | Must not block RT callback; should run continuously in idle mode without PTT. [ASSUMED] |
| VAD scoring + utterance segmentation | VAD OS thread | — | Silero VAD uses ONNX runtime and internal state; keep off Tokio and out of RT callback. [CITED: https://docs.rs/silero-vad-rust/latest/silero_vad_rust/] |
| STT inference (whisper.cpp tiny.en) | STT blocking OS thread | — | Hard constraint: never on Tokio; CPU-heavy; isolate from wake/VAD. [VERIFIED: Phase context D-02], [CITED: https://docs.rs/whisper-rs/latest/whisper_rs/] |
| Output (JSONL transcripts) | Main thread / IO writer | STT thread | stdout contract must stay clean and line-buffered; stderr used for instrumentation. [VERIFIED: Phase context D-19/D-20] |
| Latency instrumentation | Pipeline coordinator (main + threads) | — | Needs monotonic timestamps taken at stage boundaries near the stage work. [VERIFIED: Phase context D-21] |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `silero-vad-rust` | 6.2.1 | Silero VAD inference + helper iterators | Rust wrapper around Silero ONNX VAD with streaming helpers. [VERIFIED: crates.io via `cargo info silero-vad-rust`], [CITED: https://docs.rs/silero-vad-rust/latest/silero_vad_rust/] |
| `whisper-rs` | 0.16.0 | whisper.cpp model loading + transcription | Direct whisper.cpp binding with a stable “`WhisperState::full()`” API. [VERIFIED: crates.io via `cargo info whisper-rs`], [CITED: https://docs.rs/whisper-rs/latest/whisper_rs/] |
| `sherpa-onnx` | 1.12.39 | On-device keyword spotting (wake word) | Provides keyword spotter config + stream decoding; supports static linking by default. [VERIFIED: crates.io via `cargo info sherpa-onnx`], [CITED: https://docs.rs/sherpa-onnx] |
| `crossbeam-channel` | 0.5.15 | Bounded queue between stages | MPMC bounded channels; supports non-blocking `try_send`/`try_recv` useful for drop-oldest. [VERIFIED: crates.io via `cargo search crossbeam-channel`] |
| `serde_json` | 1.0.149 | JSONL output encoding | Matches existing `serde` stack; stable, ubiquitous. [VERIFIED: crates.io via `cargo search serde_json`] |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tracing` | 0.1.x (already in repo) | Structured logging to stderr | Use for internal status; do not mix with stdout JSONL. [VERIFIED: local `Cargo.toml`] |
| `tokio-util` (`CancellationToken`) | 0.7.x (already in repo) | Cooperative thread shutdown | Use to stop VAD/STT threads cleanly. [VERIFIED: local code `src/main.rs`] |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `crossbeam-channel` | `flume` 0.12.0 | `flume` is ergonomic and fast but adds another dependency; crossbeam is very standard and simple for drop-oldest via try-ops. [VERIFIED: crates.io via `cargo search flume`] |
| `silero-vad-rust` | whisper.cpp built-in VAD (`WhisperVadContext`) | Built-in VAD exists but Phase decision locked to Silero; Silero-specific tuning + ONNX runtime packaging risk must be handled anyway. [CITED: https://docs.rs/whisper-rs/latest/whisper_rs/] |

**Installation (Cargo):**

```toml
# Cargo.toml
[dependencies]
silero-vad-rust = "6.2.1"
whisper-rs = "0.16.0"
sherpa-onnx = "1.12.39"
crossbeam-channel = "0.5.15"
serde_json = "1.0.149"
```

## Architecture Patterns

### System Architecture Diagram

```text
                (allocation-free)
Mic -> CPAL RT callback -> ringbuf producer  ---------------------------+
                                                                       |
                                                                       v
                                                          ringbuf consumer drain
                                                               (poll + short sleep)
                                                                       |
                                                                       v
  Idle Mode (no PTT)                                       Active Mode (PTT or wake LISTENING)
  ------------------                                       -------------------------------
  WakeWord: sherpa-onnx keyword spotter                     VAD: silero-vad-rust (ONNX)
    - consumes rolling audio                                 - consumes rolling audio
    - emits Trigger event                                    - emits Start/End events
            |                                                - includes pre-roll + tail
            v
      LISTENING state (≈5s window)
            |
            v
  Utterance builder (pre-roll + speech + tail, cap 10s)
            |
            v
  Bounded queue (drop-oldest)  -->  STT OS thread (whisper-rs/whisper.cpp)
                                         - model loaded at startup
                                         - run state.full() per utterance
                                         - emits final transcript
            |
            v
stdout JSONL (transcript events)          stderr (instrumentation/status events)
```

### Recommended Project Structure

```text
src/
├── pipeline/                 # Orchestrator + state machine (IDLE/LISTENING/RECORDING)
│   ├── mod.rs
│   ├── timing.rs             # monotonic + wall clock capture
│   └── jsonl.rs              # stdout JSONL encoding
├── vad/                      # Silero VAD wrapper + hysteresis + segmentation
├── stt/                      # whisper-rs wrapper (model load, state, transcription)
├── wake/                     # sherpa-onnx keyword spotter wrapper
└── main.rs                   # wires everything, owns stdout/stderr separation
```

### Pattern 1: Silero streaming iterator with 20ms pacing
**What:** Keep D-01’s 20ms pacing for pipeline decisions, but run Silero inference on 512-sample (32ms) windows by maintaining an internal sliding buffer that advances every 20ms.
**When to use:** Always (Silero’s default 16kHz window size is 512 samples; feeding smaller chunks is under-documented and risks incorrect state/probabilities). [CITED: https://docs.rs/silero-vad-rust/latest/src/silero_vad_rust/silero_vad/utils_vad.rs.html]
**Example (conceptual):**

```rust
// Source: silero-vad-rust docs show 512-sample window at 16kHz
// https://docs.rs/silero-vad-rust/latest/silero_vad_rust/
//
// Keep a VecDeque<f32> window of 512 samples.
// Every 20ms (320 samples), push 320 new samples, pop 320 old samples,
// then call iterator.process_chunk(&window[..], ...).
```

### Pattern 2: whisper-rs “load once, transcribe on demand”
**What:** Load `tiny.en` at startup into a `WhisperContext`; create a `WhisperState` per utterance (or reuse carefully if the API allows), run `state.full(params, audio_f32)` on the STT OS thread.
**When to use:** Every utterance, on the dedicated blocking OS thread (never on Tokio).
**Example:**

```rust
// Source: whisper-rs docs/README
// https://docs.rs/whisper-rs/latest/whisper_rs/
// https://codeberg.org/tazz4843/whisper-rs/raw/branch/master/README.md
use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};

let ctx = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())?;
let params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
let mut state = ctx.create_state()?;
state.full(params, &utterance_audio_f32)?;
```

### Pattern 3: sherpa-onnx keyword spotter stream decode loop
**What:** Construct `KeywordSpotterConfig` with local paths (encoder/decoder/joiner/tokens/keywords_file), accept waveform, decode until ready, read keyword result JSON.
**When to use:** In IDLE mode continuously (lightweight), and/or as a gate to enter LISTENING state.
**Example:**

```rust
// Source: official sherpa-onnx Rust example
// https://raw.githubusercontent.com/k2-fsa/sherpa-onnx/master/rust-api-examples/examples/keyword_spotter.rs
use sherpa_onnx::{KeywordSpotter, KeywordSpotterConfig};
```

### Anti-Patterns to Avoid
- **Running inference inside the CPAL callback:** violates the RT no-alloc/no-block invariant. [VERIFIED: local code `src/audio/mod.rs`]
- **Using Tokio for STT work (spawn_blocking or otherwise):** Phase decision explicitly forbids it; keep STT on a dedicated OS thread. [VERIFIED: Phase context], [VERIFIED: local code patterns in `src/main.rs`]
- **Mixing stdout JSONL with log lines:** breaks downstream piping; keep *all* logs/instrumentation on stderr. [VERIFIED: Phase context D-19/D-20]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Keyword spotting | Custom DSP wake-word model | `sherpa-onnx` keyword spotter | Keyword decoding, keyword file format, model artifacts are non-trivial. [CITED: https://k2-fsa.github.io/sherpa/onnx/kws/index.html] |
| VAD NN inference | Handwritten VAD network | `silero-vad-rust` | Already wraps Silero model + state tracking; focus on thresholds/hysteresis + buffering. [CITED: https://docs.rs/silero-vad-rust/latest/silero_vad_rust/] |
| STT engine | Homemade STT | `whisper-rs` | Stable whisper.cpp integration; avoid maintaining FFI bindings yourself. [CITED: https://docs.rs/whisper-rs/latest/whisper_rs/] |
| JSON encoding | Manual string building | `serde_json` | Correct escaping + schema evolution without footguns. [VERIFIED: crates.io via `cargo search serde_json`] |

**Key insight:** The complexity in Phase 2 is not “doing ML” but **wiring ML safely** under RT constraints and making it **observable** (stage timings, bounded backpressure, stable output).

## Common Pitfalls

### Pitfall 1: VAD windowing mismatch vs 20ms frames
**What goes wrong:** Planning assumes Silero can score 320-sample chunks directly; runtime produces unstable probabilities or incorrect state transitions.
**Why it happens:** Silero VAD helpers and examples use 512 samples at 16kHz; the crate’s helper logic is structured around that window size. [CITED: https://docs.rs/silero-vad-rust/latest/src/silero_vad_rust/silero_vad/utils_vad.rs.html]
**How to avoid:** Keep 20ms “frame clock” but score with a sliding 512-sample window advanced every 20ms (or buffer 2×20ms then pad to 512 as needed). Emit VAD decisions at 20ms resolution.
**Warning signs:** Frequent start/end flapping; utterances clipped; VAD probability trace seems noisy even on clear speech.

### Pitfall 2: ONNX Runtime deployment/bundling
**What goes wrong:** Silero VAD works on dev machine but fails in AppImage / other systems because `onnxruntime` shared library can’t be loaded.
**Why it happens:** `silero-vad-rust` expects ONNX Runtime shared library to be discoverable at runtime (dynamic load). [CITED: https://raw.githubusercontent.com/sheldonix/silero-vad-rust/master/README.md]
**How to avoid:** Plan explicit runtime discovery: document `ORT_DYLIB_PATH` and/or bundle the library in distribution artifacts; verify with an integration test that exercises `load_silero_vad()` in a minimal environment.
**Warning signs:** Runtime errors mentioning missing `onnxruntime`/`libonnxruntime.so`.

### Pitfall 3: whisper-rs build tooling missing
**What goes wrong:** Phase 2 planning assumes “Linux builds work out of the box”, but the dev environment lacks `cmake`, so whisper.cpp integration fails to compile.
**Why it happens:** whisper.cpp bindings are built from C/C++ sources; toolchain dependencies matter.
**How to avoid:** Include environment availability checks and add a “preflight: build whisper-rs” step in Wave 0.
**Warning signs:** Build failures in `whisper-rs-sys` / CMake configure steps.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Build/test | ✓ | `rustc 1.95.0`, `cargo 1.95.0` | — [VERIFIED: local `rustc --version`, `cargo --version`] |
| `cmake` | `whisper-rs` build (whisper.cpp) | ✗ | — | Install `cmake` (blocking). [VERIFIED: local `command -v cmake` produced no path] |
| `clang` | C/C++ compilation (whisper.cpp, sherpa-onnx) | ✓ | 22.1.3 | gcc also available. [VERIFIED: local `clang --version`] |
| `gcc` | C/C++ compilation | ✓ | 15.2.1 | — [VERIFIED: local `gcc --version`] |
| `pkg-config` | Native deps detection (varies) | ✓ | 2.5.1 | — [VERIFIED: local `pkg-config --version`] |
| ONNX Runtime shared library (`libonnxruntime.so`) | `silero-vad-rust` runtime | ✗ | — | Install/bundle ONNX Runtime 1.22.x; set `ORT_DYLIB_PATH` or system library path. [CITED: https://raw.githubusercontent.com/sheldonix/silero-vad-rust/master/README.md], [VERIFIED: local `ldconfig -p | rg onnxruntime` empty] |

**Missing dependencies with no fallback:**
- `cmake` (required to build whisper.cpp bindings in practice)
- ONNX Runtime shared library (required to *run* Silero VAD as configured by `silero-vad-rust`)

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (`cargo test`) |
| Config file | none |
| Quick run command | `cargo test -q` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| ACT-02 | Wake word triggers LISTENING and emits trigger event (stderr) | integration (gated) | `RUN_KWS_TESTS=1 cargo test --test wake_word -- --include-ignored` | ❌ Wave 0 |
| STT-01 | Local model load at startup and transcription produces text without network | integration (gated) | `RUN_STT_TESTS=1 cargo test --test stt_smoke -- --include-ignored` | ❌ Wave 0 |
| STT-04 | Latency instrumentation emits stage durations; pipeline meets <500ms on target | manual + scripted smoke | `cargo run -- --config ... | tee transcript.jsonl` (manual verify) | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -q`
- **Per wave merge:** `cargo test`
- **Phase gate:** A reproducible latency run on target hardware with archived stderr timing output and stdout JSONL transcript.

### Wave 0 Gaps
- [ ] Add integration-test harness that can be gated by env vars for heavy model tests (`RUN_STT_TESTS`, `RUN_KWS_TESTS`).
- [ ] Add unit tests for **drop-oldest bounded queue** semantics and for **JSONL schema stability**.
- [ ] Add a synthetic-audio fixture strategy that does not require network downloads (pre-checked-in tiny fixtures and/or ignored tests until local models are present).

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | Strict config schema (`deny_unknown_fields`) + typed parsing; validate paths exist at startup. [VERIFIED: local `src/config.rs`] |
| V6 Cryptography | no | — |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Untrusted config file causing crash | DoS | Fail-fast config validation + clear errors; deny unknown fields already in place. [VERIFIED: local `src/config.rs`, tests/config_parse.rs] |
| Unbounded memory growth from audio buffering | DoS | Bounded queues + ring buffer; cap utterance duration (D-08). [VERIFIED: Phase context D-03/D-08] |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Stage budgets (e.g., VAD ≤ 50ms, STT ≤ 200ms) are achievable with tiny.en on target hardware with the chosen thread topology | Summary / Requirements | If too slow, Phase 2 must revisit model size, quantization, or pipeline shape before Phase 3. |
| A2 | Scoring Silero on sliding 512-sample windows advanced every 20ms yields stable 20ms-resolution VAD decisions | Architecture Patterns | If inaccurate, utterance boundaries may regress; may need alternate windowing or parameter tuning. |

## Open Questions (RESOLVED)

1. **Where will the ONNX Runtime shared library come from in dev + packaging?**
   - **Resolved for Phase 2 (dev baseline):** Use the **system ONNX Runtime shared library** (`libonnxruntime.so`) and document how to install it on Arch/CachyOS. Allow overriding discovery via an explicit environment variable (e.g., `ORT_DYLIB_PATH`) for non-standard installs.
   - **Deferred (distribution/AppImage):** Bundling/redistributing ONNX Runtime for AppImage is a Phase 5 packaging concern; Phase 2 focuses on proving the pipeline locally with clear, actionable setup steps.

2. **Which exact sherpa-onnx KWS pretrained model artifacts will ship / be referenced by default?**
   - **Resolved for Phase 2:** Do **not** pick/bundle a default model artifact set yet. Require explicit paths in config for the needed sherpa-onnx artifacts (encoder/decoder/joiner/tokens/keywords file). Phase 2 proves the integration + behavior; Phase 5 can decide what to ship by default in release artifacts.

## Sources

### Primary (HIGH confidence)
- `silero-vad-rust` crates.io metadata via `cargo info` (version 6.2.1, MIT, features) [VERIFIED: crates.io]
- `whisper-rs` crates.io metadata via `cargo info` (version 0.16.0, Unlicense, features) [VERIFIED: crates.io]
- `sherpa-onnx` crates.io metadata via `cargo info` (version 1.12.39, Apache-2.0, static/shared features) [VERIFIED: crates.io]
- `silero-vad-rust` docs.rs + source (window size 512 at 16kHz in helper) [CITED: https://docs.rs/silero-vad-rust/latest/silero_vad_rust/], [CITED: https://docs.rs/silero-vad-rust/latest/src/silero_vad_rust/silero_vad/utils_vad.rs.html]
- `whisper-rs` docs.rs crate docs (WhisperState::full) [CITED: https://docs.rs/whisper-rs/latest/whisper_rs/]
- `whisper-rs` upstream README (Codeberg) [CITED: https://codeberg.org/tazz4843/whisper-rs/raw/branch/master/README.md]
- Sherpa-onnx keyword spotting documentation (keywords file format, trigger threshold/boosting) [CITED: https://k2-fsa.github.io/sherpa/onnx/kws/index.html]
- Sherpa-onnx Rust keyword spotter example [CITED: https://raw.githubusercontent.com/k2-fsa/sherpa-onnx/master/rust-api-examples/examples/keyword_spotter.rs]
- Local codebase: CPAL ring buffer invariants and existing thread patterns [VERIFIED: local code `src/audio/mod.rs`, `src/main.rs`]

### Secondary (MEDIUM confidence)
- None used beyond primary sources in this research.

### Tertiary (LOW confidence)
- None.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — exact crate names/versions and core APIs verified from crates.io + docs.rs.
- Architecture: MEDIUM — topology/constraints are locked, but the precise VAD windowing adaptation and end-to-end latency need empirical verification.
- Pitfalls: HIGH — build/runtime dependency gaps (cmake, onnxruntime) are confirmed in the current environment and upstream docs.

**Research date:** 2026-04-22  
**Valid until:** 2026-05-22 (recheck crate versions + sherpa model docs if planning slips)