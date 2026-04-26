# S03: Wizard UI panels — UAT

**Milestone:** M003
**Written:** 2026-04-26T00:19:37.903Z

# S03 UAT: Wizard UI Panels

## Prerequisites
- Machine with a display server (X11 or Wayland)
- vibe-attack-config built: `cargo build --bin vibe-attack-config --features gui`

## Test Cases

### 1. Wizard appears when config missing
Delete (or rename) `~/.config/vibe-attack/config.yaml`.
Launch `./target/debug/vibe-attack-config`.
Expected: "Step 1 of 4: Create config file" panel is shown.

### 2. Copy config button works
Click "Copy example config".
Expected: `~/.config/vibe-attack/config.yaml` is created; wizard advances to Step 2.

### 3. Install model panel renders
Expected: monospace curl command shown; Re-check button re-probes.

### 4. SetupUinput panel renders
Expected: modprobe and usermod commands shown in code blocks; yellow CachyOS note visible; Re-check button re-probes.

### 5. PTT key capture
On ConfigurePtt step: click "Listen for key", press a key.
Expected: key name (e.g. KEY_GRAVE) appears in config ptt.key; wizard advances to main app.

### 6. Wizard transitions to main app
After all four checks pass: "Vibe Attack" heading and profile/log view shown (no wizard).

