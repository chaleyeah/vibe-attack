# Phase 3: Phrase Matching + Dispatch - Context

**Gathered:** 2026-04-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver the **decision and action engine** that bridges transcripts to in-game macros:

1. **Fuzzy Phrase Matching**: Match transcripts against configured phrases using Levenshtein distance with a configurable confidence threshold.
2. **Conditional Logic**: Support `if/else` branches in macros based on simple boolean state flags.
3. **Sound Feedback**: Play audio clips on macro activation using the `rodio` backend.
4. **Dispatcher thread**: A dedicated OS thread that orchestrates the flow from STT result -> match -> condition check -> macro injection.

New capabilities (Pack system, GUI, AppImage packaging) are out of scope for this phase.

</domain>

<decisions>
## Implementation Decisions

### Phrase Matching (STT-02)

- **D-01:** Use **Levenshtein distance** (edit distance) for fuzzy phrase matching. This provides a robust and tunable mechanism for handling minor STT mispronunciations.
- **D-02:** Matching logic:
    - Normalization: Transcripts and target phrases are **lowercased and trimmed**.
    - Punctuation: Strip common punctuation (commas, periods, exclamation marks) before matching.
    - Score Calculation: `1.0 - (distance / max(len_transcript, len_phrase))`.
- **D-03:** Default **confidence threshold is 0.8**. If no phrase exceeds this threshold, the event is logged as `NO_MATCH` and no macro fires.

### Conditional Logic (MCRO-03)

- **D-04:** Implement **Boolean State Flags**. Macros can include an `if` condition that checks a named flag (e.g., `if is_stratagem_menu_open`).
- **D-05:** Flags are **process-local and transient** (reset on daemon restart).
- **D-06:** Macros can include a special `set_flag` action to toggle or set these flags, enabling "stateful" voice control (e.g., "Open Menu" macro sets the flag; "Close Menu" macro clears it).

### Sound Feedback (MCRO-04)

- **D-07:** Use **`rodio`** as the audio playback backend. It provides a high-level API for playing WAV, MP3, and Ogg files with low latency.
- **D-08:** Playback happens on a **background thread managed by rodio** (the Dispatcher just enqueues the sound).
- **D-09:** Sound files are referenced by **relative or absolute paths** in the macro definition. Missing files emit a `WARN` log but do not block the macro's key sequence.

### Threading & Integration

- **D-10:** A dedicated **Dispatcher thread (OS thread)** sits between the STT result queue and the Output (JSONL) thread.
- **D-11:** The Dispatcher receives `SttResult` from the STT service, performs matching, checks conditions, and sends `MacroCmd` to the existing injection thread's channel.
- **D-12:** The Dispatcher then forwards the (possibly matched) result to the Output thread for final logging.

### Claude's Discretion

- Exact JSON schema for conditional macro definitions in `config.yaml`.
- The internal registry structure for boolean flags.
- Optimization of the Levenshtein calculation (e.g., skip calculation if length difference is too large).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Roadmap + requirements
- `.planning/ROADMAP.md` — Phase 3 goal + success criteria (HD2 demo, threshold, logic, sound)
- `.planning/REQUIREMENTS.md` — Requirement definitions (STT-02, MCRO-03, MCRO-04)
- `.planning/PROJECT.md` — Wayland-first, local-only, AGPL-3.0

### Existing implementation
- `src/input/inject.rs` — `MacroCmd` enum and injection thread interface.
- `src/pipeline/coordinator.rs` — Current VAD -> STT -> Output flow to be intercepted.
- `src/stt/mod.rs` — `SttResult` structure.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `hd_linux_voice::input::inject::MacroCmd`: Already supports `KeySequence` and `Shutdown`. Phase 3 will likely extend this or the macro execution logic.
- `src/pipeline/coordinator.rs`: The `spawn_pipeline` function is the integration point for the new Dispatcher.

### Established Patterns
- **OS threads for blocking work**: The Dispatcher and Sound playback must not run on the Tokio executor.
- **Fail-fast preflight**: Any sound feedback initialization (rodio device check) should happen during startup.

### Integration Points
- `src/main.rs`: Needs to pass the `macro_tx` (injection channel) into `spawn_pipeline`.
- `src/config.rs`: Needs to be updated to support macro conditions and sound file paths.

</code_context>

<specifics>
## Specific Ideas

- The "Eagle Airstrike" demo should be the primary test case.
- Sound feedback should be "fire and forget"—we don't wait for the sound to finish before starting the key sequence.

</specifics>

<deferred>
## Deferred Ideas

- **Phonetic Matching**: Deferred to a future optimization phase if Levenshtein proves insufficient.
- **Complex Expressions**: Full scripting language deferred to v2.

</deferred>

---

*Phase: 03-phrase-matching-dispatch*
*Context gathered: 2026-04-23*
