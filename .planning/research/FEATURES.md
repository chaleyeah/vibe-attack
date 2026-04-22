# Feature Landscape: Linux Voice-Macro / Gaming Voice Control

**Domain:** Voice-driven keyboard/mouse macro application for Linux gaming (Helldivers 2 / VoiceAttack-class)
**Researched:** 2026-04-21
**Reference Implementations:** VoiceAttack (Windows, gold standard), voice-commander (Python/MIT), hyprwhspr, Vocalinux

---

## Table Stakes

Features users expect from day one. Missing any of these = product is unusable or users immediately drop it.

| Feature | Why Expected | Complexity | v1? | Notes |
|---------|--------------|------------|-----|-------|
| **Push-to-talk (PTT) activation** | Without gating, open mic fires macros constantly; PTT is the simplest gate | Low | YES | Configurable hotkey; hold-to-listen semantics |
| **Wake-word / listen-toggle gate** | Alternative to PTT — say a keyword, system starts listening | Medium | YES | Must be decoupled from PTT; both modes needed |
| **Local (offline) speech recognition** | Privacy + latency + offline play; cloud dependency = deal-breaker for gaming | High | YES | Whisper.cpp or Vosk; model hot in RAM for sub-1s latency |
| **Phrase → key-sequence macro execution** | Core value: say word → emit key events | Medium | YES | Must deliver key press + release + timed sequences |
| **Wayland input injection** | Project is Wayland-first; X11 xdotool doesn't work on Wayland | High | YES | ydotool (uinput) or libei/XDG RemoteDesktop portal |
| **Configurable key-sequence timing** | HD2 stratagem entry requires fine timing; wrong delays = missed inputs | Low | YES | Per-command inter-key delay (ms-level control) |
| **Profile concept (named command sets)** | Users want different macro sets per game; profiles are universal in this space | Low | YES | Load/switch profiles; one active at a time |
| **Recognition log / feedback UI** | Users can't tune commands they can't see; log of "heard: X → fired: Y" is essential | Low | YES | Even a terminal log satisfies v1 |
| **Mic level indicator** | Must know if mic is live; otherwise debugging is impossible | Low | YES | Simple RMS meter or visual indicator |
| **Importable/exportable command packs** | Sharing HD2 stratagem packs is the killer v1 use case; manual re-entry is intolerable | Medium | YES | JSON/TOML pack format; import from file |

---

## Differentiators

Features not universally expected but strongly valued — competitive moats vs a shell script or a wine-VoiceAttack setup.

| Feature | Value Proposition | Complexity | v1? | Notes |
|---------|-------------------|------------|-----|-------|
| **Data-driven stratagem pack (Helldivers 2)** | Bundled, maintained, game-accurate pack with all ~80+ stratagems; instantly useful | Medium | YES — flagship | Separate data file from engine; versioned; community-updatable |
| **Built-in phrase editor** | Edit phrases + key sequences without touching JSON; non-developers can onboard | Medium | YES (minimal) | Can be CLI-based in v1; GUI is v2 |
| **Dynamic / optional phrase segments** | VA's `[Orbital;] EMS [Strike;]` syntax: required + optional words → fewer commands to author | Medium | v2 | Massively reduces authoring burden; complex to parse |
| **Confidence threshold control** | Tune false-positive vs false-negative tradeoff per command or globally | Low | YES | Essential for gaming where false triggers cause wipes |
| **Command chaining / sub-commands** | `open_strat_menu` → `key_sequence` → `close_strat_menu` as reusable sub-calls | Medium | YES (minimal) | HD2 profiles use this pattern heavily |
| **Wayland-native (no Wine/Proton layer)** | VoiceAttack via Wine is flaky (2489+ downloads of Wine workaround gists); native = reliability | High | YES — core | Primary technical differentiator vs running VA on Wine |
| **AGPL-licensed, community extensible** | No vendor lock-in; hobbyist community can fork/extend stratagem packs | Low | YES | License is decided; matters for pack ecosystem |
| **Multiple speech engine backends (pluggable)** | Whisper.cpp for GPU users, Vosk for CPU-only; swap without reconfiguring macros | High | v2 | v1 can ship one engine; architecture should allow swapping |
| **Conditions / if-then logic** | Run macro only if X is true (e.g., only fire if game window focused) | High | v2 | VA condition builder; needed for power users |
| **Variables and state** | Counters, toggles, stored values across macro executions | High | v2 | VA has 6 variable types; gaming use: track cooldowns, toggle modes |
| **Auto profile switching (window detection)** | Switch to HD2 profile when game window gains focus | Medium | v2 | VA does process/window-title matching |
| **Text-to-Speech confirmation** | Say "Eagle Airstrike confirmed" after macro fires — positional audio feedback | Low | v2 | Useful for HD2; low engineering cost once audio pipeline exists |
| **Soundboard / audio feedback** | Play a clip on recognition (e.g., HD2 ship radio crackle) | Low | v2 | Community delight feature |

---

## Anti-Features

Things to explicitly NOT build in v1 — scope traps that burn time without user value.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Full graphical profile editor (v1)** | GUI takes 3-5x longer than a minimal CLI/TUI; delays shipping the actual macro engine | Ship JSON editor + terminal preview; GUI in v2 |
| **Cloud speech recognition as primary path** | Privacy violation, adds latency, breaks offline play, dependency risk | Keep local-first; cloud = optional plugin later |
| **Windows/macOS client** | Doubles platform complexity; dilutes Linux focus | Linux-only in v1 per PROJECT.md |
| **Plugin/extension API (v1)** | VA's plugin system took years to stabilize; premature abstraction kills momentum | Hard-code extension points; formalize API in v2 |
| **Natural language / conversational AI integration** | "Hey computer, use the one that kills bugs" is cool but unacceptably slow/flaky for live gaming | Exact phrase matching only in v1 |
| **LLM-powered phrase disambiguation** | Latency is 300–2000ms; stratagem timing is <100ms window | Deterministic matching only |
| **Auto-update / OTA pack fetching** | Needs signing, CDN, rollback — ops complexity out of scope | Ship pack files; users download updated packs manually |
| **Voice training / personal model fine-tuning** | Whisper already generalizes well; training pipeline is a separate product | Use pretrained models; expose confidence tuning |
| **Gamepad / joystick voice-combo triggers** | Joystick + voice combos are a niche; adds input device complexity | Keyboard PTT is sufficient for v1 |

---

## Helldivers 2 Stratagem Pack — Specific Requirements

This section defines what the v1 bundled HD2 pack must look like. The pack is the flagship differentiator.

### Stratagem Coverage

| Category | Count (approx.) | Examples | Notes |
|----------|-----------------|----------|-------|
| Orbital strikes | ~10 | Orbital Laser, Orbital 380mm Barrage, Orbital Railcannon Strike | Codes change rarely |
| Eagle strikes | ~8 | Eagle Airstrike, Eagle Napalm, Eagle Cluster Bomb | "Eagle" prefix ambiguous with "Orbital" — needs distinct phrase design |
| Support weapons | ~15 | Autocannon, Railgun, Recoilless Rifle, Flamethrower | Many share prefix words (Rifle, Gun) — require unambiguous phrases |
| Sentries / emplacements | ~8 | Gatling Sentry, Mortar, Tesla Tower | |
| Backpacks | ~5 | Jetpack, Shield Generator, Supply Pack | |
| Vehicles | ~3 | Hellpod, Exosuit variants | Added post-launch |
| Mission support | ~4 | Resupply, Reinforce, SOS Beacon, SEAF Artillery | Highest frequency — must be fastest |

**Total: ~80+ stratagems** (game continues adding via patches; data file must survive engine updates).

### Pack Data Format Requirements

- **Machine-readable and human-editable** — JSON or TOML; not binary
- **Versioned** — `pack_version` + `game_version` fields; stale pack warnings on load
- **One stratagem per entry** with: `id`, `phrase[]` (multiple spoken variants), `key_sequence[]`, `inter_key_delay_ms`, `category`, `description`
- **Community-patchable** — users can override a single entry without replacing the whole file
- **Conflict detection** — tool should warn when two phrases share a prefix and could be ambiguous

### Key Sequence Implementation Notes (from VA community research)

VoiceAttack forum experience (HD2-specific):
1. **PRESS binding > HOLD binding** — Game's "press" mode for stratagem menu open is faster and more consistent than "hold"; macros using press require only one key event, not hold+release
2. **Arrow key remapping required** — Stratagem directionals must be remapped to arrow keys in-game for reliable macro injection; game's default WASD directionals interfere with movement input
3. **Quick Input method preferred** — Sending `[arrowD][arrowD][arrowU][arrowR]` as a sequence is faster and easier to maintain than calling individual sub-commands per arrow press
4. **Sub-10ms inter-key delays** work reliably once using PRESS mode; HOLD mode needed ~50-100ms hold durations
5. **Ambiguous phrase pairs to handle:** Eagle Smoke Strike vs Orbital Smoke Strike; Eagle vs Orbital prefixes on shared weapon names — phrases need required discriminating words

### Phrase Authoring Approach

| Approach | Description | HD2 Suitability |
|----------|-------------|-----------------|
| Exact phrase | "Orbital Railcannon Strike" | Safe, no ambiguity risk |
| Optional segments `[word;]` | Optional prefix/suffix words | Reduces authoring count; requires disambiguation pass |
| Required keyword only | "Railcannon" | Risky — "rail" sounds similar to other words |

**Recommendation for v1:** Exact phrases only — simpler engine, safer for live play. Optional segments in v2 when confidence tuning exists.

---

## Feature Dependencies

```
PTT / wake-word gate
    └─► Local speech recognition (engine must be running)
            └─► Phrase matching
                    └─► Key-sequence execution
                            └─► Wayland input injection

Profile concept
    └─► Pack import/export (a pack is a profile)
            └─► HD2 stratagem pack (a pack instance)

Command chaining / sub-commands
    └─► Conditions (conditions gate sub-command execution)
            └─► Variables (conditions read variables)

Recognition log
    └─► Confidence threshold control (log shows scores, threshold applied)
```

---

## MVP Recommendation (v1 Scope)

**Ship these — core loop is not working without them:**
1. Local Whisper.cpp speech recognition, model hot in RAM
2. PTT activation (configurable hotkey)
3. Exact-phrase → key-sequence macro execution
4. Wayland input injection (ydotool/uinput path; libei as stretch)
5. JSON stratagem pack format + bundled HD2 pack (all ~80 stratagems)
6. Confidence threshold (global, not per-command)
7. Profile load/switch
8. Recognition log (terminal output acceptable)
9. Configurable inter-key timing per command
10. Pack import from file

**Defer to v2 (do not block v1):**
- Dynamic/optional phrase segments
- Per-command confidence override
- Conditions and variables
- Auto profile switching by window
- TTS/audio confirmation
- GUI editor
- Multiple STT engine backends
- Soundboard

---

## Sources

- VoiceAttack Help PDF (voiceattack.com/VoiceAttackHelp.pdf) — HIGH confidence; official docs
- VoiceAttack HD2 community profile thread (forum.voiceattack.com/topic=4733) — HIGH confidence; real user experience
- voice-commander PyPI/GitHub (spyoungtech/voice-commander) — MEDIUM confidence; codebase inspection
- ydotool GitHub (ReimuNotMoe/ydotool) + libei (libinput.pages.freedesktop.org) — HIGH confidence; official sources
- hyprwhspr.com / Vocalinux STT comparison — MEDIUM confidence; current 2026 sources
- Helldivers 2 stratagem lists (divers.gg, thegamer.com) — MEDIUM confidence; game wikis (patch-sensitive)
- WebSearch — LOW confidence where not corroborated
