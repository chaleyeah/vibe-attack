# hd-linux-voice

hd-linux-voice is an open-source voice-macro daemon for Helldivers 2 on Linux. Hold a push-to-talk key, speak a stratagem name, and the daemon injects the keystrokes automatically — no second monitor, no proprietary tools.

## Features

- **Stratagem Automation**: Trigger all 80+ Helldivers 2 stratagems with simple voice commands.
- **Profile Management**: Import and export macro profiles using a simple `.hdpack` zip format.
- **Built-in Editor**: Interactive TUI to create and edit macros without touching YAML files.
- **Sound System**: Integrated audio feedback using `rodio`, supporting custom sounds per macro.
- **Fail-Safe Design**: Built-in delays and double-tap detection to prevent accidental activations.

## Installation

### Prerequisites

- Linux (Debian/Ubuntu or Arch-based recommended)
- Rust stable toolchain ([rustup.rs](https://rustup.rs))
- A working microphone

### System Dependencies

**Debian / Ubuntu:**
```bash
sudo apt-get install -y \
  build-essential \
  libasound2-dev \
  pkg-config
```

**Arch / Manjaro:**
```bash
sudo pacman -S base-devel alsa-lib pkg-config
```

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### uinput / evdev Permissions

The daemon injects keypresses via `/dev/uinput`. See [docs/uinput-setup.md](docs/uinput-setup.md) for permission setup instructions.

### Whisper Model

Speech-to-text requires a Whisper model file. Models are **not** downloaded automatically — you must place the `.gguf` file yourself and point to it in `config.yaml` under `stt.model_path`.

### Clone and Build

```bash
git clone https://github.com/chaleyeah/hd-linux-voice.git
cd hd-linux-voice
cargo build --release
```

The binary will be at `./target/release/hd-linux-voice`.

## Usage

Running with no subcommand starts the daemon:

```bash
./target/release/hd-linux-voice
```

- **stdout** emits machine-readable JSONL transcripts (one JSON object per utterance).
- **stderr** receives all log output.

### Flags

| Flag | Description |
|------|-------------|
| `-v` | Enable DEBUG logging |
| `-vv` | Enable TRACE logging |
| `--config FILE` | Use a specific config file (default: `$XDG_CONFIG_HOME/hd-linux-voice/config.yaml`) |
| `--list-devices` | Print available audio input devices and exit |

### Subcommands

| Command | Description |
|---------|-------------|
| `ping` | Check if a running daemon is alive |
| `switch <name>` | Switch the active macro pack/profile |
| `test <name>` | Execute a specific macro immediately (for testing) |
| `import <file>` | Import a `.hdpack` file |
| `export <name> [output]` | Export the current profile to a `.hdpack` file |
| `edit` | Open the interactive TUI editor |

### Config File Location

```
~/.config/hd-linux-voice/config.yaml
```

The `XDG_CONFIG_HOME` environment variable overrides the base directory. Copy `config.example.yaml` from the repo as a starting point.

## Configuration

See [docs/configuration.md](docs/configuration.md) for a full reference of all config options, including push-to-talk key binding, audio device selection, VAD thresholds, and STT model path.

## Troubleshooting

See [docs/troubleshooting.md](docs/troubleshooting.md) for common issues including uinput permission errors, audio device problems, and Whisper model setup.

## License

AGPL-3.0-only — see [LICENSE](LICENSE) for details.
