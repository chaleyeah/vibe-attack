# AppImage Distribution Proofs

This directory contains structured transcripts proving that `vibe-attack-x86_64.AppImage` builds and runs on each supported Linux distribution.

## Directory Layout

```
docs/distribution-proofs/appimage/
├── README.md          ← this file
├── debian13/
│   └── transcript.md  ← Debian 13 proof transcript
├── ubuntu2604/
│   └── transcript.md  ← Ubuntu 26.04 proof transcript
├── fedora44/
│   └── transcript.md  ← Fedora 44 proof transcript
└── cachyos/
    └── transcript.md  ← CachyOS proof transcript
```

## Transcript Format

Each `transcript.md` begins with structured key-value fields (one per line), followed by optional free-form reproduction notes:

```
STATUS: <value>
DISTRO: <os-release PRETTY_NAME>
KERNEL: <uname -r output>
SIZE_BYTES: <AppImage size in bytes>
SHA256: <sha256 hash of AppImage>
EXIT_CODE: <exit code from ./vibe-attack-x86_64.AppImage --version>
VERSION_OUTPUT: <stdout+stderr from --version>
FAILURE_REASON: <human-readable failure description>   ← only present on non-ok status
```

## STATUS Values

| Value | Meaning |
|---|---|
| `ok` | AppImage built successfully, `--version` passed, size ≤ 50 MB |
| `skipped:tools-missing` | `linuxdeploy` or `appimagetool` absent on this host; structure is valid, full run deferred |
| `failed:build-script-nonzero` | `packaging/appimage/build.sh` exited non-zero |
| `failed:appimage-missing` | `build.sh` exited 0 but AppImage file not found |
| `failed:too-large` | AppImage exists but exceeds 50 MB limit |
| `failed:version-check-failed` | `--version` run failed (non-zero exit) |
| `pending VM run` | Transcript seeded; full run not yet executed on this distro |

Transcripts with `STATUS: pending VM run` or `STATUS: skipped:tools-missing` are **acceptable** — they preserve structural completeness so tests can assert field presence regardless of whether the real VM run has occurred. The full VM runs are completed in milestone S02.

## Generating a Transcript

Use `scripts/verify-appimage.sh` to capture a transcript:

```bash
bash scripts/verify-appimage.sh docs/distribution-proofs/appimage/<distro>/transcript.md
```

The script:
1. Detects `linuxdeploy` and `appimagetool`; emits `STATUS: skipped:tools-missing` if absent
2. Runs `packaging/appimage/build.sh`
3. Checks the produced AppImage is ≤ 50 MB
4. Runs `./vibe-attack-x86_64.AppImage --version`
5. Writes all transcript fields unconditionally (even on failure)

## Per-Distro Reproduction

### Debian 13

```bash
# System packages (mirrors release.yml)
sudo apt-get install -y libasound2-dev libclang-dev librsvg2-bin libfuse2 wget

# Install packaging tools
wget -q https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
wget -q https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
chmod +x linuxdeploy-x86_64.AppImage appimagetool-x86_64.AppImage
sudo mv linuxdeploy-x86_64.AppImage /usr/local/bin/linuxdeploy
sudo mv appimagetool-x86_64.AppImage /usr/local/bin/appimagetool

# Capture transcript
bash packaging/appimage/build.sh && bash scripts/verify-appimage.sh docs/distribution-proofs/appimage/debian13/transcript.md
```

### Ubuntu 26.04

```bash
# System packages
sudo apt-get install -y libasound2-dev libclang-dev librsvg2-bin libfuse2 wget

# Install packaging tools (same as above)
wget -q https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
wget -q https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
chmod +x linuxdeploy-x86_64.AppImage appimagetool-x86_64.AppImage
sudo mv linuxdeploy-x86_64.AppImage /usr/local/bin/linuxdeploy
sudo mv appimagetool-x86_64.AppImage /usr/local/bin/appimagetool

# Capture transcript
bash packaging/appimage/build.sh && bash scripts/verify-appimage.sh docs/distribution-proofs/appimage/ubuntu2604/transcript.md
```

### Fedora 44

```bash
# System packages
sudo dnf install -y alsa-lib-devel clang-devel librsvg2-tools fuse-libs wget

# Install packaging tools (same as above)
wget -q https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
wget -q https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
chmod +x linuxdeploy-x86_64.AppImage appimagetool-x86_64.AppImage
sudo mv linuxdeploy-x86_64.AppImage /usr/local/bin/linuxdeploy
sudo mv appimagetool-x86_64.AppImage /usr/local/bin/appimagetool

# Capture transcript
bash packaging/appimage/build.sh && bash scripts/verify-appimage.sh docs/distribution-proofs/appimage/fedora44/transcript.md
```

### CachyOS

```bash
# System packages
sudo pacman -Sy --noconfirm alsa-lib clang librsvg fuse2 wget

# Install packaging tools (same as above)
wget -q https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
wget -q https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
chmod +x linuxdeploy-x86_64.AppImage appimagetool-x86_64.AppImage
sudo mv linuxdeploy-x86_64.AppImage /usr/local/bin/linuxdeploy
sudo mv appimagetool-x86_64.AppImage /usr/local/bin/appimagetool

# Capture transcript
bash packaging/appimage/build.sh && bash scripts/verify-appimage.sh docs/distribution-proofs/appimage/cachyos/transcript.md
```

## Policy: Pending Transcripts

Transcripts with `STATUS: pending VM run` count as structural proof — all metadata fields are present with `pending` placeholders, allowing `tests/distribution_proofs.rs` to assert field presence. The policy is that full `STATUS: ok` runs must be completed before the milestone is closed, but intermediate tasks may commit pending-state transcripts.
