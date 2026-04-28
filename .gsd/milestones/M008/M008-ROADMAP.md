# M008: UI / Tray Runtime Control

**Vision:** A user can right-click the tray icon, change activation mode and confidence threshold, and have the running daemon honor those changes immediately — no restart, no hand-editing YAML. After this milestone the runtime-control surface is complete: tray → config window → daemon.

## Success Criteria

- cargo test passes (all non-hardware-gated tests) at the end of every slice
- cargo clippy -D warnings clean for both default and gui feature sets throughout
- SetMode, SetThreshold, SetInputDevice, SetPttBinding, ReloadConfig control requests have request/response types, server handlers, and round-trip serde tests
- ConfigApp exposes mode/threshold/input-device/PTT-binding fields with unit tests
- Tray icon reflects all DaemonState variants (Idle, Listening, Recording, Muted) — verified by a unit test mapping each variant to its icon name
- Mode swap (PTT ↔ wake-word) is performed via a RuntimeCommand MPSC channel from coordinator; only the activation thread is restarted (not the full pipeline)
- Threshold is owned by the coordinator as an f32; activation thread receives a fresh matcher on mode swap; threshold changes take effect on the next utterance cycle
- Config window reads config.yaml on open, writes on Save, sends ReloadConfig via control socket; audio device changes prompt 'Save + Restart' rather than live hot-swap
- Tray icon state is fed from JSONL stdout event stream (not a new control-protocol push path)
- Config window shows 'daemon not running' state with recovery affordance when socket is absent; no panics

## Slices

- [x] **S01: S01** `risk:high` `depends:[]`
  > After this: cargo test passes including new control_protocol tests; round-trip serde test for each new request variant passes; coordinator accepts RuntimeCommand::SetMode and RuntimeCommand::SetThreshold without restarting the full pipeline

- [ ] **S02: S02** `risk:medium` `depends:[]`
  > After this: vibe-attack-config opens a config window; user changes mode toggle → Save → daemon log shows SetMode received; threshold slider moves → Save → daemon log shows SetThreshold received

- [ ] **S03: Tray icon state mapping + Mode submenu** `risk:low` `depends:[S02]`
  > After this: Run vibe-attack; tray icon changes between Idle/Listening/Recording/Muted as daemon transitions; Mode submenu shows current mode checkmarked; selecting the other mode triggers SetMode and the daemon switches without restart

- [ ] **S04: End-to-end UAT + headless integration test** `risk:low` `depends:[S03]`
  > After this: cargo test --test control_integration passes; S04-UAT.md manual steps produce 'mode changed, stratagem fired by voice' result without daemon restart

## Boundary Map

## Boundary Map

### Internal boundaries touched
- **src/control/protocol.rs** — add SetMode, SetThreshold, SetInputDevice, SetPttBinding, ReloadConfig to ControlRequest/ControlResponse
- **src/control/mod.rs** — server handlers for new requests; RuntimeCommand MPSC channel wired to coordinator
- **src/pipeline/coordinator.rs** — add RuntimeCommand channel; drain between utterances; surgical activation-thread swap on SetMode
- **src/pipeline/matcher.rs** — threshold becomes runtime-mutable via coordinator-owned f32; matcher rebuilt on activation-thread spawn
- **src/ui/config_app.rs** — new fields: mode, threshold, input_device, ptt_binding; Save logic
- **src/ui/tray.rs** — JSONL stdout parsing for state; Mode submenu; icon/tooltip mapping for all DaemonState variants
- **src/bin/vibe-attack-config.rs** — main config panel routing

### Untouched (explicitly out of scope)
- Config file YAML schema changes beyond adding mode/threshold fields
- JSONL log schema
- Control plane Unix socket protocol (extends but does not break existing variants)
- All packaging files
- Macro editor (M009)
