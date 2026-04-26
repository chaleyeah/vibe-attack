# S04: Main config app wired up — UAT

**Milestone:** M003
**Written:** 2026-04-26T00:22:32.227Z

# S04 UAT: Main Config App Wired Up

## Prerequisites
- Machine with display server, audio device, and profiles dir populated
- `cargo build --bin vibe-attack-config --features gui`

## Test Cases

### 1. Profiles listed
Create `~/.config/vibe-attack/profiles/hd2.yaml`.
Launch app after wizard completes.
Expected: "Profiles (1)" label shows; "hd2" appears in the list.

### 2. Mic level responds to input
Expected: ProgressBar moves when you speak into the mic.

### 3. No-device graceful
Launch with no audio input device configured.
Expected: "no device" label shown in place of ProgressBar; no panic or crash.

### 4. Log feed
Expected: log lines appear in the ScrollArea; "probing first-run environment" visible from probe::run(); area auto-scrolls to bottom.

### 5. Setup transition
Start with config missing, complete wizard steps.
Expected: after final step passes, mic starts, profiles load, main config view appears.

