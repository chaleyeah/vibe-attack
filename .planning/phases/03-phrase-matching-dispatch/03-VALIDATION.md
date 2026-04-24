# Phase 3: Phrase Matching + Dispatch - Validation

**Created:** 2026-04-23
**Phase:** 03-phrase-matching-dispatch

## Validation Strategy (Nyquist Dimension 8)

This phase introduces decision logic and action triggering. Validation must prove that the "brain" of the daemon correctly maps recognized phrases to macros under various conditions.

### 1. Automated Tests (Unit/Integration)

- **[x] Phrase Matching (Fuzzy)**:
    - Test normalization (case-insensitivity, whitespace).
    - Test Levenshtein distance matching against various threshold levels (0.5, 0.8, 1.0).
    - Test "no match" scenarios for low-similarity transcripts.
- **[x] Conditional Logic**:
    - Test `if/else` execution paths using mocked boolean flags.
    - Test flag updates (`set_flag`) and their persistence across subsequent matching events.
- **[x] Dispatcher Coordination**:
    - Mock the STT result queue and verify that matched events correctly produce `MacroCmd` sequences in the injection queue.
    - Verify that `SttResult` is still forwarded to the output (JSONL) thread regardless of match success.

### 2. Manual Verification (Live Demo)

- **[ ] HD2 Demo**:
    - Load a mini-pack with "Eagle Airstrike" (Up, Right, Down, Down, Down).
    - Speak the phrase into the mic and verify the sequence fires in a text editor or Helldivers 2.
- **[ ] Sound Feedback**:
    - Bind a sound clip to a macro.
    - Verify the sound plays audibly when the phrase is recognized.
- **[ ] Confidence Threshold**:
    - Intentionally mispronounce a phrase and verify it only fires if it meets the configured threshold.

### 3. Latency Check

- **[ ] STT-04 Extension**:
    - Measure time from `SttResult` availability to `MacroCmd` emission.
    - Target: Dispatch overhead < 10ms (matching and logic should be near-instant compared to STT).

---
*Generated via /gsd-plan-phase*
