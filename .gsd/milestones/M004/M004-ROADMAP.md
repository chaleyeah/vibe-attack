# M004: Runtime UX — System Tray + Daemon Control

**Vision:** Replace the terminal-only runtime experience with a proper desktop-integrated UI: a system tray icon that reflects live daemon state (idle / listening / muted) and lets the user mute, switch profiles, and open the config window without touching a terminal. The daemon stays headless by default; the tray is an optional overlay that attaches to a running daemon via the existing control socket.

## Success Criteria

- A system tray icon appears when vibe-attack-config is running and correctly reflects daemon state (idle / listening / muted) polled from the control socket
- User can mute/unmute from the tray menu without opening the full config window
- User can switch the active profile from the tray menu (lists profiles from XDG config dir)
- User can open the config window from the tray menu
- Daemon state changes (e.g. started from terminal) are reflected in the tray within 2 seconds
- Works on X11 and Wayland (XDG system tray / StatusNotifierItem)

## Slices

- [x] **S01: Control Socket — Daemon Status Query** `risk:medium` `depends:[]`
  > After this: cargo test control:: passes; a CLI one-liner (echo STATUS | nc -U /run/...) returns a JSON status line from a running daemon

- [x] **S02: Tray Icon — Static Placeholder** `risk:medium` `depends:[S01]`
  > After this: Run vibe-attack-config on X11 and Wayland; tray icon appears in system bar; Open Config brings the egui window to front; Quit exits cleanly

- [x] **S03: Live Daemon State in Tray** `risk:medium` `depends:[S01,S02]`
  > After this: With daemon running: tray icon turns green when wake-word or PTT is active, red when muted via tray click. With daemon stopped: tray shows grey 'Not running' tooltip.

- [x] **S04: Profile Switcher in Tray** `risk:low` `depends:[S03]`
  > After this: With hd2 profile installed: tray Profiles submenu shows 'hd2' with a checkmark. Selecting a different profile (if present) sends LOAD_PROFILE and the daemon logs the switch.

## Boundary Map

Not provided.
