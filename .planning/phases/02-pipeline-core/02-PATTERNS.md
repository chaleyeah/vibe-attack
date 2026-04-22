# Phase 2: Pipeline Core - Pattern Map

**Mapped:** 2026-04-22  
**Files analyzed:** 14 (new + modified; inferred from `02-CONTEXT.md` + `02-RESEARCH.md`)  
**Analogs found:** 10 / 14

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---:|---|---|
| `src/main.rs` | orchestrator/entrypoint | request-response + event-driven | `src/main.rs` | exact |
| `src/config.rs` | config/schema | file-I/O + transform | `src/config.rs` | exact |
| `src/lib.rs` | module index | n/a | `src/lib.rs` | exact |
| `src/pipeline/mod.rs` | orchestrator/service | streaming + event-driven | `src/main.rs` | role-match |
| `src/pipeline/timing.rs` | utility | transform | *(none)* | none |
| `src/pipeline/jsonl.rs` | utility/IO | request-response (stdout) | *(none)* | none |
| `src/vad/mod.rs` | service | streaming + transform | `src/input/ptt.rs` | partial (thread loop shape only) |
| `src/stt/mod.rs` | service | batch/transform | `src/input/inject.rs` | partial (worker thread + queue only) |
| `src/wake/mod.rs` | service | streaming + event-driven | `src/input/ptt.rs` | partial (thread loop shape only) |
| `tests/drop_oldest_queue.rs` | test | transform | `tests/*` | role-match |
| `tests/jsonl_schema.rs` | test | transform | `tests/config_parse.rs` | role-match |
| `tests/stt_smoke.rs` | test (integration, gated) | batch | `tests/macro_inject.rs` | role-match |
| `tests/wake_word.rs` | test (integration, gated) | streaming | `tests/macro_inject.rs` | role-match |
| `Cargo.toml` | config/deps | n/a | `Cargo.toml` | exact (existing style) |

## Pattern Assignments

### `src/main.rs` (entrypoint, event-driven orchestration)

**Analog:** `src/main.rs`

**Fail-fast preflight + “print actionable error to stderr”** (lines 49-86):

```rust
// Load config (fail-hard on any error)
let config = hd_linux_voice::config::load(args.config.as_deref()).map_err(|e| {
    eprintln!("{e:#}");
    e
})?;

// Parse PTT key code
let ptt_key = hd_linux_voice::input::ptt::parse_key_code(&config.ptt.key)
    .map_err(|e| { eprintln!("{e:#}"); e })?;

// Preflight: verify /dev/input readable (fail-hard)
hd_linux_voice::input::ptt::check_input_readable().map_err(|e| {
    eprintln!("{e:#}");
    e
})?;
```

**Threading convention (“long-lived work uses `std::thread::spawn`”) + cancellation** (lines 87-110, 143-153):

```rust
let ptt_active = Arc::new(AtomicBool::new(false));
let shutdown = CancellationToken::new();

// Spawn long-lived threads on OS threads
let (macro_tx, macro_rx) = mpsc::channel::<hd_linux_voice::input::inject::MacroCmd>();
let inject_handle = hd_linux_voice::input::inject::spawn_injection_thread(virtual_kbd, macro_rx);

let ptt_handle = hd_linux_voice::input::ptt::spawn_ptt_thread(
    ptt_device,
    ptt_key,
    Arc::clone(&ptt_active),
    shutdown.clone(),
);

// ...
shutdown.cancel();
let _ = macro_tx.send(hd_linux_voice::input::inject::MacroCmd::Shutdown);
let _ = inject_handle.join();
```

**SIGTERM/SIGINT wait loop using Tokio only for signals** (lines 130-141):

```rust
let mut sigterm = tokio::signal::unix::signal(
    tokio::signal::unix::SignalKind::terminate()
)?;
let mut sigint = tokio::signal::unix::signal(
    tokio::signal::unix::SignalKind::interrupt()
)?;

tokio::select! {
    _ = sigterm.recv() => tracing::info!("SIGTERM received"),
    _ = sigint.recv()  => tracing::info!("SIGINT received (Ctrl+C)"),
}
```

**How Phase 2 should copy this:**
- Keep **Tokio usage limited** to signal handling + lightweight coordination; keep VAD/STT/wake work on OS threads.
- Keep “stdout clean” by using `tracing` (stderr) for status and writing JSONL to stdout explicitly (new pattern; see `src/pipeline/jsonl.rs` section).

---

### `src/config.rs` (config schema extensions for VAD/STT/wake)

**Analog:** `src/config.rs`, `tests/config_parse.rs`

**Strict schema and typed structs** (lines 5-14 and repeated on nested structs):

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub ptt: PttConfig,
    pub timing: TimingConfig,
    #[serde(default)]
    pub macros: Vec<MacroConfig>,
}
```

**Load path: XDG default + contextual errors** (lines 63-94):

```rust
pub fn default_config_path() -> Result<PathBuf> {
    let xdg = xdg::BaseDirectories::with_prefix("hd-linux-voice");
    xdg.place_config_file("config.yaml")
        .context("Failed to create XDG config directory for hd-linux-voice")
}

pub fn load(path_override: Option<&std::path::Path>) -> Result<Config> {
    // ...
    let file = std::fs::File::open(&path).with_context(|| {
        format!(
            "Config file not found: {}\n\
             Create it at that path. See config.example.yaml for the format.",
            path.display()
        )
    })?;

    let config: Config = serde_yaml_ng::from_reader(file).with_context(|| {
        format!("Failed to parse config file: {}", path.display())
    })?;

    Ok(config)
}
```

**Test pattern for “deny unknown fields” and “wrong type returns Err”** (`tests/config_parse.rs` lines 26-53):

```rust
#[test]
fn config_rejects_unknown_fields() {
    // ...
    let result = hd_linux_voice::config::load(Some(f.path()));
    assert!(result.is_err(), "unknown_field must cause an error");
}

#[test]
fn config_wrong_type_returns_err_not_panic() {
    // ...
    let result = hd_linux_voice::config::load(Some(f.path()));
    assert!(result.is_err(), "wrong type must fail with Err");
}
```

**How Phase 2 should copy this:**
- Add `vad`, `stt`, `wake`, and `pipeline` sections as new structs under `Config`, each with `#[serde(deny_unknown_fields)]`.
- Keep load-time validation **fail-fast** and return contextual `anyhow::Result` errors (no `unwrap` in load path).

---

### `src/pipeline/mod.rs` (pipeline coordinator + state machine)

**Analog:** `src/main.rs` (orchestration), `src/input/ptt.rs` (thread loop shape), `src/audio/mod.rs` (ringbuffer drain constraints)

**Main “orchestrate resources before spawning threads”** (from `src/main.rs` lines 49-79):
- Create/validate config + model paths **before** spawning VAD/STT worker threads.
- Create bounded channels/queues before threads.

**Cancellation loop shape** (from `src/input/ptt.rs` lines 116-138):

```rust
loop {
    if shutdown.is_cancelled() {
        break;
    }
    match device.fetch_events() {
        Ok(events) => { /* process batch */ }
        Err(e) => { /* log + exit */ break; }
    }
}
```

**How Phase 2 should copy this:**
- VAD/wake thread loop should check `CancellationToken` regularly (between “drain ringbuf” batches).
- Keep “poll + short sleep” pacing (Decision D-05) in the drain loop (no busy spin).
- Do not run anything heavy in the CPAL callback (reinforced by `src/audio/mod.rs`).

---

### `src/pipeline/jsonl.rs` (stdout JSONL transcript/events)

**Analog:** *(none in codebase yet; closest is logging to stderr via `tracing` and `eprintln!` in `src/main.rs`.)*

**Existing stderr pattern to preserve** (from `src/main.rs` lines 49-53):

```rust
eprintln!("{e:#}");
```

**Guidance for the new module (no existing analog):**
- Implement a `Write`-to-stdout helper that writes **one JSON object per line**.
- Keep diagnostics/instrumentation on stderr (`tracing` or `eprintln!`), never stdout.
- Prefer `serde` + `serde_json` to avoid manual escaping (Phase decision D-19/D-20).

---

### `src/pipeline/timing.rs` (monotonic + wall-clock timestamps)

**Analog:** *(none; no existing timing utilities in `src/`.)*

**Guidance for the new module (no existing analog):**
- Use `std::time::Instant` for monotonic durations (stage latency fields).
- Use `std::time::SystemTime` for wall-clock timestamps (RFC3339 or unix ms) as required by D-21.
- Capture timings **at the stage boundaries** in the thread that performs the work (per `02-RESEARCH.md` responsibility map).

---

### `src/vad/mod.rs` (Silero VAD wrapper + utterance segmentation)

**Analog:** `src/input/ptt.rs` (long-lived worker loop + cancellation), `src/audio/mod.rs` (streaming buffer constraints)

**Worker thread loop + cancellation** (from `src/input/ptt.rs` lines 116-138; see excerpt above).

**RT constraint reminder + ringbuffer ownership model** (from `src/audio/mod.rs` lines 24-33):

```rust
pub struct AudioHandle {
    _stream: cpal::Stream,
    pub consumer: HeapCons<f32>,
    pub actual_config: StreamConfig,
}
```

**How Phase 2 should copy this:**
- Drain `AudioHandle.consumer` on the VAD/wake OS thread; do not touch the producer/callback path.
- Use a rolling pre-roll buffer and tail buffer (D-06/D-07) in the VAD segmentation logic.

---

### `src/stt/mod.rs` (whisper-rs wrapper + dedicated STT thread)

**Analog:** `src/input/inject.rs` (channel-driven worker thread) and `src/main.rs` (never do heavy work on Tokio)

**Channel-driven long-lived worker** (from `src/input/inject.rs` lines 171-220):

```rust
pub fn spawn_injection_thread(
    mut device: evdev::uinput::VirtualDevice,
    rx: mpsc::Receiver<MacroCmd>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        loop {
            match rx.recv() {
                Ok(MacroCmd::Execute { .. }) => { /* work */ }
                Ok(MacroCmd::Shutdown) => break,
                Err(_) => break,
            }
        }
    })
}
```

**How Phase 2 should copy this:**
- The STT thread should be a **dedicated `std::thread`** that blocks on a bounded queue (research recommends `crossbeam-channel`; Decision D-02).
- Add an explicit “Shutdown” path to stop cleanly (either via `CancellationToken` or a control message).

---

### `src/wake/mod.rs` (wake word keyword spotter)

**Analog:** `src/input/ptt.rs` (loop + cancellation + event-driven toggles)

Use the same cancellation/loop structure as `spawn_ptt_thread`, but replace `fetch_events()` with “drain audio → feed keyword spotter → emit trigger event”.

---

### `src/lib.rs` (module exports)

**Analog:** `src/lib.rs` lines 1-5:

```rust
pub mod audio;
pub mod config;
pub mod error;
pub mod input;
```

**How Phase 2 should copy this:**
- Add `pub mod pipeline; pub mod vad; pub mod stt; pub mod wake;` following the existing simple module-export style.

---

### `tests/drop_oldest_queue.rs` (unit test: bounded drop-oldest semantics)

**Analog:** existing unit-style tests in `src/audio/mod.rs` (lines 138-191) and integration tests in `tests/config_parse.rs`.

**Pattern to copy (small focused invariants)** (from `src/audio/mod.rs` lines 183-190):

```rust
#[test]
fn ring_buffer_overflow_does_not_panic() {
    let rb = HeapRb::<f32>::new(4);
    let (mut producer, _consumer) = rb.split();
    let data = [0.1f32; 8];
    let _ = producer.push_slice(&data); // must not panic
}
```

---

### `tests/jsonl_schema.rs` (unit test: JSONL schema stability)

**Analog:** `tests/config_parse.rs` (table-driven YAML strings + `assert!(is_err())` semantics).

Pattern to copy: build an event struct, encode with `serde_json`, assert required fields exist and types match; keep it pure (no filesystem, no threads).

---

### `tests/stt_smoke.rs` and `tests/wake_word.rs` (integration, env-gated, heavy deps)

**Analog:** `tests/macro_inject.rs` pattern for privileged / ignored integration tests (not loaded here; treat as “gated integration test exists in `tests/`”).

Use `#[ignore = "..."]` and/or env vars (as suggested in `02-RESEARCH.md` “Wave 0 gaps”) so CI and dev boxes without models/libs can still run `cargo test`.

## Shared Patterns

### OS thread policy (no long-lived work on Tokio)

**Sources:** `src/input/ptt.rs` lines 103-110, `src/input/inject.rs` lines 171-178

```rust
// spawn_blocking is for short-duration tasks; the PTT loop is long-lived.
// Use std::thread::spawn, NOT tokio::task::spawn_blocking.
```

### RT audio callback invariants (no alloc / no blocking)

**Source:** `src/audio/mod.rs` lines 108-121

```rust
move |data: &[f32], _info: &cpal::InputCallbackInfo| {
    // INVARIANT: this closure must never allocate or block.
    if ptt_active.load(Ordering::Relaxed) {
        // ... push into pre-allocated ring buffer ...
    }
}
```

### Config strictness (deny unknown fields)

**Source:** `src/config.rs` lines 7-14 and `tests/config_parse.rs` lines 26-39

```rust
#[serde(deny_unknown_fields)]
pub struct Config { /* ... */ }
```

## No Analog Found

Files with no close match in the codebase (planner should follow `02-RESEARCH.md` and standard Rust patterns):

| File | Role | Data Flow | Reason |
|---|---|---|---|
| `src/pipeline/jsonl.rs` | utility/IO | request-response (stdout) | No existing stdout JSONL writer; existing output is logs/errors to stderr. |
| `src/pipeline/timing.rs` | utility | transform | No shared timing utilities in `src/` yet. |
| `src/pipeline/mod.rs` | orchestrator/service | streaming + event-driven | Only `main.rs` exists for orchestration; pipeline state machine is new. |
| `src/vad/mod.rs` | service | streaming + transform | VAD-specific buffering/segmentation is new; only thread loop patterns exist. |

## Metadata

**Analog search scope:** `src/**/*.rs`, `tests/**/*.rs`  
**Existing anchors read:** `src/main.rs`, `src/config.rs`, `src/audio/mod.rs`, `src/error.rs`, `src/input/inject.rs`, `src/input/ptt.rs`, `tests/config_parse.rs`  
**Pattern extraction date:** 2026-04-22

