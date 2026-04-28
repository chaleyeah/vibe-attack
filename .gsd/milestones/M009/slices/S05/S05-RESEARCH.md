# S05: TriggerMacro control request + editor Test button — Research

**Date:** 2026-04-27

## Summary

The control plane uses Unix Domain Sockets with newline-delimited JSON (serde_json). `ControlRequest` is tagged with `#[serde(tag = "cmd", content = "args", rename_all = "snake_case")]`. A `TestMacro { name: String }` variant **already exists** in `protocol.rs` but falls into the `_ => ControlResponse::Error { message: "Not yet implemented" }` catch-all in `control/mod.rs`. S05's first task is to implement that handler, not to add a new protocol variant. The name `TestMacro` fits the slice purpose exactly — no rename needed.

The `Dispatcher` holds the live macro registry in `Arc<RwLock<Vec<MacroConfig>>>`. Its `process()` method is the only current fire path: it matches transcripts, then sends a `MacroCmd::Execute { keys, ... }` over the `macro_tx: Sender<MacroCmd>` channel. There is no separate `fire_by_name` method. S05 must add one — a `fire_named(&self, name: &str) -> Result<DispatchOutcome, String>` method (or equivalent) that skips phrase matching and directly executes the `MacroCmd::Execute` for the named macro. The `macro_tx` sender is already held by `Dispatcher`; the new method just reads the registry, looks up by name, and sends.

The editor binary (`src/bin/vibe-attack-config.rs`) already uses `vibe_attack::control::client::send_command(ControlRequest::...)` to dispatch `SetMode`, `SetThreshold`, etc. The pattern for a Test button is identical: on button click, send `ControlRequest::TestMacro { name }` via `send_command`. The 1-second confirmation prompt must be implemented as UI-side state (a `pending_test: Option<(String, std::time::Instant)>` field on the app struct), not as a sleep — the eframe render loop checks each frame whether the deadline has passed before actually sending the command.

## Recommendation

Implement in this order: (1) add `fire_named` to `Dispatcher` — pure logic, testable in isolation; (2) wire `TestMacro` handler in `control/mod.rs` using `tokio::task::block_in_place` (matching the `SwitchProfile` pattern) to call `dispatcher.fire_named`; (3) add confirmation-pending state to `VibeAttackConfigApp` and render the Test button + 1-second countdown in `show_main_config`; (4) wire the confirmed send to `send_command(ControlRequest::TestMacro { name })`.

Do NOT rename `TestMacro` to `TriggerMacro`. The existing variant is correct; the boundary map's mention of `TriggerMacro { category, name }` is aspirational — the actual protocol already has `TestMacro { name }` and `name` is sufficient since macro names are unique within a profile (dispatcher matches by name across the flattened list). Adding `category` is unnecessary complexity.

## Implementation Landscape

### Key Files

- `src/control/protocol.rs` — `TestMacro { name: String }` already declared at line 35; no changes needed here unless we later want `category` disambiguation.
- `src/control/mod.rs` — `TestMacro` arm currently hits `_ =>` catch-all (line 187). Handler goes here, calling `tokio::task::block_in_place(|| dispatcher.fire_named(&name, handle))` mirroring the `SwitchProfile` arm (lines 156–163).
- `src/pipeline/dispatcher.rs` — add `pub fn fire_named(&self, name: &str) -> Result<DispatchOutcome, String>`. Look up by name in `self.macros.read()`, build `KeyStep` vec, send `MacroCmd::Execute` over `self.macro_tx`. Return `Fired` or an error string if not found. Sound playback should be included for consistency with `process()`.
- `src/bin/vibe-attack-config.rs` — add `pending_test: Option<(String, std::time::Instant)>` to `VibeAttackConfigApp`. In `show_main_config`, render a "Test" button next to each macro in the profile list (currently profiles are shown as `selectable_label`; this will need a horizontal layout per row). On click, set `pending_test`. Each frame: if `pending_test` is set and elapsed >= 1s, call `send_command(ControlRequest::TestMacro { name })` and clear it. Show countdown in the status bar.
- `src/ui/config_app.rs` — no structural changes needed; `ConfigApp` does not need new fields (test state belongs in the egui app struct, not the pure-logic layer).

### Build Order

1. **`Dispatcher::fire_named`** — add and unit-test with the existing `make_dispatcher` harness. This is the only code path that touches uinput-adjacent logic; prove it compiles and routes correctly before wiring the socket handler.
2. **`TestMacro` handler in `control/mod.rs`** — implement and verify with `cargo test` (default features). Integration test: start daemon in test mode, send `TestMacro` over socket, assert `DispatchOutcome::Fired` (or `Ok` response).
3. **UI confirmation state and Test button** — add to `vibe-attack-config.rs` under `cfg(feature = "gui")`. Verify `cargo build --features gui` compiles clean and `cargo clippy --features gui -D warnings` passes.
4. **JSONL observation** — run daemon, click Test button, confirm `{"event":"macro_fired","macro_id":"..."}` appears in dispatcher output.

### Verification Approach

```
# Default features (no gui) must be clean
cargo clippy -D warnings
cargo test

# GUI feature must be clean
cargo clippy --features gui -D warnings
cargo build --features gui

# Integration smoke test (manual)
# 1. cargo run           (start daemon with a profile loaded)
# 2. cargo run --features gui --bin vibe-attack-config
# 3. Select a macro, click Test, observe 1-second countdown
# 4. After countdown: daemon log shows "Firing macro name=<n>" and uinput event fires
# 5. JSONL dispatcher output shows Fired event
```

Unit test target: add `test_fire_named_found` and `test_fire_named_not_found` to `dispatcher.rs` tests using the existing `make_dispatcher` helper. The `MacroCmd` receiver can be checked to confirm `Execute` was sent.

## Constraints

- **No direct uinput from the editor binary** — `vibe-attack-config` MUST only send a socket command; all key injection stays in the daemon's `macro_tx` pipeline.
- **GUI feature boundary** — Test button code lives in `src/bin/vibe-attack-config.rs` only; `src/ui/config_app.rs` stays feature-agnostic. `cargo build` (default features) must not see any eframe/egui types.
- **`tokio::task::block_in_place`** — the control socket handler is async (Tokio); `fire_named` will hold an `RwLock` read guard (sync). Must use `block_in_place` as `SwitchProfile` does, not `.await` a blocking call.
- **1-second confirmation is non-negotiable** — accidental macro fire in a live game session could waste a stratagem. The delay is a hard safety requirement.
- **`cargo clippy -D warnings` clean** — both default and `gui` feature sets. Confirm `pending_test` unused-variable warnings don't slip through on non-gui builds.

## Common Pitfalls

- **Adding `category` to `TestMacro`** — unnecessary and breaks backward compat with any existing CLI usage of `TestMacro { name }`. `Dispatcher` already flattens all macros; name lookup is O(n) over the flat list and sufficient. Avoid scope creep.
- **Blocking the async executor** — calling `dispatcher.fire_named` directly inside the `async move` block (without `block_in_place`) will deadlock if the `RwLock` is ever held across a yield point elsewhere. Match the `SwitchProfile` pattern exactly.
- **Confirmation timer as `std::thread::sleep`** — never sleep the UI thread. Use an `Instant` stored in app state; check elapsed each frame in the eframe render loop.
- **Missing `Ok` response on `TestMacro`** — the handler must send `ControlResponse::Ok` back on success (or `Error`) before the socket closes; the client (`send_command`) blocks on reading that response line. Forgetting it causes a hang in the editor.
- **`fire_named` returning `DispatchOutcome` vs `bool`** — return `DispatchOutcome` (same as `process`) so callers can log which macro fired at what score (implicit 1.0 for direct triggers). This keeps the JSONL output format consistent.
- **Profile not loaded** — if the daemon has no active profile, `fire_named` will find an empty registry and return an error. The UI should show daemon status before enabling the Test button; grey it out when `daemon_running` is false.
