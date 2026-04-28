# Requirements

This file is the explicit capability and coverage contract for the project.

## Validated

### ACT-03 — Untitled
- Status: validated
- Primary owning slice: M008/S02
- Supporting slices: M008/S01, M008/S03, M008/S04
- Validation: M008/S01: SetMode ControlRequest + RuntimeCommand channel; M008/S02: config window mode toggle dispatches SetMode on Save; M008/S03: tray Mode submenu dispatches SetMode fire-and-forget; M008/S04: control_integration test set_mode_round_trip_via_socket passes. Mode swap does not restart the pipeline.

### ACT-04 — Untitled
- Status: validated
- Primary owning slice: M008/S03
- Supporting slices: M008/S01
- Validation: M008/S03: icon_name_for_state free function maps all DaemonState variants (Idle, Listening, Recording, Muted, None) to icon names; 5 unit tests pass; tray polls query_status() and updates icon each tick.

### STT-02 — Untitled
- Status: validated
- Primary owning slice: M008/S01
- Supporting slices: M008/S04
- Validation: M008/S01: PhraseMatcher wrapped in RwLock; threshold is a runtime-mutable f32 owned by coordinator; update_threshold() clamps and replaces matcher under write lock. test_update_threshold_changes_match_behavior passes.

### STT-03 — Untitled
- Status: validated
- Primary owning slice: M008/S02
- Supporting slices: M008/S01, M008/S04
- Validation: M008/S02: config window exposes threshold_pct slider (0-100 integer); Save converts to f32 and dispatches SetThreshold via control socket. M008/S04: set_threshold_via_socket_updates_dispatcher integration test passes.

### UI-02 — Untitled
- Status: validated
- Primary owning slice: M008/S03
- Supporting slices: M008/S01
- Validation: M008/S03: tray icon reflects all DaemonState variants; Mode submenu with PTT/Wake checkmark items dispatches SetMode; profile submenu already present from M001. Tray state fed from JSONL stdout event stream via query_status().

### UI-03 — Untitled
- Status: validated
- Primary owning slice: M008/S02
- Supporting slices: M008/S01, M008/S03, M008/S04
- Validation: M008/S02: config window reads config.yaml on open, exposes mode toggle, threshold slider, input device selector, PTT binding (read-only display in M008); Save dispatches SetMode/SetThreshold/SetInputDevice via control socket with atomic YAML write. Daemon-absent state shows recovery message.

## Traceability

| ID | Class | Status | Primary owner | Supporting | Proof |
|---|---|---|---|---|---|
| ACT-03 |  | validated | M008/S02 | M008/S01, M008/S03, M008/S04 | M008/S01: SetMode ControlRequest + RuntimeCommand channel; M008/S02: config window mode toggle dispatches SetMode on Save; M008/S03: tray Mode submenu dispatches SetMode fire-and-forget; M008/S04: control_integration test set_mode_round_trip_via_socket passes. Mode swap does not restart the pipeline. |
| ACT-04 |  | validated | M008/S03 | M008/S01 | M008/S03: icon_name_for_state free function maps all DaemonState variants (Idle, Listening, Recording, Muted, None) to icon names; 5 unit tests pass; tray polls query_status() and updates icon each tick. |
| STT-02 |  | validated | M008/S01 | M008/S04 | M008/S01: PhraseMatcher wrapped in RwLock; threshold is a runtime-mutable f32 owned by coordinator; update_threshold() clamps and replaces matcher under write lock. test_update_threshold_changes_match_behavior passes. |
| STT-03 |  | validated | M008/S02 | M008/S01, M008/S04 | M008/S02: config window exposes threshold_pct slider (0-100 integer); Save converts to f32 and dispatches SetThreshold via control socket. M008/S04: set_threshold_via_socket_updates_dispatcher integration test passes. |
| UI-02 |  | validated | M008/S03 | M008/S01 | M008/S03: tray icon reflects all DaemonState variants; Mode submenu with PTT/Wake checkmark items dispatches SetMode; profile submenu already present from M001. Tray state fed from JSONL stdout event stream via query_status(). |
| UI-03 |  | validated | M008/S02 | M008/S01, M008/S03, M008/S04 | M008/S02: config window reads config.yaml on open, exposes mode toggle, threshold slider, input device selector, PTT binding (read-only display in M008); Save dispatches SetMode/SetThreshold/SetInputDevice via control socket with atomic YAML write. Daemon-absent state shows recovery message. |

## Coverage Summary

- Active requirements: 0
- Mapped to slices: 0
- Validated: 6 (ACT-03, ACT-04, STT-02, STT-03, UI-02, UI-03)
- Unmapped active requirements: 0
