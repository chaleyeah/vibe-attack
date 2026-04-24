# Phase 4: Pack System + HD2 Bundle - Context

**Gathered:** 2026-04-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver a robust management system for macro "packs" and a complete Helldivers 2 stratagem bundle:

1. **Pack System**: Support for `.hdpack` files (Zip archives containing YAML and sounds).
2. **Profile Management**: Mechanism to store, load, and switch between multiple named profiles (packs).
3. **Built-in Editor**: An interactive TUI for users to edit macros and profile settings without touching raw YAML.
4. **HD2 Bundle**: A complete, data-driven pack covering all 80+ stratagems.
5. **Control Channel**: IPC mechanism to switch profiles at runtime.

New capabilities (GUI, AppImage, first-run wizard) are deferred to Phase 5.

</domain>

<decisions>
## Implementation Decisions

### Pack & Profile System (PACK-01, PACK-02, PACK-03)

- **D-01: Pack Format (`.hdpack`)**: A `.hdpack` file is a **Zip Archive** containing a `pack.yaml` (macro definitions) and a `sounds/` directory for optional audio feedback.
- **D-02: Profile Storage**: Profiles are stored as individual files in a dedicated `profiles/` directory (e.g., `~/.config/hd-linux-voice/profiles/`).
- **D-03: Import/Export**: The CLI provides `import` and `export` commands that handle the Zipping/Unzipping and checksum verification.
- **D-04: Category Grouping**: The `pack.yaml` structure supports grouping macros by category (e.g., "Offensive", "Supply") for better organization in the TUI/Editor.

### Control & Activation (ACT-03, PACK-04)

- **D-05: Runtime Switching**: The daemon listens on a **Unix Domain Socket** for control commands. The CLI sends a `SwitchProfile(name)` command to trigger a hot-reload.
- **D-06: Activation Mode Toggle**: The choice between PTT and Wake-word is a top-level field in `config.yaml`. While it persists across restarts, the daemon supports switching the "active" mode at runtime via the control channel.

### Built-in Editor (PACK-05)

- **D-07: Interactive TUI**: Implement a TUI (e.g., using `ratatui` or `cursive`) for the `edit` command. It allows browsing macros by category, editing key sequences, and adjusting thresholds.

### Helldivers 2 Bundle

- **D-08: Complete Coverage**: The bundled `hd2.hdpack` includes all 80+ current stratagems. Key sequences are validated against the current game version.

### Claude's Discretion

- Exact Unix socket protocol (JSON vs. Binary).
- TUI layout and color scheme (should feel "tactical/military" to match HD2).
- Checksum algorithm for pack verification.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Roadmap + requirements
- `.planning/ROADMAP.md` — Phase 4 goal + success criteria (80+ stratagems, import/export, TUI)
- `.planning/REQUIREMENTS.md` — Requirement definitions (ACT-03, STT-03, PACK-01..05)
- `.planning/PROJECT.md` — Product constraints: local-only, Wayland-first, AGPL-3.0

### Existing implementation
- `src/config.rs` — Existing `MacroConfig` and `KeyAction` schemas.
- `src/input/inject.rs` — Virtual device and key sequence injection logic.
- `src/pipeline/dispatcher.rs` — Phrase matching and macro triggering logic (Phase 3).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `MacroConfig` struct in `src/config.rs`: already supports `if_flag`, `set_flag`, `sound`, and `keys`.
- `rodio` backend (Phase 3): used for playing sounds from the `sounds/` dir in a pack.

### Established Patterns
- **YAML for data**: All configuration and macro definitions use YAML.
- **Fail-fast validation**: Packs are validated for schema correctness and file existence upon import/load.

### Integration Points
- `src/main.rs`: Needs to initialize the Control Channel (Unix socket) listener.
- `src/pipeline/coordinator.rs`: The registry/dispatcher must be swappable at runtime when a profile switch occurs.

</code_context>

<specifics>
## Specific Ideas

- The TUI should allow "Live Testing" of a macro (firing the keys) while editing.
- Sound files in a pack should be referenced relatively to the `sounds/` dir in the ZIP.

</specifics>

<deferred>
## Deferred Ideas

- **Cloud Sync**: Sharing packs via a central registry is deferred to v2.
- **Auto-Update**: Auto-updating the HD2 bundle from a remote source is deferred.

</deferred>
