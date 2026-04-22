# Phase 2: Pipeline Core - Context

**Gathered:** 2026-04-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver the **core voice pipeline** for the daemon:

- Ingest the existing **16kHz f32** audio stream (already warm + PTT-gated)
- Segment speech using **VAD (Silero)**
- Run **on-device STT (whisper.cpp)** with `tiny.en` loaded at startup
- Emit **timestamped transcripts** and **per-stage latency instrumentation** proving the \(< 500ms\) end-to-end budget on target hardware
- Support **wake word activation** (without PTT) to enter a temporary LISTENING state

New capabilities (phrase matching, macro dispatch, pack system, UI) are out of scope for this phase.

</domain>

<decisions>
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

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Roadmap + requirements
- `.planning/ROADMAP.md` — Phase 2 goal + success criteria (latency budgets, wake word, preload)
- `.planning/REQUIREMENTS.md` — Requirement definitions (ACT-02, STT-01, STT-04)
- `.planning/PROJECT.md` — Product constraints: local-only, Wayland-first, AGPL-3.0
- `.planning/STATE.md` — Prior phase decisions + recorded Phase 2 risks/concerns

### Existing implementation to integrate with
- `src/audio/mod.rs` — Ringbuffer-based, allocation-free audio callback + `AudioHandle.consumer`
- `src/main.rs` — Threading conventions and “STT never on Tokio” guardrails

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `hd_linux_voice::audio::AudioHandle` (`src/audio/mod.rs`): provides a **ringbuffer consumer** intended for Phase 2 to drain audio.
- `tokio_util::sync::CancellationToken` usage in `src/main.rs`: existing cancellation pattern for threads.

### Established Patterns
- **Fail-fast preflight** before spawning threads (permissions, device availability).
- Latency-sensitive work (injection thread; future STT) runs on **dedicated OS threads**, not the Tokio executor.
- Logging is **warn by default**, debug/trace behind `-v/-vv`.

### Integration Points
- `src/main.rs` currently starts audio but ignores `AudioHandle.consumer` — Phase 2 should wire this into the VAD→STT pipeline.
- `src/config.rs` currently lacks VAD/STT/wake-word sections — Phase 2 will extend config schema and example config accordingly.

</code_context>

<specifics>
## Specific Ideas

- Keep the **audio callback allocation-free** and avoid any “notify” mechanisms that would risk allocations/locks inside the callback.
- Preserve the “daemon piping” UX: stdout stays clean and machine-readable (JSONL transcript), with diagnostics on stderr.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 02-pipeline-core*
*Context gathered: 2026-04-22*
