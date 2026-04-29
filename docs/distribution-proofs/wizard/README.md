# Wizard Distribution Proofs

This directory contains structured transcripts proving that the first-run wizard in `vibe-attack-config` runs end-to-end on each supported Linux distribution, covering four UAT scenarios per distro.

## Directory Layout

```
docs/distribution-proofs/wizard/
├── README.md          ← this file
├── debian13/
│   └── transcript.md  ← Debian 13 wizard UAT transcript
├── ubuntu2604/
│   └── transcript.md  ← Ubuntu 26.04 wizard UAT transcript
├── fedora44/
│   └── transcript.md  ← Fedora 44 wizard UAT transcript
└── cachyos/
    └── transcript.md  ← CachyOS wizard UAT transcript
```

## Transcript Format

Each `transcript.md` begins with structured key-value fields (one per line), followed by a free-form `## Reproduction Notes` section:

```
STATUS: <value>
DISTRO: <os-release PRETTY_NAME>
KERNEL: <uname -r output>
BINARY: vibe-attack-config
BINARY_VERSION: <version string from --version>
SCENARIO_A: <ok|failed|pending>
SCENARIO_B: <ok|failed|pending>
SCENARIO_C: <ok|failed|pending>
SCENARIO_D: <ok|failed|pending>
STRATAGEM_FIRED: <yes|no|pending>
```

## STATUS Values

| Value | Meaning |
|---|---|
| `ok` | All four scenarios passed on this distro |
| `pending VM run` | Transcript seeded; full run not yet executed on this distro |
| `failed:scenario-A` | Scenario A (fresh install) failed |
| `failed:scenario-B` | Scenario B (model pre-placed) failed |
| `failed:scenario-C` | Scenario C (relaunch skips wizard) failed |
| `failed:scenario-D` | Scenario D (--skip-wizard flag) failed |

Transcripts with `STATUS: pending VM run` are **acceptable** — they preserve structural completeness so tests can assert field presence regardless of whether the real VM run has occurred. Full `STATUS: ok` runs are completed in milestone S02.

## UAT Scenarios

### Scenario A — Fresh Install

Start from a completely clean state (no config, no model). The wizard must run all four steps (CreateConfig → InstallModel → SetupUinput → ConfigurePtt) and then transition to the main config screen.

```bash
rm -rf ~/.config/vibe-attack ~/.local/share/vibe-attack
vibe-attack-config
# Wizard shows step 1; click "Copy example config"
# Step 2: download model (or place stub file at model path)
# Step 3: modprobe uinput + usermod -aG input $USER (or confirm already accessible)
# Step 4: press PTT key to capture
# Wizard clears; main config screen appears
# Fire at least one stratagem by voice to confirm mic pipeline is live
```

Expected: `SCENARIO_A: ok`, `STRATAGEM_FIRED: yes`

### Scenario B — Partial State (Model Pre-Placed)

Place the model file before launch. The wizard should skip step 2 and show only the remaining steps.

```bash
mkdir -p ~/.local/share/vibe-attack/models/whisper
cp /path/to/ggml-tiny.en.bin ~/.local/share/vibe-attack/models/whisper/ggml-tiny.en.bin
rm -rf ~/.config/vibe-attack
vibe-attack-config
# Wizard shows step 1 (CreateConfig), then jumps past step 2 (model already present)
```

Expected: `SCENARIO_B: ok`

### Scenario C — Relaunch (Wizard Skipped)

After completing Scenario A, relaunch. The wizard must NOT appear; the main config screen must be shown immediately.

```bash
# (After Scenario A is complete)
vibe-attack-config
# Main config screen appears — NO wizard shown
```

Expected: `SCENARIO_C: ok`

### Scenario D — --skip-wizard Flag

Launch with `--skip-wizard`. The main config screen must appear regardless of disk state.

```bash
rm -rf ~/.config/vibe-attack ~/.local/share/vibe-attack
vibe-attack-config --skip-wizard
# Main config screen appears immediately — NO wizard shown
```

Expected: `SCENARIO_D: ok`

## Per-Distro Reproduction

### Debian 13

```bash
# System packages
sudo apt-get install -y libasound2-dev libclang-dev librsvg2-bin libfuse2 wget

# Build the binary
cargo build --release --bin vibe-attack-config
export PATH="$PWD/target/release:$PATH"

# Run UAT scenarios A–D per the steps above
# Fill in transcript.md fields with observed results
```

### Ubuntu 26.04

```bash
# System packages
sudo apt-get install -y libasound2-dev libclang-dev librsvg2-bin libfuse2 wget

# Build the binary
cargo build --release --bin vibe-attack-config
export PATH="$PWD/target/release:$PATH"

# Run UAT scenarios A–D per the steps above
# Fill in transcript.md fields with observed results
```

### Fedora 44

```bash
# System packages
sudo dnf install -y alsa-lib-devel clang-devel librsvg2-tools fuse-libs wget

# Build the binary
cargo build --release --bin vibe-attack-config
export PATH="$PWD/target/release:$PATH"

# Run UAT scenarios A–D per the steps above
# Fill in transcript.md fields with observed results
```

### CachyOS

```bash
# System packages
sudo pacman -Sy --noconfirm alsa-lib clang librsvg fuse2 wget

# Build the binary
cargo build --release --bin vibe-attack-config
export PATH="$PWD/target/release:$PATH"

# Run UAT scenarios A–D per the steps above
# Fill in transcript.md fields with observed results
```

## Policy: Pending Transcripts

Transcripts with `STATUS: pending VM run` count as structural proof — all metadata fields are present with `pending` placeholders, allowing `tests/distribution_proofs.rs` to assert field presence. Full `STATUS: ok` runs must be completed before the milestone is closed, but intermediate tasks may commit pending-state transcripts.

## Common Pitfalls

- **polkit agent required for pkexec** — Scenario A step 3 (modprobe + usermod) calls `pkexec`. On Wayland desktops without a running polkit agent, the dialog will not appear. Run `ps aux | grep polkit` to confirm the agent is running.
- **`input` group membership is session-scoped** — `usermod -aG input $USER` does not take effect until re-login. After step 3 completes, log out and back in (or run `newgrp input`) before re-testing uinput access.
- **HuggingFace download redirect** — `ureq` must follow the CDN 302 redirect. Verify with `curl -L <MODEL_URL>` before UAT if network access is restricted.
- **evdev device selection** — PTT capture in step 4 uses the first keyboard device supporting KEY_A. On machines with multiple keyboards or HID dongles, it may not capture from the intended device. Use the manual entry fallback if capture hangs.
- **Scenario C `setup_just_completed` guard** — confirm the `was_incomplete && is_setup_complete()` predicate is never spuriously true on relaunch (it must not be, since probe runs at construction time and the app starts complete).
