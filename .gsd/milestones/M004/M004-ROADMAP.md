# M004: Runtime UX - System Tray + Daemon Control

**Vision:** 

## Slices

- [x] **S01: S01** `risk:medium` `depends:[]`
  > After this: cargo test control:: passes; a CLI one-liner (echo STATUS | nc -U /run/...) returns a JSON status line from a running daemon

- [x] **S02: S02** `risk:medium` `depends:[]`
  > After this: Run vibe-attack-config on X11 and Wayland; tray icon appears in system bar; Open Config brings the egui window to front; Quit exits cleanly

- [x] **S03: S03** `risk:medium` `depends:[]`
  > After this: With daemon running: tray icon turns green when wake-word or PTT is active, red when muted via tray click. With daemon stopped: tray shows grey 'Not running' tooltip.

- [x] **S04: S04** `risk:low` `depends:[]`
  > After this: With hd2 profile installed: tray Profiles submenu shows 'hd2' with a checkmark. Selecting a different profile (if present) sends LOAD_PROFILE and the daemon logs the switch.
