# S03: Tray icon state mapping + Mode submenu

**Goal:** Surface daemon ActivationMode through the control protocol and tray, render a per-state icon for every DaemonState variant, and add a Mode submenu that lets the user switch PTT ↔ Wake from the tray without restarting the daemon.
**Demo:** Run vibe-attack; tray icon changes between Idle/Listening/Recording/Muted as daemon transitions; Mode submenu shows current mode checkmarked; selecting the other mode triggers SetMode and the daemon switches without restart

## Must-Haves

- Tray icon changes between Idle/Listening/Recording/Muted as the daemon transitions; Mode submenu shows the current ActivationMode checkmarked; selecting the other mode triggers SetMode and the daemon switches without restart. cargo test passes (default + gui features); cargo build is clean for both feature sets. Unit test maps each Option<DaemonState> variant to the expected icon name.

## Proof Level

- This slice proves: Not provided.

## Integration Closure

Not provided.

## Verification

- Not provided.

## Tasks

- [x] **T01: Expose active_mode through DaemonStatus and DaemonHandle** `est:45m`
  Plumb the daemon's current ActivationMode through the control protocol so the tray can read it. Add `active_mode: ActivationMode` to `DaemonStatus`. Derive `Default` for `ActivationMode` returning `Ptt` so `#[serde(default)]` on the new field keeps existing JSON readable. Add `active_mode: Arc<RwLock<ActivationMode>>` to `DaemonHandle`, initialise it to `Ptt` in `DaemonHandle::new`, expose it from `DaemonHandle::status()`, and update the `SetMode` handler in `spawn_control_listener` to write the new value into the handle BEFORE forwarding `RuntimeCommand::SetMode` to the coordinator (matches the active_profile pattern). Update `tests/control_protocol.rs::status_data_response_roundtrip` to construct/assert the new field. Add a new test `daemon_handle_active_mode_defaults_to_ptt`. Per project convention (MEM002), every new pub item gets a `///` doc comment.

Failure modes are limited to RwLock poisoning — follow the existing `active_profile.write().unwrap()` pattern (a poisoned lock here means the process is already toast).
  - Files: `src/control/protocol.rs`, `src/control/mod.rs`, `tests/control_protocol.rs`
  - Verify: cargo test --test control_protocol && cargo build && cargo build --features gui

- [x] **T02: Map every DaemonState to a distinct tray icon name with unit tests** `est:30m`
  Extract the icon-name match in `src/ui/tray.rs::VibeTray::icon_name` into a free function `pub(crate) fn icon_name_for_state(state: Option<&DaemonState>) -> &'static str` so the mapping is unit-testable without spawning a tray. Mapping:
- None → "audio-input-microphone-muted"
- Some(Muted) → "audio-input-microphone-muted"
- Some(Idle) → "audio-input-microphone"
- Some(Listening) → "audio-input-microphone-high"
- Some(Recording) → "audio-input-microphone"

Update `VibeTray::icon_name` to call the new function with `self.current_state().daemon_state.as_ref()`. Add a `#[cfg(test)] mod tests` block in `tray.rs` with five unit tests:
- `icon_name_for_none_is_muted`
- `icon_name_for_idle`
- `icon_name_for_listening`
- `icon_name_for_recording`
- `icon_name_for_muted`

No D-Bus, no async, no ksni in tests — they call the free function directly. Per project convention (MEM002) the new free function gets a `///` doc comment summarising the mapping rationale.
  - Files: `src/ui/tray.rs`
  - Verify: cargo test --features gui icon_name_for && cargo build && cargo build --features gui

- [ ] **T03: Render Mode submenu in tray and dispatch SetMode on selection** `est:45m`
  In `src/ui/tray.rs`: (1) Extend `TrayState` with `active_mode: Option<ActivationMode>` (None = unknown / daemon stopped). (2) In the poll loop, populate it from `query_status()`'s new `active_mode` field — wrap as `Some(_)` only when the daemon responds. Add `s.active_mode != new_active_mode` to the `changed` comparison and the inner update closure. (3) In `menu()`, after the Profiles submenu and before the Quit separator, push a `SubMenu` labelled "Mode" with two `CheckmarkItem`s — "Push-to-talk" and "Wake word" — each `enabled: daemon_running`, `checked` based on `state.active_mode == Some(ActivationMode::Ptt|Wake)`. The activate closure mirrors the existing Profiles pattern: `std::thread::spawn(move || { let _ = send_command(ControlRequest::SetMode { mode: ... }); })` — fire-and-forget, never blocks the ksni callback.

When `state.active_mode` is `None`, neither checkmark is shown (acceptable fallback per research). Per project convention (MEM002), no new pub items are added so no extra doc comments are required, but any new helpers must carry `///`.

Verification is build-clean for both feature sets and the existing test suite continues to pass — the menu layout itself is exercised manually in S04 UAT (no D-Bus in CI).
  - Files: `src/ui/tray.rs`
  - Verify: cargo test --features gui && cargo build && cargo build --features gui

## Files Likely Touched

- src/control/protocol.rs
- src/control/mod.rs
- tests/control_protocol.rs
- src/ui/tray.rs
