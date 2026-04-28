# M008: UI / Tray Runtime Control

**Gathered:** 2026-04-27
**Status:** Planned — architectural decisions locked in, ready for execution

## Project Description

vibe-attack already has a working pipeline (audio → VAD → wake/PTT → STT → dispatcher → uinput) and a system-tray binary (`src/ui/tray.rs`) that exposes Mute/Unmute, profile switch, and Open Config via D-Bus (ksni). What's missing is a **real config window** the tray can open, the **runtime mode toggle** (PTT ↔ wake-word) the user expects to flip mid-session, and a **confidence-threshold slider** for fuzzy phrase matching.

This milestone closes the runtime-control surface: tray → config window → daemon. After this milestone, a user can install vibe-attack, click the tray icon, change mode and threshold, and have the daemon honor it without a restart.

## Why This Milestone

The runtime-control gap is the single largest barrier to first-time users actually using the tool. The pipeline works; the tray works; but there is no in-app way to change activation mode, configure threshold, or pick an audio device — users must hand-edit `~/.config/vibe-attack/config.yaml` and restart the daemon. That is unacceptable for a "small release" target audience.

This is the natural next milestone because (a) the tray already partially exists and is shipping behavior, (b) every other UX requirement (UI-04 wizard, PACK-04 macro editor) depends on a working config window, and (c) the requirements clustered here all share the same surfaces — `ConfigApp`, control protocol, tray menu.

## User-Visible Outcome

### When this milestone is complete, the user can:

- Right-click the tray icon and toggle between PTT and wake-word mode without restarting the daemon
- Open a config window from the tray that lets them set audio input device, activation mode, confidence threshold, and the PTT keybinding
- Watch the tray icon change between idle / listening / muted states reflecting actual daemon state
- Save changes from the config window and have the daemon pick them up live (or with a clean reload, no manual restart)

### Entry point / environment

- Entry point: `vibe-attack` (daemon) + `vibe-attack-config` (GUI) launched from desktop file or AppImage
- Environment: Linux desktop (Wayland or X11), D-Bus session bus available
- Live dependencies involved: D-Bus (ksni), Unix-socket control plane, evdev, PipeWire/PulseAudio via CPAL

## Completion Class

- Contract complete means: control-protocol additions (set-mode, set-threshold, set-input-device, set-ptt-binding) have request/response types, server handlers, and unit tests; `ConfigApp` exposes pure-logic methods for each surface with tests
- Integration complete means: the egui config window and tray menu issue real control commands, the daemon applies them live, and a manual run from `vibe-attack-config` proves the round trip — change mode in the window → daemon log shows mode change → wake/PTT activation behaves accordingly
- Operational complete means: the config window survives a daemon restart (reconnects), the tray correctly reflects "daemon not running" vs each running state, and there are no panics when the user opens the config window before the daemon starts

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- A user runs `vibe-attack-config` from the desktop, the window opens, they change mode from PTT to wake-word, click Save, and the next utterance is dispatched via wake-word — without restarting `vibe-attack`
- The tray icon visually changes (icon name AND tooltip) when the daemon transitions Idle → Listening → Recording → Idle
- The confidence-threshold slider in the config window changes a value the dispatcher actually consults — verified by a test that feeds a fuzzy phrase at threshold-1 (rejected) and threshold+1 (accepted)
- The config window does not crash or hang when the daemon is not running; it shows a clear "daemon not running" state with a Start button or instructions

## Architectural Decisions

### Config window UI framework

**Decision:** Continue using egui via the `gui` feature flag (already in tree).

**Rationale:** egui is already wired into `vibe-attack-config` (`src/bin/vibe-attack-config.rs`), already used for the first-run wizard, and is the only GUI dependency the project carries. Adding GTK/Qt now would double the dependency surface for no gain.

**Alternatives Considered:**
- GTK4 — would match the GNOME desktop but doubles toolchain complexity and is harder to package in AppImage
- Web UI served on localhost — adds an HTTP stack and breaks the "local desktop app" model

### Live config reload vs restart

**Decision:** Add a `ReloadConfig` control request that re-reads `config.yaml` and re-builds the activation mode hot-swappable; threshold is a runtime setting, not a config-file setting (or both — file is persistent, runtime overrides until next save).

**Rationale:** Restarting the daemon for every threshold tweak is hostile during a first-run tuning session. The pipeline already has bounded channels and crossbeam senders — swapping the activation mode is achievable without tearing down the audio thread.

**Alternatives Considered:**
- Restart-only — simpler but produces a 2-3 second audio gap on every change; unacceptable during gameplay tuning
- Full config hot-reload — broader scope; defer non-mode/non-threshold settings (audio device change) to "save + restart prompt"

---

## Error Handling Strategy

The control plane returns explicit `ControlResponse::Error(message)` variants. The config window surfaces those in a non-modal status bar, never panics. If the daemon disconnects mid-session, the window switches to "daemon not running" state and lets the user retry. Audio-device-change errors (CPAL device not found, permission denied) propagate as a typed variant the UI can render without exposing internal paths.

## Risks and Unknowns

- Live activation-mode swap may race with the audio callback thread — needs a clean shutdown of the wake/PTT side before swap
- egui + ksni in the same process: `vibe-attack-config` already handles this for the first-run wizard, but we haven't tested an active tray + active config window simultaneously
- Audio device enumeration via CPAL is platform-dependent; PipeWire vs PulseAudio may surface different device names — need to test on both Debian (PulseAudio) and Arch (PipeWire)
- `serde::Deserialize` impl for `EvDevKey` (PTT binding rebinder UI) — requires a key-capture widget; non-trivial in egui

## Existing Codebase / Prior Art

- `src/ui/tray.rs` — D-Bus tray with mute/profile-switch/open-config; needs mode toggle and reload-config menu items
- `src/ui/config_app.rs` — pure-logic state for the config window; currently exposes profile list, log lines, mic level. Needs threshold field, mode field, input-device field, PTT-binding field
- `src/bin/vibe-attack-config.rs` — egui binary host; render loop already routes wizard panels — extend with a main config panel
- `src/control/protocol.rs` — `ControlRequest` enum (Mute/Unmute/SwitchProfile/etc); add `SetMode`, `SetThreshold`, `SetInputDevice`, `SetPttBinding`, `ReloadConfig`
- `src/control/mod.rs` — server-side handlers; mirror new requests
- `src/pipeline/matcher.rs` — phrase matcher with confidence; threshold currently a config-file constant — needs to be runtime-mutable via `Arc<AtomicU32>` or similar
- `src/input/ptt.rs` — PTT key binding; needs a setter that rebinds without restart

## Relevant Requirements

- **ACT-03** — switch PTT ↔ wake-word mode at runtime — primary objective
- **ACT-04** — tray icon reflects idle / listening / muted — partial; tray already has Mute and basic state, needs full state-icon mapping for Listening/Recording
- **STT-02** — confidence threshold for fuzzy phrase matching — primary objective
- **STT-03** — configure threshold from the config UI — primary objective
- **UI-02** — tray with mute / profile switch / open config — partial; already implemented, this milestone adds Mode toggle and Reload entries
- **UI-03** — config window: audio input device, activation mode, confidence threshold, keybindings — primary objective

## Scope

### In Scope

- Adding mode toggle, threshold slider, input-device picker, PTT key-capture to the egui config window
- Extending `ControlRequest`/`ControlResponse` with `SetMode`, `SetThreshold`, `SetInputDevice`, `SetPttBinding`, `ReloadConfig`
- Server-side handlers that apply changes live (mode swap, threshold update, PTT rebind)
- Tray icon state mapping for all `DaemonState` variants (Idle, Listening, Recording, Muted)
- Tray menu additions: Mode submenu (PTT / Wake) with the active option checkmarked
- `config.yaml` persistence — Save button writes the current `ConfigApp` state back to disk

### Out of Scope / Non-Goals

- Macro editor (PACK-04) — separate milestone
- First-run wizard polish (UI-04) — separate milestone or a later slice if time allows
- Pack import/export UI (PACK-02/PACK-03) — separate milestone (M009)
- AppImage / AUR distribution — separate milestone (M010)
- Conditional logic in macros (MCRO-03) and per-macro sounds (MCRO-04) — out of scope
- Theming / dark-mode polish — out of scope; egui defaults are fine
- Internationalization — out of scope

## Technical Constraints

- All existing tests must pass
- `cargo clippy -D warnings` clean for both default and `gui` feature sets (CI enforces)
- No regressions in headless daemon mode — `vibe-attack` (no `gui`) must still build and run
- Control-plane backwards compatibility: the request enum is `#[serde(tag = "type")]` and unknown variants must be rejected with a clean error, not a panic

## Integration Points

- D-Bus session bus (ksni) — tray already uses it; new menu items don't change the contract
- Unix-socket control plane — extend the request enum and handlers
- CPAL — device enumeration for the audio-input dropdown
- evdev — PTT key capture widget reads raw events

## Testing Requirements

- Unit tests for new `ControlRequest` variants (round-trip serde)
- Unit tests for `ConfigApp` mode/threshold/device fields and validation
- Server-handler tests in `tests/control_protocol.rs` for each new request — assert the daemon-state side effect
- Headless integration test that drives a mode swap via the control socket and asserts the dispatcher uses the new mode on the next utterance
- Manual UAT script in `S0X-UAT.md` covering: launch config window → change mode → verify dispatcher behavior

## Acceptance Criteria

(Per-slice — to be refined during decomposition)

- All new control requests have explicit pass/fail tests
- Tray icon state mapping verified by a unit test that constructs each `DaemonState` and checks the resulting icon name
- Config window opens, displays current daemon state, accepts changes, persists to `config.yaml`, and applies live
- Round-trip from tray menu → control plane → daemon state change → tray icon update completes within 2 seconds

## Open Questions

- Does the audio-device dropdown need to live-restart the audio thread, or is a "Save + Restart Required" prompt acceptable for device changes? — current thinking: prompt for device, live for mode/threshold/PTT
- Should the threshold slider be 0–100 (integer percent) or 0.0–1.0 (float)? — current thinking: integer 0–100 in the UI, stored as f32 0.0–1.0 in config
- PTT key capture — single key only, or chord support? — current thinking: single key for M008; chord support is a follow-up if asked

## Suggested Slice Decomposition

- **S01** (high risk, depends:[]): Control-protocol extensions — `SetMode`, `SetThreshold`, `SetInputDevice`, `SetPttBinding`, `ReloadConfig`, plus server handlers and tests
- **S02** (medium risk, depends:[S01]): `ConfigApp` state + egui main config panel (mode toggle, threshold slider, device dropdown, PTT capture, Save button)
- **S03** (low risk, depends:[S02]): Tray icon state mapping for all daemon states + Mode submenu in tray
- **S04** (low risk, depends:[S03]): End-to-end UAT — manual script + automated headless integration test

## Locked Architectural Decisions (from grilling session 2026-04-27)

These decisions were stress-tested before planning and are binding for M008 execution.

### Runtime-mutable state transport
**Decision B chosen:** Bounded MPSC channel from control plane → coordinator carrying `RuntimeCommand::SetMode` / `SetThreshold`. Coordinator drains it between utterances and applies changes. No shared mutable state across threads.

**Why over Arc<RwLock>:** Coordinator already owns an event loop (bounded crossbeam channels); adding a command channel fits the pattern. Arc<RwLock> risks blocking the CPAL real-time callback if the audio thread holds the lock during a write.

### Mode swap strategy
**Decision A chosen:** Surgical swap of only the activation thread (PTT or wake-word) on mode change. The audio ring-buf, VAD, STT, and dispatcher threads stay live.

**Why over full restart:** Full restart drops buffered audio mid-utterance (~200–500ms gap); the user would notice during gameplay. The activation thread is isolated at the VAD→STT boundary, so tearing down only that thread is architecturally clean.

### Threshold ownership
**Decision B chosen:** Coordinator owns threshold as a local `f32`. On `SetThreshold`, coordinator updates its value. The activation thread receives a fresh `PhraseMatcher` at spawn time. Threshold changes take effect on the next utterance cycle.

**Why over threading command channel to leaf thread:** Keeps all mutable runtime state in one place (coordinator). The activation thread is torn down on mode swap anyway, so rebuilding the matcher at spawn is free.

### Tray state transport
**Decision A chosen:** Tray reads JSONL stdout from the daemon subprocess. Parse event lines and update icon state.

**Why over extending control protocol:** Zero protocol changes. JSONL on stdout is already the canonical event stream. Server-push to the socket is a larger change worth doing eventually but not at M008 scope.

### Config persistence model
**Decision A chosen:** Config window reads `config.yaml` on open, writes on Save, sends `ReloadConfig` via control socket. Audio device changes prompt "Save + Restart Required" rather than live hot-swap.

**Why over live sync:** Simpler, auditable, and consistent with how most config tools work. Live sync for non-mode/non-threshold settings (audio device, PTT binding) adds protocol surface that's hard to validate in one milestone.
