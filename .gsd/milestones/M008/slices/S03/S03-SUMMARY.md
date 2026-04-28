---
id: S03
parent: M008
milestone: M008
provides:
  - ["active_mode field in DaemonStatus and DaemonHandle (control protocol readable)", "icon_name_for_state free function with 5 unit tests covering all DaemonState variants", "Mode submenu in tray with PTT/Wake checkmark items dispatching SetMode"]
requires:
  []
affects:
  - ["S04"]
key_files:
  - ["src/control/protocol.rs", "src/control/mod.rs", "tests/control_protocol.rs", "src/ui/tray.rs"]
key_decisions:
  - ["Arc<RwLock<ActivationMode>> on DaemonHandle (not AtomicXxx) — ActivationMode is not atomically encodable; matches the active_profile pattern.", "SetMode handler writes active_mode to handle BEFORE forwarding RuntimeCommand to coordinator — Status responses always coherent with last SetMode.", "Option<ActivationMode> in TrayState so None cleanly represents daemon-not-running without a sentinel value.", "query_status() called once per tick; active_mode extracted with as_ref().map() before active_profile's consuming and_then() call.", "icon_name_for_state extracted as free pub(crate) function (not method) to enable unit tests without D-Bus/ksni.", "Idle and Recording collapsed to single match arm since both map to 'audio-input-microphone'.", "Mode submenu activate closures use std::thread::spawn fire-and-forget — ksni D-Bus callbacks must not block."]
patterns_established:
  - ["Free pub(crate) functions for icon/state mappings enable headless unit tests without D-Bus or ksni instantiation.", "TrayState optional fields use Option<T> for daemon-not-running sentinel — enables natural item disabling via daemon_running flag.", "All tray menu activate closures follow fire-and-forget spawn pattern: std::thread::spawn → send_command → discard result."]
observability_surfaces:
  - ["tracing::debug! log in SetMode handler emits cached mode value after write — visible in RUST_LOG=debug daemon output"]
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-28T01:55:38.377Z
blocker_discovered: false
---

# S03: Tray icon state mapping + Mode submenu

**Tray icon now reflects all DaemonState variants and exposes a Mode submenu that dispatches SetMode fire-and-forget to switch PTT ↔ Wake without daemon restart**

## What Happened

Three tasks built the tray state surface from control protocol up through the menu layer.

**T01 — active_mode through DaemonStatus and DaemonHandle**: Added `Default for ActivationMode` (returns `Ptt`) and `active_mode: ActivationMode` with `#[serde(default)]` to `DaemonStatus` in `src/control/protocol.rs`, keeping existing JSON readable. Added `active_mode: Arc<RwLock<ActivationMode>>` to `DaemonHandle`, initialized to `Ptt`, exposed from `status()`. The `SetMode` handler writes the cache before forwarding `RuntimeCommand::SetMode` to the coordinator — same ordering as `active_profile` — so `Status` responses are always coherent with the last SetMode. Five new tests: `daemon_handle_active_mode_defaults_to_ptt`, `daemon_handle_active_mode_updates_on_write`, `status_active_mode_serializes_snake_case`, `daemon_status_backward_compat_no_active_mode_field` (serde default path), and an update to `status_data_response_roundtrip`. All 21 control_protocol tests pass.

**T02 — icon_name_for_state free function**: Extracted the inline icon-name match from `VibeTray::icon_name` into `pub(crate) fn icon_name_for_state(state: Option<&DaemonState>) -> &'static str` placed above the impl block. Mapping: `None`/`Some(Muted)` → `"audio-input-microphone-muted"`, `Some(Idle)`/`Some(Recording)` → `"audio-input-microphone"`, `Some(Listening)` → `"audio-input-microphone-high"`. `Idle` and `Recording` share a single match arm since they map identically. Five unit tests call the free function directly — no D-Bus, no ksni, no async. `VibeTray::icon_name` reduced to a one-liner delegating to the free function.

**T03 — Mode submenu and TrayState.active_mode**: Extended `TrayState` with `active_mode: Option<ActivationMode>` (None = daemon stopped/unknown). In the tokio poll loop, `query_status()` is called once; `new_active_mode` is extracted with `status.as_ref().map()` before the consuming `and_then()` call for `active_profile`. Added `active_mode` to the `changed` guard and `update` closure, mirroring the `active_profile` pattern exactly. Added a Mode `SubMenu` after Profiles with two `CheckmarkItem`s ("Push-to-talk", "Wake word"), both `enabled: daemon_running`, checked against `state.active_mode`. Each activate closure spawns a thread and calls `send_command(ControlRequest::SetMode { mode: ... })` fire-and-forget — matching the Mute/Profile pattern so ksni callbacks never block. When `active_mode` is `None`, neither item is checked and both are greyed (daemon not running).

## Verification

cargo test --test control_protocol: 21/21 pass. cargo test --features gui icon_name_for: 5/5 pass (icon_name_for_none_is_muted, icon_name_for_idle, icon_name_for_listening, icon_name_for_recording, icon_name_for_muted). cargo test --features gui: 63 passed, 0 failed, 1 ignored (privileged hardware test). cargo build (default features): clean. cargo build --features gui: clean. All slice-level verification criteria met.

## Requirements Advanced

- ACT-03 — Mode submenu in tray now lets user switch PTT ↔ Wake-word at runtime; SetMode dispatched fire-and-forget without daemon restart
- ACT-04 — icon_name_for_state maps all DaemonState variants (Idle, Listening, Recording, Muted, None) to distinct freedesktop icon names; tray polls and updates on every state change
- UI-02 — Tray icon now reflects full daemon state and exposes Mode submenu for runtime control

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

T01 added three extra tests beyond the two specified (daemon_handle_active_mode_updates_on_write, status_active_mode_serializes_snake_case, daemon_status_backward_compat_no_active_mode_field) to cover the serde default path and tray-relevant snake_case serialization — implied by the task narrative. T02 collapsed Idle and Recording into a single match arm (both map to 'audio-input-microphone') — avoids redundancy while remaining exhaustive.

## Known Limitations

Mode submenu menu layout is exercised manually in S04 UAT; no automated menu rendering test is possible in CI (no D-Bus session). Recording state icon requires a live phrase match to observe; not verifiable in headless CI.

## Follow-ups

S04 end-to-end UAT will verify the full round-trip: tray Mode switch → daemon mode change → stratagem fired by voice without restart. The headless integration test (cargo test --test control_integration) covering SetMode round-trip is planned for S04.

## Files Created/Modified

None.
