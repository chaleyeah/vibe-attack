# Final Distribution UAT Proofs

This directory contains structured transcripts proving the complete end-user loop on each supported Linux distribution: download AppImage → run → first-run wizard completes → stratagem fires by voice.

## Directory Layout

```
docs/distribution-proofs/final/
├── README.md          ← this file
├── debian13/
│   └── transcript.md  ← Debian 13 final UAT transcript
├── ubuntu2604/
│   └── transcript.md  ← Ubuntu 26.04 final UAT transcript
├── fedora44/
│   └── transcript.md  ← Fedora 44 final UAT transcript
└── cachyos/
    └── transcript.md  ← CachyOS final UAT transcript
```

## Transcript Format

Each `transcript.md` begins with structured key-value fields (one per line), followed by a `## Reproduction Notes` section:

```
STATUS: <value>
DISTRO: <os-release PRETTY_NAME>
KERNEL: <uname -r output>
APPIMAGE_VERSION: <stdout from ./vibe-attack-x86_64.AppImage --version>
APPIMAGE_SIZE_BYTES: <size in bytes from stat -c %s>
WIZARD_COMPLETED: <yes|no|pending>
STRATAGEM_FIRED: <yes|no|pending>
INSTALL_METHOD: appimage
```

## STATUS Values

| Value | Meaning |
|---|---|
| `ok` | Full end-to-end loop completed; wizard finished; stratagem fired by voice |
| `pending VM run` | Transcript seeded; full run not yet executed on this distro |
| `failed:<reason>` | Loop failed; reason identifies the failing step (e.g. `failed:wizard-stalled`, `failed:no-voice-trigger`) |

Transcripts with `STATUS: pending VM run` are **acceptable** — they preserve structural completeness so tests can assert field presence regardless of whether the real VM run has occurred. Human operators convert `pending VM run` → `ok` after executing the full loop on a real VM.

## Field Reference

| Field | How to obtain |
|---|---|
| `DISTRO` | `grep PRETTY_NAME /etc/os-release \| cut -d= -f2 \| tr -d '"'` |
| `KERNEL` | `uname -r` |
| `APPIMAGE_VERSION` | `./vibe-attack-x86_64.AppImage --version` |
| `APPIMAGE_SIZE_BYTES` | `stat -c %s vibe-attack-x86_64.AppImage` |
| `WIZARD_COMPLETED` | `yes` if the wizard ran all steps and transitioned to the main config screen |
| `STRATAGEM_FIRED` | `yes` if at least one stratagem was fired by voice after wizard completion |
| `INSTALL_METHOD` | Always `appimage` for this proof set |

## Per-Distro Reproduction

### Debian 13

```bash
# Install FUSE2 dependency required by AppImage runtime
sudo apt-get install -y libfuse2

# Download AppImage from the GitHub Releases page
wget https://github.com/<owner>/vibe-attack/releases/latest/download/vibe-attack-x86_64.AppImage
chmod +x vibe-attack-x86_64.AppImage

# Verify it launches
./vibe-attack-x86_64.AppImage --version

# Run the full wizard end-to-end
./vibe-attack-x86_64.AppImage
# Step through all wizard steps; fire a stratagem by voice after wizard completes

# Capture field values
DISTRO=$(grep PRETTY_NAME /etc/os-release | cut -d= -f2 | tr -d '"')
KERNEL=$(uname -r)
APPIMAGE_VERSION=$(./vibe-attack-x86_64.AppImage --version 2>&1)
APPIMAGE_SIZE_BYTES=$(stat -c %s vibe-attack-x86_64.AppImage)
# Fill transcript.md with observed results
```

### Ubuntu 26.04

```bash
# Install FUSE2 dependency required by AppImage runtime
sudo apt-get install -y libfuse2

# Download AppImage from the GitHub Releases page
wget https://github.com/<owner>/vibe-attack/releases/latest/download/vibe-attack-x86_64.AppImage
chmod +x vibe-attack-x86_64.AppImage

# Verify it launches
./vibe-attack-x86_64.AppImage --version

# Run the full wizard end-to-end
./vibe-attack-x86_64.AppImage
# Step through all wizard steps; fire a stratagem by voice after wizard completes

# Capture field values (same commands as Debian 13 above)
```

### Fedora 44

```bash
# Install FUSE2 dependency required by AppImage runtime
sudo dnf install -y fuse-libs

# Download AppImage from the GitHub Releases page
wget https://github.com/<owner>/vibe-attack/releases/latest/download/vibe-attack-x86_64.AppImage
chmod +x vibe-attack-x86_64.AppImage

# Verify it launches
./vibe-attack-x86_64.AppImage --version

# Run the full wizard end-to-end
./vibe-attack-x86_64.AppImage
# Step through all wizard steps; fire a stratagem by voice after wizard completes

# Capture field values (same commands as Debian 13 above)
```

### CachyOS

```bash
# Install FUSE2 dependency required by AppImage runtime
sudo pacman -Sy --noconfirm fuse2

# Download AppImage from the GitHub Releases page
wget https://github.com/<owner>/vibe-attack/releases/latest/download/vibe-attack-x86_64.AppImage
chmod +x vibe-attack-x86_64.AppImage

# Verify it launches
./vibe-attack-x86_64.AppImage --version

# Run the full wizard end-to-end
./vibe-attack-x86_64.AppImage
# Step through all wizard steps; fire a stratagem by voice after wizard completes

# Capture field values (same commands as Debian 13 above)
```

## Policy: Pending Transcripts

Transcripts with `STATUS: pending VM run` count as structural proof — all 8 metadata fields are present with `pending` placeholders (except `INSTALL_METHOD` which is always `appimage`), allowing `tests/distribution_proofs.rs` to assert field presence. Full `STATUS: ok` runs must be completed by a human operator with real VMs before the milestone is closed.
