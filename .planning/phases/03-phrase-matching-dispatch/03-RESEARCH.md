# Phase 3 Research: Phrase Matching + Dispatch

## 1. Fuzzy Phrase Matching (STT-02)

### Crate Selection: `strsim`
- **Choice**: [`strsim`](https://crates.io/crates/strsim)
- **Rationale**: It is the industry standard in the Rust ecosystem for string similarity metrics. It implements Levenshtein, Jaro, Jaro-Winkler, and others.
- **License**: MIT (AGPL-3.0 compatible).
- **Implementation**:
    ```rust
    use strsim::levenshtein;
    let distance = levenshtein("eagle airstrike", "eagal air strike");
    let similarity = 1.0 - (distance as f64 / transcript.len().max(target.len()) as f64);
    ```

## 2. Sound Feedback (MCRO-04)

### Backend: `rodio`
- **Choice**: [`rodio`](https://crates.io/crates/rodio)
- **Rationale**: High-level, cross-platform, and handles decoding (WAV, MP3, Ogg) and mixing.
- **CPAL Coexistence**: `rodio` uses `cpal` internally for output. The current app uses `cpal` for input. Modern audio servers (PipeWire/PulseAudio) handle simultaneous input and output streams from the same process without issue.
- **Latency**: `rodio` is generally low-latency enough for feedback sounds. We should use `Decoder::new_looped` or pre-load sounds into memory (e.g., `StaticSource`) to avoid disk IO latency during trigger.

## 3. Dispatcher Thread & Boolean Logic (MCRO-03)

### Thread Topology
The Dispatcher will be a dedicated OS thread spawned in `coordinator.rs`.
- **Inputs**: `Receiver<SttResult>` from the STT service.
- **Outputs**: 
    - `Sender<MacroCmd>` to the input injection thread.
    - `Sender<SttResult>` to the output (JSONL) thread.
- **Shared State**: `Arc<RwLock<HashMap<String, bool>>>` for boolean flags.

### Boolean Flags Data Structure
- Use `dashmap` or a simple `Arc<RwLock<HashMap<String, bool>>>`. Given the low frequency of updates and small number of flags, `RwLock<HashMap>` is sufficient and keeps dependencies light.

## 4. Validation Architecture (Nyquist Dimension 8)

To verify the dispatch logic without requiring a live game session during CI:
- **Mock Input**: A test harness that injects known transcripts into the Dispatcher's input queue.
- **Mock Output**: A way to intercept `MacroCmd` events and verify they match the expected sequence for a given phrase.
- **Sound Check**: Verify `rodio` sinks are created/played (can be mocked or verified via logs).

### Test Matrix
| Transcript | Configured Phrase | Threshold | Expected Action |
|------------|-------------------|-----------|-----------------|
| "eagle airstrike" | "Eagle Airstrike" | 0.8 | Fire Macro |
| "eagal air strike" | "Eagle Airstrike" | 0.8 | Fire Macro (Similarity ~0.9) |
| "hello world" | "Eagle Airstrike" | 0.8 | NO_MATCH |
| "Eagle Airstrike" (if flag=true) | "Eagle Airstrike" | 0.8 | Fire Macro |
| "Eagle Airstrike" (if flag=false) | "Eagle Airstrike" | 0.8 | Ignore (Condition fail) |

## 5. Integration Plan
1. Add `strsim` and `rodio` to `Cargo.toml`.
2. Update `Config` struct to include `macros` with `conditions` and `sound` fields.
3. Implement `Dispatcher` in `src/pipeline/dispatcher.rs`.
4. Update `coordinator.rs` to wire the `Dispatcher` between `SttService` and `JsonlWriter`.
5. Pass `macro_tx` into the `Dispatcher`.
