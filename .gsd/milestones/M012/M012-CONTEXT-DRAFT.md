# M012 — GUI Redesign: Tactical Field Equipment Aesthetic

## Source

Design spec and reference files uploaded to `ui/` by the user. See:
- `ui/README.md` — full implementation brief (authoritative)
- `ui/screenshots/` — 12 PNG reference renders (visual ground truth)
- `ui/theme.css` — design tokens
- `ui/Shell.jsx`, `ui/ConfigApp.jsx`, `ui/Wizard.jsx`, `ui/PackEditor.jsx`, `ui/Icons.jsx` — React prototypes (reference only, not production code)

## Scope

Complete UI rewrite of the three egui surfaces in `src/ui/`:
- `config_app.rs` — main config window (5 nav panes)
- `wizard.rs` — 6-step first-run wizard
- `pack_editor.rs` — 3-column macro editor

Plus new files:
- `src/ui/theme.rs` — palette struct + `apply_theme(ctx)`
- `src/ui/widgets.rs` — 10 shared widget functions

And asset additions:
- `assets/fonts/JetBrainsMono-Regular.ttf`, `JetBrainsMono-Medium.ttf`
- `assets/tray/` — 4 status dot PNGs (22px ARGB)

## Stack Constraints (hard — do not change)

- GUI: `eframe` 0.34 / `egui` (immediate-mode, glow, X11 + Wayland)
- Language: Rust 2021
- No web stack, no JS/CSS, no new heavy deps
- New deps must be optional behind existing `gui` feature flag
- Do NOT replace egui with another toolkit

## Out of Scope

- Audio, STT, VAD, input subsystems — no changes
- `ratatui` TUI — derive colors from same palette eventually, but not part of this milestone
- Windows/macOS support

## Key Design Decisions

### Fidelity
High-fidelity. All colors, type sizes, spacing are final. Where egui can't reproduce something exactly (animated glow, blurred backdrops), drop the effect — the design degrades cleanly.

### Letter-spacing
egui has no native letter-spacing. For uppercase section titles/eyebrows, insert spaces between characters at render time: `"DEVICES"` → `"D E V I C E S"`.

### Side rail animation
Either lerp width stored in state (preferred) or click-to-toggle. Decide in S02.

### Font bundling
Embed JetBrains Mono in `assets/fonts/` and load via `egui::FontDefinitions`. Do not load from system fonts.

### Tray icons
Generate 22px status dot PNGs for ksni ARGB format. Ship in `assets/tray/`. Four states: green (armed), grey (standby), amber (offline), red (error).

## Priority Pain Points (from brief)

1. **Daemon-disconnected error** — replace wall-of-text with `banner(kind=Error)` + Reconnect + View logs. Highest-impact single change.
2. **PTT key capture** in wizard — big dashed-border drop-zone with inline evdev permission hint.

## Design Tokens (summary — full list in ui/README.md)

| Token | Hex |
|---|---|
| bg-window | #0e1012 |
| bg-panel | #14171a |
| accent | #e8a317 |
| fg | #c4c7cb |
| ok | #5fd47a |
| err | #e85d3c |

Corner radius: 3 (inputs/buttons), 4 (windows/banners/cards). No shadows. No blur.

## Implementation Order

1. S01 — theme.rs + font bundling
2. S02 — widgets.rs (10 shared widgets)
3. S03 — ConfigApp rewrite (depends S02)
4. S04 — Wizard rewrite (depends S02)
5. S05 — PackEditor rewrite (depends S02, highest complexity)
6. S06 — Tray icon update (depends S01)
7. S07 — Integration + screenshot capture (depends S03–S06)
