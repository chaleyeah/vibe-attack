---
phase: M008
phase_name: UI / Tray Runtime Control
project: hd-linux-voice
generated: "2026-04-27T22:11:00Z"
counts:
  decisions: 8
  lessons: 6
  patterns: 7
  surprises: 2
missing_artifacts: []
---

### Decisions

- **RwLock<PhraseMatcher> for live threshold.** Chose RwLock<PhraseMatcher> over Arc<AtomicF32> because threshold is consumed by PhraseMatcher::new(); replacing the full matcher under a write lock is cleaner than maintaining two separate atomics with a consistency risk.
  Source: S01-SUMMARY.md/Key decisions

- **ActivationMode runtime-only in M008 (not persisted to config.yaml).** Deferred YAML schema change to a future milestone; SetMode is sent over the control socket on Save instead. Mode reverts to config default after daemon restart.
  Source: S02-SUMMARY.md/Key decisions

- **threshold_pct uses u8 (0–100) integer domain in ConfigApp.** Prevents float drift in the UI layer; round+clamp conversion happens only at I/O boundaries (load_config_into_app / save_app_to_config).
  Source: S02-SUMMARY.md/Key decisions

- **Arc<RwLock<ActivationMode>> on DaemonHandle.** ActivationMode is not atomically encodable; Arc<RwLock<T>> matches the existing active_profile pattern. SetMode handler writes active_mode to handle BEFORE forwarding RuntimeCommand to coordinator so Status responses are always coherent with the last SetMode.
  Source: S03-SUMMARY.md/Key decisions

- **icon_name_for_state extracted as free pub(crate) function.** Enables unit tests without requiring a live D-Bus session or ksni instantiation — the free function takes Option<&DaemonState> and returns a &'static str, fully testable in headless CI.
  Source: S03-SUMMARY.md/Key decisions

- **Tray menu activate closures use std::thread::spawn fire-and-forget.** ksni D-Bus callbacks must not block. All tray menu activate closures follow the pattern: spawn → send_command → discard result.
  Source: S03-SUMMARY.md/Patterns established

- **SocketGuard uses place_runtime_file (server-side path) not find_runtime_file.** In integration tests, the cleanup path must match the bound socket path; place_runtime_file is the server-side path-generation function while find_runtime_file is the client-side lookup, which may differ.
  Source: S04-SUMMARY.md/Key decisions

- **SetThreshold integration test asserts on the RuntimeCommand channel, not dispatcher.threshold().** The coordinator drain loop applies the threshold asynchronously on the next audio frame; asserting the dispatcher value directly would introduce a race condition. Asserting on the channel that the command was received is the correct boundary.
  Source: S04-SUMMARY.md/Key decisions

---

### Lessons

- **clippy not available at system level (rustup not installed).** cargo build was substituted throughout S02 and S04 for lint verification. No new warnings were introduced, but the automated clippy gate was not enforced. Install clippy via rustup for proper lint gating in CI.
  Source: S02-SUMMARY.md/Known limitations

- **ActivationMode changes are runtime-only and not persisted to config.yaml.** Mode reverts to config default after daemon restart. This was an explicit scope decision for M008, but future milestones should consider write-back so mode survives restarts.
  Source: S04-SUMMARY.md/Known limitations

- **Integration tests that bind UDS sockets need #[serial] + a Drop-based SocketGuard.** Without #[serial], parallel test execution causes bind conflicts on the same socket path. The SocketGuard Drop impl ensures deterministic cleanup even on panic, and both tests skip gracefully when XDG_RUNTIME_DIR is absent (bare CI environments).
  Source: S04-SUMMARY.md/Patterns established

- **XDG_CONFIG_HOME override + serial_test::serial required for config file tests.** Without path isolation, concurrent config tests race on the same file. This is a project-wide convention established in S02.
  Source: S02-SUMMARY.md/Patterns established

- **Daemon-absent graceful degradation: always write to disk, only send socket commands when daemon_running=true.** Always leaving the user with an actionable status message prevents silent failure. The pattern: write YAML regardless, skip socket commands if daemon_running is false, set status_message so the UI shows feedback.
  Source: S02-SUMMARY.md/Patterns established

- **UAT assertions should reference exact log strings from source code.** Using approximate descriptions in UAT steps leads to ambiguity during manual testing. Exact grep-able strings (e.g. "runtime_command_applied") let testers unambiguously confirm the right log line fired.
  Source: S04-SUMMARY.md/Key decisions

---

### Patterns

- **try_recv drain loop at top of coordinator frame.** Non-blocking ~50/s drain processes all queued RuntimeCommands before audio work each frame. Ensures mode/threshold changes take effect between utterances without blocking the audio pipeline.
  Source: S01-SUMMARY.md/Patterns established

- **send_runtime_cmd helper on DaemonHandle.** Option check + SendError → ControlResponse::Error{pipeline not running}. Centralises the "is the channel live?" guard so callers don't need to handle Option<Sender> directly.
  Source: S01-SUMMARY.md/Patterns established

- **update_threshold() pattern: clamp → write-lock → replace matcher → emit tracing::info(old, new).** Three-step pattern for live threshold updates. Clamp before lock acquisition; replace entire PhraseMatcher under the write lock; log old and new values for observability.
  Source: S01-SUMMARY.md/Patterns established

- **ConfigApp I/O helpers: load returns full Config for caching; save takes cached Config as base to clone+mutate.** Preserves all non-UI fields (macros, pack metadata, etc.) across save cycles. Per-frame I/O is avoided by populating cached_config and device_names once at startup.
  Source: S02-SUMMARY.md/Patterns established

- **Free pub(crate) functions for icon/state mappings enable headless unit tests without D-Bus or ksni.** Extracting icon_name_for_state as a free function (not a method on a ksni struct) makes the mapping fully testable in CI with no display server.
  Source: S03-SUMMARY.md/Patterns established

- **TrayState optional fields use Option<T> for daemon-not-running sentinel.** Option<ActivationMode> cleanly represents "daemon not running" without a sentinel value; items are disabled via the daemon_running flag rather than dummy values.
  Source: S03-SUMMARY.md/Patterns established

- **make_handle_with_runtime_tx() helper for integration tests.** Mirrors make_handle from control_protocol.rs but adds .with_runtime_tx() so SetMode/SetThreshold commands succeed in integration tests. Keeps test setup DRY and explicit about which tests require a live RuntimeCommand channel.
  Source: S04-SUMMARY.md/Patterns established

---

### Surprises

- **cargo build substituted for cargo clippy throughout due to missing rustup/clippy at system level.** This was discovered mid-S02 and propagated as a known limitation through S04. The substitution is safe (build catches type errors) but does not enforce lint cleanliness. Not a blocker, but unexpected for a Rust project.
  Source: S02-SUMMARY.md/Known limitations

- **SetThreshold integration test cannot assert dispatcher.threshold() directly.** Initial approach would assert the dispatcher's threshold field after sending SetThreshold over the socket, but the coordinator drain loop is asynchronous — the value is applied on the next audio frame tick, not synchronously in the handler. Asserting on the RuntimeCommand channel (that the command was received) is the correct boundary for a handler-level test.
  Source: S04-SUMMARY.md/Key decisions
