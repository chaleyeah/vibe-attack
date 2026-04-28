---
estimated_steps: 3
estimated_files: 1
skills_used: []
---

# T03: Render Mode submenu in tray and dispatch SetMode on selection

In `src/ui/tray.rs`: (1) Extend `TrayState` with `active_mode: Option<ActivationMode>` (None = unknown / daemon stopped). (2) In the poll loop, populate it from `query_status()`'s new `active_mode` field — wrap as `Some(_)` only when the daemon responds. Add `s.active_mode != new_active_mode` to the `changed` comparison and the inner update closure. (3) In `menu()`, after the Profiles submenu and before the Quit separator, push a `SubMenu` labelled "Mode" with two `CheckmarkItem`s — "Push-to-talk" and "Wake word" — each `enabled: daemon_running`, `checked` based on `state.active_mode == Some(ActivationMode::Ptt|Wake)`. The activate closure mirrors the existing Profiles pattern: `std::thread::spawn(move || { let _ = send_command(ControlRequest::SetMode { mode: ... }); })` — fire-and-forget, never blocks the ksni callback.

When `state.active_mode` is `None`, neither checkmark is shown (acceptable fallback per research). Per project convention (MEM002), no new pub items are added so no extra doc comments are required, but any new helpers must carry `///`.

Verification is build-clean for both feature sets and the existing test suite continues to pass — the menu layout itself is exercised manually in S04 UAT (no D-Bus in CI).

## Inputs

- ``src/ui/tray.rs``
- ``src/control/protocol.rs``

## Expected Output

- ``src/ui/tray.rs` — adds active_mode to TrayState, polls it from query_status, adds Mode submenu with two CheckmarkItems that dispatch ControlRequest::SetMode via send_command on a fire-and-forget thread`

## Verification

cargo test --features gui && cargo build && cargo build --features gui
