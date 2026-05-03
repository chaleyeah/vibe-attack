# M012: GUI Redesign — Tactical Field Equipment Aesthetic

**Vision:** Replace the current ad-hoc egui UI with a cohesive dark-panel design (amber accent, JetBrains Mono, LED meter, status pill) across all three surfaces — ConfigApp, Wizard, and PackEditor — using a shared theme + widget layer. The redesign ships zero new heavy dependencies, degrades cleanly on egui limitations, and makes the daemon-disconnected error story a single Banner + Reconnect instead of wall-of-text.

## Success Criteria

- apply_theme(ctx) sets all Visuals/Style tokens; confirmed by visual diff against ui/screenshots/ reference renders
- All shared widgets (app_header, side_nav, status_footer, section_header, field_row, primary_button, led_meter, status_pill, kbd, banner) exist in src/ui/widgets.rs and compile
- ConfigApp renders all 5 nav panes (Devices, Voice, Packs, Hotkeys, Advanced) using the new primitives
- Wizard renders all 6 steps including PTT drop-zone capture flow and animated LED mic-test step
- PackEditor renders 3-column layout with category list, macro list with search, and detail panel
- Daemon-disconnected banner replaces wall-of-text error display
- Tray icons use palette-matched status dots (green/amber/red)
- All existing functionality is preserved — no regressions to audio, STT, VAD, or input subsystems
- App builds with cargo build --features gui and launches on X11 and Wayland

## Slices

- [ ] **S01: Theme + Font Foundation** `risk:low` `depends:[]`
  > After this: Launch config app — dark background, amber accent on interactive elements, monospace font throughout.

- [ ] **S02: Shared Widget Library** `risk:medium` `depends:[S01]`
  > After this: Isolated widget test page (or probe screen) showing each widget in its various states with the new theme.

- [ ] **S03: ConfigApp Rewrite** `risk:medium` `depends:[S02]`
  > After this: Config app launches, all 5 panes navigable, disconnected banner appears when daemon is not running.

- [ ] **S04: Wizard Rewrite** `risk:medium` `depends:[S02]`
  > After this: Wizard runs to completion from a clean state; PTT capture drop-zone captures a key; mic-test LED meter animates.

- [ ] **S05: PackEditor Rewrite** `risk:high` `depends:[S02]`
  > After this: Pack editor opens existing pack; drag-reorder categories and macros; search filters macro list; edits persist on save.

- [ ] **S06: Tray Icon Update** `risk:low` `depends:[S01]`
  > After this: System tray icon changes color correctly when daemon state changes.

- [ ] **S07: Integration + Screenshot Capture** `risk:low` `depends:[S03,S04,S05,S06]`
  > After this: Clean build, tests green, screenshot gallery showing all 12 screens vs reference renders.

## Boundary Map

Not provided.
