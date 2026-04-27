---
id: M004
provides:
  - System tray icon reflecting live daemon state (idle/muted/not running)
  - Mute/unmute toggle from tray menu
  - Profile switcher submenu with active-profile checkmark
  - Open Config and Quit actions in tray
key_decisions:
  - ksni chosen for tray integration (StatusNotifierItem — works on both X11 and Wayland via SNI)
  - Poll loop at 1s interval drives live state; only calls handle.update() on change to avoid D-Bus churn
  - Profile switch fires on a fresh OS thread so ksni callback never blocks
  - Mute/Unmute menu item only rendered when daemon is running
patterns_established:
  - TrayState behind tokio::sync::Mutex shared between tray struct and background poll task
  - icon_name driven by DaemonState enum (audio-input-microphone vs audio-input-microphone-muted)
observability_surfaces:
  - Tray tooltip reports daemon state in human-readable form (Not running / Idle / Muted / Listening… / Recording…)
  - D-Bus SNI surface is itself observable via snoop tools
requirement_outcomes: []
duration: ~4 sessions
verification_result: passed
completed_at: 2026-04-26
---

# M004: Runtime UX — System Tray + Daemon Control

**Desktop-integrated runtime UX: a ksni system tray icon that tracks live daemon state and exposes mute/unmute, profile switching, and config access — no terminal required during normal use.**

## What Happened

S01 laid the foundation by extending the control protocol with STATUS/MUTE/UNMUTE commands and wrapping the socket client in a `DaemonHandle` abstraction. This gave the tray a clean async interface to query and mutate daemon state without knowing socket paths or framing details.

S02 wired in ksni and produced a static tray icon with two menu entries: Open Config (raises the egui window) and Quit. This proved the SNI plumbing worked on both X11 and Wayland before adding any dynamic behavior.

S03 added the poll loop: a background tokio task queries `DaemonHandle::status()` every second, compares against `TrayState`, and calls `handle.update()` only when something changed. The icon name and tooltip text are driven by the `DaemonState` enum. Mute/Unmute appears in the menu only when the daemon is alive.

S04 added the profile switcher submenu. Profiles are read from the XDG config directory at menu-open time. The active profile gets a checkmark; selecting another fires `SwitchProfile` on a dedicated OS thread.

## Cross-Slice Verification

- S01: `cargo test control::` passes; STATUS/MUTE/UNMUTE round-trip verified against a live daemon socket.
- S02: Tray icon appears on X11 and Wayland; Open Config raises egui window; Quit exits cleanly.
- S03: Mute/Unmute toggle verified end-to-end — tray icon and tooltip update within ~1s; daemon processes the command and logs the state change.
- S04: Profile submenu lists installed profiles; active profile shows checkmark; switching profiles sends LOAD_PROFILE and daemon logs the switch.

## Requirement Changes

- No formal requirement transitions — M004 was scoped as a feature milestone without pre-registered requirements.

## Decision Re-evaluation

| Decision | Original Rationale | Still Valid? | Action |
|---|---|---|---|
| ksni for SNI tray | Works on X11 + Wayland via StatusNotifierItem; pure Rust | Yes | Keep |
| 1s poll interval | Responsive without hammering D-Bus | Yes | Keep |

## Forward Intelligence

### What the next milestone should know
- `vibe-attack-config` is the process that owns the tray; the daemon (`vibe-attack`) is headless. Any future tray features go in `src/ui/tray.rs`.
- ksni menus are rebuilt from scratch on every open — no incremental update needed. Reading profiles at menu-open time (rather than caching) avoids stale state.
- The control socket path is determined by `DaemonHandle` — tray code never constructs socket paths directly.

### What's fragile
- Profile list reads XDG config dir on every menu open — if the dir doesn't exist yet (first run before wizard), the submenu is empty rather than erroring. Acceptable but worth noting.
- If the daemon crashes and leaves a stale socket file, `query_status()` returns `None` correctly, but the file cleanup path is the daemon's responsibility.

### Authoritative diagnostics
- `RUST_LOG=debug cargo run --bin vibe-attack-config --features gui` — tray poll loop logs each state query result.
- D-Bus snoop (`dbus-monitor --session`) shows SNI property updates when tray state changes.

### What assumptions changed
- Originally assumed ksni might require X11-specific setup; in practice it works transparently on Wayland compositors that implement SNI (KDE, GNOME with AppIndicator extension).

## Files Created/Modified

- `src/control/protocol.rs` — STATUS/MUTE/UNMUTE commands, SwitchProfile command, DaemonState enum
- `src/control/client.rs` — DaemonHandle with query_status(), mute(), unmute(), switch_profile()
- `src/control/mod.rs` — re-exports DaemonHandle
- `src/ui/tray.rs` — VibeTray struct, TrayState, poll loop, icon/tooltip/menu logic, profile submenu
- `src/ui/mod.rs` — tray module wired in
