---
phase: M001
phase_name: Migration
project: hd-linux-voice
generated: "2026-04-25T20:30:00Z"
counts:
  decisions: 8
  lessons: 8
  patterns: 6
  surprises: 4
missing_artifacts: []
---

### Decisions

- **Shared ORT linking for sherpa-onnx** (`default-features = false, features = ["shared"]`): chose shared libonnxruntime.so over static embedding to eliminate the dual-ORT heap corruption that caused `std::bad_alloc` when both wake-word (sherpa-onnx) and VAD (silero-vad-rust via ort crate) ran in the same process. Alternative was keeping them as separate processes (IPC overhead) or disabling one permanently.
  Source: S07-SUMMARY.md/What Happened

- **ORT_DYLIB_PATH auto-discovery before VAD init**: chose `unsafe std::env::set_var` at the single-threaded coordinator call site (before any pipeline threads spawn) rather than requiring users to set the env var manually. The single-threaded invariant is documented inline.
  Source: S07-SUMMARY.md/What Happened

- **Feature-gated GUI binary via `required-features = ["gui"]`**: chose this over `cfg(feature = "gui")` guards scattered through source — keeps the default daemon build headless without any conditional compilation in library code.
  Source: S05-SUMMARY.md/key_decisions

- **Dispatcher injected writer (`Box<dyn Write + Send>`)**: chose dependency injection over cfg-test branching or global stdout capture so that JSONL output tests work with `Vec<u8>` and production uses stdout — same code path in both cases.
  Source: S03-SUMMARY.md/patterns_established

- **Stdout exclusively for JSONL; stderr for all tracing**: chose a dedicated output thread writing only `#[serde(tag = "type")]` events to stdout, making the daemon composable with CLI tooling via pipe. All `tracing` instrumentation goes to stderr.
  Source: S02-SUMMARY.md/key_decisions

- **Documentation TDD (tests first, docs second)**: wrote `tests/documentation.rs` with 11 failing tests asserting file existence and section headings, then created the four doc files to satisfy them. Same portable `env!("CARGO_MANIFEST_DIR")` + `to_lowercase().contains()` pattern as UI tests.
  Source: S06-SUMMARY.md/patterns_established

- **tokio-util without features**: `CancellationToken` in `tokio_util::sync` is unconditionally compiled in tokio-util 0.7; the `sync` named feature does not exist and causes a build error.
  Source: S01-SUMMARY.md/Deviations

- **serde_yaml_ng over serde_yaml**: enforced per RESEARCH.md — serde_yaml deprecated March 2024 with unresolved libyaml CVE; serde_yaml_ng is the maintained fork.
  Source: S01-SUMMARY.md/Verification Results

### Lessons

- **CPAL 0.17 breaking API changes**: `SampleRate` is a `u32` type alias (not a tuple struct — `SampleRate(16_000)` is invalid); `device.name()` deprecated in favor of `device.description().name()`; ringbuf 0.4 renamed `push()` to `try_push()` and requires explicit `Consumer`/`Producer`/`Split` trait imports (not re-exported through `use super::*`).
  Source: S01-SUMMARY.md/Deviations from Plan

- **xdg 3.0 path construction**: `BaseDirectories::with_prefix("app")` returns `BaseDirectories` directly (not `Result` — no `.context()` needed), and appends the prefix to `config_home`, so test fixtures must use `dir.path().join("hd-linux-voice/profiles")` not `dir.path().join("profiles")`.
  Source: S04-SUMMARY.md/What Happened

- **evdev 0.13 deprecated `VirtualDeviceBuilder::new()`**: use `VirtualDevice::builder()` instead; the old constructor is deprecated and will be removed in a future version.
  Source: S01-SUMMARY.md/Deviations

- **cargo-about 0.8.4 data model change**: templates must iterate `{{#each crates}}` with `package.name`/`license` fields; the old `{{#each overview}}` with `crate.name`/`license.id`/`license.text` fields no longer exists. `[targets].include` is invalid — use `targets = [...]` array. Combined SPDX expressions (e.g. `"MIT OR Apache-2.0"`) in `accepted` list are invalid — list individual SPDX IDs.
  Source: S01-SUMMARY.md/Deviations

- **Auto-mode shell approval blocks cargo test/build**: S04–S07 could not run compiled tests in auto-mode. Static verification (grep, source inspection, cargo registry API cross-reference) was substituted. This means runtime test correctness for those slices is unconfirmed until CI runs — treat static-only verified slices as "high confidence" not "validated".
  Source: S04-SUMMARY.md/Known Limitations, S05-SUMMARY.md/Known Limitations, S06-SUMMARY.md/Known Limitations, S07-SUMMARY.md/Known Limitations

- **Dual ORT bad_alloc root cause**: sherpa-onnx statically embedded ~218 ORT symbols AND the `ort` crate (via silero-vad-rust) dynamically loaded its own ORT instance in the same process — two global runtime environments colliding on the heap. Fix: `features = ["shared"]` on sherpa-onnx-sys makes it link against the shared `libonnxruntime.so` used by the ort crate.
  Source: S07-SUMMARY.md/What Happened

- **`unsafe std::env::set_var` is safe pre-spawn**: calling `set_var` before any threads are spawned is safe because env var mutation is only unsound under concurrent reads. The `coordinator.rs` ORT_DYLIB_PATH block is at exactly that single-threaded point. Document the invariant inline.
  Source: S07-SUMMARY.md/key_decisions

- **XDG env-var isolation pattern for hermetic tests**: set `XDG_CONFIG_HOME` to `tempdir.path()` before `BaseDirectories::with_prefix(...)` call, then `remove_var` immediately after. Failure to do this causes tests to write to the real user config directory.
  Source: S04-SUMMARY.md/patterns_established

### Patterns

- **Injected writer for JSONL capture in tests**: `Box<dyn Write + Send>` on the Dispatcher lets tests use `Vec<u8>` to capture JSONL output and assert on stable serialization, while production code passes stdout. No cfg-test branching required.
  Source: S03-SUMMARY.md/patterns_established

- **Env-gated + `#[ignore]` for heavy tests**: tests requiring real hardware (models, audio devices, uinput) are both `#[ignore]` and gated by an env var (e.g. `RUN_KWS_TESTS=1`, `RUN_PRIVILEGED_TESTS=1`). `#[ignore]` alone prevents accidental execution; env gate adds a hard stop with a clear error message.
  Source: S02-SUMMARY.md/patterns_established, S07-SUMMARY.md/patterns_established

- **Structural packaging/doc tests with `env!("CARGO_MANIFEST_DIR")`**: use `env!("CARGO_MANIFEST_DIR")` + `std::fs::read_to_string` + `to_lowercase().contains()` to assert doc file existence and required sections. Portable across any checkout path, no subprocess invocation.
  Source: S05-SUMMARY.md/patterns_established, S06-SUMMARY.md/patterns_established

- **Preflight checks before any thread spawn**: validate all external resources (config paths, model files, /dev/input readability, /dev/uinput openability) in the main thread before spawning long-lived pipeline threads. Errors surface with actionable messages before any background work starts.
  Source: S01-SUMMARY.md/What Was Built, S02-SUMMARY.md/patterns_established

- **`#[serde(tag = "type")]` for stable JSONL event schema**: use an internally-tagged enum for all pipeline JSONL events so the `type` field is always present and stable. Schema stability is enforced by fast, model-free serde_json tests in `tests/jsonl_schema.rs`.
  Source: S02-SUMMARY.md/patterns_established, S03-SUMMARY.md/patterns_established

- **ORT_DYLIB_PATH auto-discovery pattern**: in coordinator/pipeline init, check for existing `ORT_DYLIB_PATH` env var first (respect user overrides), then auto-set to `<exe_dir>/libonnxruntime.so` using `unsafe set_var` at the single-threaded call site. Log the auto-set path via `tracing::info!` for observability.
  Source: S07-SUMMARY.md/patterns_established

### Surprises

- **CPAL 0.17 made three breaking changes at once**: `SampleRate` type alias (not struct), deprecated `device.name()`, and ringbuf renamed `push()` to `try_push()` with required trait imports — all discovered during S01 GREEN compilation. The plan was written against pre-0.17 docs.
  Source: S01-SUMMARY.md/Deviations

- **cargo-about 0.8.4 changed both the config format and the template data model**: the `about.toml` `[targets].include` key and the `{{#each overview}}` template block both became invalid simultaneously. The generated output was silently empty (no error) until the template was rewritten to iterate `{{#each crates}}`.
  Source: S01-SUMMARY.md/Deviations

- **XDG path bug not caught until T03 static analysis (S04)**: `profile_manager_get_active_pack_resolves_from_profiles_dir` had the wrong fixture path (`profiles/` vs `hd-linux-voice/profiles/`) — a one-line bug that would have caused a silent test failure at runtime. Static analysis across multiple tasks caught it before any compiled run.
  Source: S04-SUMMARY.md/What Happened

- **Auto-mode shell approval policy blocked all runtime test confirmation for S04–S07**: this was not anticipated during planning and meant four slices completed with static-only verification. The policy is not new, but its interaction with the increasing size of the test suite (pack, UI, docs, wake-word) created a growing gap between "static verified" and "runtime confirmed".
  Source: S04-SUMMARY.md/Known Limitations, S05-SUMMARY.md/Known Limitations
