# Handoff: Vibe Attack — GUI Redesign

## Overview

A complete UI redesign for **Vibe Attack**, an open-source voice-macro daemon for Helldivers 2 on Linux. The redesign reframes the app as a piece of tactical field equipment — armed/standby states, LED-style level meters, monospace numerics, amber accent — using only primitives that **egui can actually render** (1px strokes, flat fills, rounded rects, no shadows, no blurs).

Target branch: `claude/vibe-attack-ui-design-pGAHo`.

## About the Design Files

**The HTML/JSX files in this bundle are design references — they are NOT production code to copy.** They are prototypes built in React + CSS to communicate the intended look, behavior, and information architecture. Your task is to **recreate these designs in Vibe Attack's existing Rust + egui stack**, mapping the visual tokens onto `egui::Visuals` / `egui::Style` and the layout primitives onto helper functions in `src/ui/widgets.rs` (or similar).

**Stack constraints (from the original brief — do not change):**
- GUI framework: `eframe` 0.34 / `egui` (immediate-mode, glow renderer, X11 + Wayland)
- Secondary surfaces: `ratatui` TUI, `ksni` system tray, `rfd` file dialogs
- Language: Rust 2021, no web stack, no JS/CSS
- No new heavy dependencies; new deps must be optional behind the existing `gui` feature flag

## Fidelity

**High-fidelity.** All colors, type sizes, spacing, and component states are final and intentional. Recreate pixel-faithfully in egui. Where egui can't reproduce something exactly (animated glow on the LED meter, blurred backdrops), drop the effect — the design was authored to degrade cleanly.

## Screenshots

PNG renders of every screen are in `screenshots/`:

- `01-config-devices.png` — main config, Devices pane
- `02-config-voice.png` — main config, Voice pane (PTT mode)
- `03-config-packs.png` — main config, Packs pane (profile cards)
- `04-config-hotkeys.png` — main config, Hotkeys pane
- `05-config-error-state.png` — daemon-disconnected banner state
- `06-wizard-welcome.png` through `11-wizard-done.png` — full 6-step wizard flow
- `12-pack-editor.png` — three-pane macro editor

Use these as the visual ground truth when implementing.

## Files in this Bundle

- `Vibe Attack Redesign.html` — entry point, mounts everything
- `theme.css` — design tokens (translate to `theme.rs`)
- `Icons.jsx` — two icon sets: bundled SVG (16px line, 1.5 stroke) and unicode fallback
- `Shell.jsx` — shared layout primitives (header, rail, footer, fields, meter, banner, slider, switch)
- `ConfigApp.jsx` — main config window, all 5 nav panes
- `Wizard.jsx` — 6-step first-run wizard
- `PackEditor.jsx` — three-pane pack editor
- `design-canvas.jsx`, `tweaks-panel.jsx` — design-tool scaffolding (you can ignore — they're for the canvas presentation only)

## Surfaces to Implement

### 1. Theme (`src/ui/theme.rs`) — implement first

Single function: `pub fn apply_theme(ctx: &egui::Context)`. Sets `Visuals`, `Style.spacing`, fonts, and an accent palette resource. See **Design Tokens** section below for exact values.

### 2. Shared widgets (`src/ui/widgets.rs`)

Reusable functions every screen calls:

- `side_nav(ui, items: &[NavItem], active: &mut NavId)` — collapsible rail. Default 52px wide; on hover (or on `expanded` flag) animate to 168px and fade in labels. Active item has 2px left accent bar + accent-tinted background.
- `app_header(ui, status: DaemonStatus, ...)` — 44px tall, brand mark + "VIBE ATTACK" + version + status pill + flexible spacer + action icons
- `status_footer(ui, status, mic_level, model_name, sock_path, uptime)` — 30px tall mono strip. LED meter (20 segments, ~200px) on the left. Pipe-separated cells (`MIC | STATE | MODEL | SOCK | UP`) right of it.
- `section_header(ui, title, subtitle, actions)` — uppercase 11px title with 4×4 amber square bullet, optional subtitle, optional right-side actions.
- `field_row(ui, label, required, body, help)` — 140px label column + flexible body column, 28px row height, 11px uppercase muted label, optional 11px help line beneath.
- `primary_button(ui, label, icon)` — amber-filled, uppercase 11px, 0.06em letter-spacing
- `led_meter(ui, level: f32, segments: usize)` — gap-separated cells, green / amber / red zones at 65% / 85%
- `status_pill(ui, state)` — 24px tall pill with colored dot, uppercase 11px label
- `kbd(ui, label)` — small inset key cap (extreme bg, double bottom border)
- `banner(ui, kind, title, body, actions)` — for errors like the daemon-disconnected case

### 3. Main Config Window — `src/ui/config_app.rs`

Layout (top to bottom): `app_header` → 2-column body (`side_nav` left, scrolling content right) → `status_footer`.

**Five nav panes** (collapsible rail items):
1. **Devices** — input device select, sample rate, live monitor (LED meter with dB labels), VAD sensitivity & min-speech sliders.
2. **Voice** — trigger mode radios (PTT / Wake / Always), conditional PTT-key or wake-phrase row, Whisper model file path + browse, confidence threshold slider, language select.
3. **Packs** — 2×2 grid of profile cards (radio target icon, name, macro count + mode, ACTIVE tag on selected). Below: "Open editor", "Export", "Delete pack" buttons.
4. **Hotkeys** — labeled rows for PTT, mute, cycle profile, pause daemon, open config. Each row has a `kbd` chip + raw `KEY_*` code + Rebind button.
5. **Advanced** — Autostart switch, run-as-user-service switch, socket path input, log level select. Danger zone with Reset/Wipe buttons.

**Error state:** when `DaemonStatus::Error`, a `banner(kind=Error)` appears above the active pane explaining the socket failure with Reconnect / View logs actions. Replaces the wall-of-text in the original UI.

### 4. Wizard — `src/ui/wizard.rs`

Layout: `app_header` → step indicator strip → scrolling step body (centered, ~580px max width) → 60px footer with Back / Skip / Continue buttons.

**Step indicator:** 6 numbered circles connected by 1px lines. States: `pending` (muted ring), `active` (amber-filled, dark text), `done` (green ring + check + green-tinted bg). Step label uppercase 11px below — wait, actually beside the number.

**Steps:**
1. **Welcome** — section header + paragraph + 2×2 grid of capability cards (AUDIO / VAD / INPUT / STT)
2. **Audio** — vertical list of device cards (radio target + name + meta line + optional badges/warnings)
3. **Mic test** — animated LED meter (40 segments) inside an extreme-bg card + "TOO HOT / GOOD / OK / QUIET" status text + INFO banner with detected utterance count
4. **PTT** — large dashed-border drop zone displaying current binding as a `kbd`, with "Capture new binding" button. When listening: solid amber border, amber-tinted bg, "▸ LISTENING — PRESS A KEY" + animated pulse dots. Notes about evdev permissions below.
5. **Model** — vertical list of model cards (tiny / base / small / medium) with size, latency, accuracy meta. "DOWNLOAD" tag on right.
6. **Done** — centered green check circle + "READY" eyebrow + headline + paragraph

Wizard skip behavior: optional steps (Audio test, PTT — if user wants wake mode) show "Skip for now" in footer. Don't block on optional validation.

### 5. Pack Editor — `src/ui/pack_editor.rs`

Layout: `app_header` (with Undo / Export / Save pack actions) → pack identity subheader (icon + EDITING PACK label + name + ACTIVE tag + macro count + file path on right) → 3-column body (`200px | 280px | 1fr`) → `status_footer`.

**Column 1 — Categories:** "CATEGORIES" mini-header + add button. Drag-handled rows with name + count.

**Column 2 — Macros:** Search input (with magnifier icon prefix). Scrolling list of macro rows: drag handle + name + 2-digit index (right), then trigger phrase in muted small text on second line. "Add macro" button pinned to bottom.

**Column 3 — Detail:** Three sections:
- *Macro* — name, trigger phrase (with `|` variant separator hint), min confidence slider. Header has Test / Delete actions.
- *Key Sequence* — extreme-bg card showing the keychain as `kbd` chips separated by `+` and `→`. "Capture" button on right. Below: Hold time and inter-key delay sliders.
- *Conditions* — Required flag, Set-flag-on-fire, Cooldown slider.

Drag-reorder for both categories and macros. Inline rename on double-click. Undo affordance in the header.

### 6. Tray Menu — `src/ui/tray.rs`

(Out-of-scope for the design, but the brief asks for status icons matching the new accent palette.) Use the same status semantics as the pill: green dot for armed, muted-grey for standby, amber-blinking for offline, red for error. ksni accepts ARGB icon data — render the dot at 22px and ship as PNGs in `assets/tray/`.

## Design Tokens

Translate these to a Rust `Palette` struct + an `apply_theme` builder. egui equivalent fields noted in parens.

### Surfaces (`Visuals.window_fill`, `panel_fill`, `extreme_bg_color`, etc.)

| Token | Hex | Use |
|---|---|---|
| `bg-extreme` | `#08090a` | text inputs, deepest wells |
| `bg-window`  | `#0e1012` | main window bg |
| `bg-panel`   | `#14171a` | header, footer, side rail |
| `bg-faint`   | `#1a1d20` | alt rows, hover surfaces |
| `bg-raised`  | `#1f2327` | secondary buttons |
| `bg-hover`   | `#262a2f` | `widgets.hovered.bg_fill` |
| `bg-active`  | `#2d3238` | `widgets.active.bg_fill` |

### Strokes (`Stroke`)

| Token | Hex |
|---|---|
| `stroke-faint`  | `#22262b` |
| `stroke`        | `#2d3238` |
| `stroke-strong` | `#3a4047` |
| `stroke-bright` | `#4d555e` |

### Text (`Visuals.override_text_color` + `widgets.*.fg_stroke`)

| Token | Hex |
|---|---|
| `fg-strong` | `#e6e7e8` |
| `fg`        | `#c4c7cb` |
| `fg-muted`  | `#8a9099` |
| `fg-faint`  | `#5d646d` |
| `fg-dim`    | `#444a52` |

### Accent — Amber

| Token | Hex |
|---|---|
| `accent`       | `#e8a317` |
| `accent-hover` | `#f5b733` |
| `accent-faint` | `rgba(232, 163, 23, 0.14)` |
| `accent-line`  | `rgba(232, 163, 23, 0.42)` |
| `accent-fg`    | `#15110a` (text on accent fills) |

### Status

| Token | Hex |
|---|---|
| `ok`    | `#5fd47a` |
| `warn`  | `#e8a317` (same as accent) |
| `err`   | `#e85d3c` |
| `info`  | `#6ab8e8` |

### Geometry

- Corner radius: `3` for inputs/buttons, `4` for windows/banners/cards
- Stroke width: `1.0` everywhere; `0.0` for ghost buttons
- No shadows. No blur. Rely on stroke + bg contrast.

### Type — JetBrains Mono only

Bundle `JetBrainsMono-Regular.ttf` and `JetBrainsMono-Medium.ttf` in `assets/fonts/` and load via `egui::FontDefinitions`.

| Role | Size | Weight | Letter-spacing | Transform |
|---|---|---|---|---|
| Display H2 | 22 | 500 | -0.005em | none |
| Section H3 | 16 | 500 | normal | none |
| Body | 13 | 400 | normal | none |
| Sub | 12 | 400 | normal | none |
| Field label | 11 | 400 | 0.08em | UPPERCASE |
| Section title | 11 | 600 | 0.18em | UPPERCASE |
| Eyebrow | 10 | 400 | 0.16-0.20em | UPPERCASE |
| Numerics | 11–13 | 400 | normal, tabular | none |

egui doesn't support letter-spacing natively — for the uppercase eyebrows, **insert spaces between characters at render time** (`"DEVICES"` → `"D E V I C E S"`) for the widest tracking, or accept tighter spacing. Alternatively use `RichText::small().monospace()` with extra `text_style` config.

### Spacing

Grid: `4 / 8 / 12 / 16 / 24`. Translate to `Style.spacing.item_spacing = (8, 6)` and `window_margin = (24, 20)`.

Density tweak (optional): comfortable row = 28px, compact = 24px.

## State Model

Suggested Rust types — adapt to existing `vibe-attack` state:

```rust
enum DaemonStatus { Running, Muted, Error(String), Disconnected }
enum TriggerMode  { PushToTalk, WakeWord, AlwaysListening }

struct AppState {
    status: DaemonStatus,
    mic_level: f32,           // 0.0..1.0, polled from daemon
    active_nav: NavId,
    profiles: Vec<Profile>,
    active_profile: ProfileId,
    // ...
}
```

The footer's `MIC` LED meter polls `mic_level` every ~200ms (egui `request_repaint_after`). The status pill animates: pulse for `Running`, blink for `Disconnected`. Both use simple sine-based alpha on a 1-2s period, not a dependency.

## Iconography

Two paths offered in the design:

1. **Bundled line SVGs** (recommended). 16×16, 1.5px stroke, single-color via `currentColor`. See `Icons.jsx` for the full set — paths are simple enough to hand-port to a Rust `egui::Shape` builder, or to bake into a single icon-font TTF using IcoMoon and load via egui's font system. Drop the assets in `assets/icons/`.
2. **egui unicode** (fallback). Maps in `Icons.jsx` under `UnicodeIcon`. Less precise but zero-asset.

Recommend shipping option 1 by default, with a build flag or runtime setting to switch to unicode for users with broken font rendering.

## Implementation Order

1. `theme.rs` — apply_theme, palette, fonts. Verify by toggling on top of the existing `config_app.rs`.
2. `widgets.rs` — `app_header`, `side_nav`, `status_footer`, `section_header`, `field_row`, `primary_button`, `led_meter`, `status_pill`. Test in isolation.
3. Refactor `config_app.rs` to use the new primitives. Smallest screen, validates the system.
4. Refactor `wizard.rs`. Adds the step indicator + dashed-border drop zone + animated capture state.
5. Refactor `pack_editor.rs`. Most complex — three-pane layout, drag-reorder, search.
6. Update `tray.rs` icons.
7. Capture before/after screenshots from the actual running app for the PR description.

## What's Out of Scope

- Replacing egui with another toolkit
- Web-based UI, Tauri, Electron
- Changes to audio, STT, VAD, or input subsystems
- Windows/macOS support

## Notes for the Implementer

- **The wall-of-error-text in the current UI is the worst pain point** — replacing it with a single Banner + Reconnect button is half the perceived improvement. Prioritize the daemon-disconnected story.
- **The PTT-key capture flow** in the wizard is intentionally a big drop-zone, not an inline input. Users on Linux often don't realize they need `input` group membership until something fails — surface that hint in the step body.
- **Don't lose the existing `ratatui` TUI** — its color choices should be derived from the same palette so the two surfaces feel like the same product.
- **Side rail expansion**: egui doesn't have CSS hover transitions. Either use `Response.hovered()` + a `lerp`'d width stored in state, or leave it as a click-to-toggle (less elegant but simpler). Don't block ship on the animation.
