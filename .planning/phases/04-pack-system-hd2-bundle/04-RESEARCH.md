# Phase 4 Research: Pack System + HD2 Bundle

## 1. ZIP Management (PACK-01, PACK-03)

### Crate Selection: `zip`
- **Choice**: [`zip`](https://crates.io/crates/zip)
- **Rationale**: Mature, feature-complete, and supports both reading and writing ZIP archives. Synchronous execution is acceptable for import/export operations.
- **Licensing**: MIT (compatible).

### Pack Structure (.hdpack)
- A `.hdpack` file is a ZIP containing:
    - `pack.yaml`: Metadata, categories, and macro definitions.
    - `sounds/`: Directory containing audio files referenced in `pack.yaml`.
- **Validation**: Check for the presence of `pack.yaml` on import. Optionally include a `checksum.sha256` file for integrity.

## 2. Unix Domain Socket Control Channel (ACT-03)

### Implementation: `tokio::net::UnixListener`
- **Socket Path**: Use `xdg::BaseDirectories::get_runtime_file("hd-linux-voice.sock")`. Usually maps to `/run/user/$UID/hd-linux-voice.sock`.
- **Protocol**: Line-delimited JSON.
    - Request: `{"cmd": "switch_profile", "args": {"name": "hd2"}}`
    - Response: `{"status": "ok"}` or `{"status": "error", "message": "..."}`
- **Security**: UDS permissions inherit from the user, so only the user (and root) can send commands to the daemon.

## 3. TUI Editor (PACK-05)

### Crate Selection: `ratatui`
- **Choice**: [`ratatui`](https://crates.io/crates/ratatui)
- **Rationale**: The gold standard for Rust TUIs. Supports complex layouts, themes, and mouse interaction.
- **Theme**: "Tactical/Military" using:
    - Colors: Bright Yellow (#FFFF00) on Deep Gray/Black.
    - Borders: Heavy/Double box-drawing characters.
    - Typography: Monospace (fixed by terminal).

### Functionality
- **List View**: Browse categories and macros.
- **Edit View**: Modify phrases, key sequences (with dwell/gap overrides), and sound paths.
- **Live Test**: Send a command via the UDS to the running daemon to "preview" the macro.

## 4. Category Grouping Schema

### YAML Structure
```yaml
version: "1.0"
name: "Helldivers 2"
author: "Super Earth Command"
categories:
  - name: "Offensive Stratagems"
    macros:
      - name: "Eagle Airstrike"
        phrase: "eagle airstrike"
        keys:
          - { key: "KEY_UP", dwell_ms: 50 }
          - { key: "KEY_RIGHT" }
          - { key: "KEY_DOWN" }
          - { key: "KEY_RIGHT" }
```
- The daemon flattens this into a `HashMap<String, Macro>` at load time.

## 5. Validation Architecture (Nyquist Dimension 8)

### Integration Tests
1. **Import/Export Cycle**: Export a profile to `.hdpack`, delete original, import back, verify identity.
2. **UDS Command Flow**: Start daemon, send `switch_profile` command, verify new phrases are matched.
3. **Macro Persistence**: Edit a macro in the TUI, save, verify `pack.yaml` is updated correctly.

### Mocking
- Mock the `VirtualDevice` and `Rodio` output to verify "Live Test" commands in the TUI without side effects.

## 6. Security Considerations
- **Path Traversal**: Ensure `.hdpack` extraction doesn't allow writing files outside the `profiles/` directory (check for `..` in paths).
- **Socket Permissions**: Explicitly set UDS permissions to `0600` (user only).
