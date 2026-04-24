# vibe-attack

vibe-attack is an open-source, community-driven voice command system designed specifically for Helldivers 2 on Linux. It allows players to execute complex stratagem sequences using natural voice commands, enhancing the tactical experience without requiring a second monitor or complex third-party tools.

## Features

- **Stratagem Automation**: Trigger all 80+ Helldivers 2 stratagems with simple voice commands.
- **Profile Management**: Import and export macro profiles (including custom ones) using a simple `.hdpack` zip format.
- **Built-in Editor**: An interactive Text User Interface (TUI) to create and edit macros without touching YAML files.
- **Sound System**: Integrated audio feedback using `rodio`, supporting custom sounds per macro.
- **Fail-Safe Design**: Built-in delays and double-tap detection to prevent accidental activations in the heat of battle.

## Installation

### Prerequisites
- **Linux Distribution**: Debian/Ubuntu or Arch Linux based systems.
- **Audio Devices**: A working microphone (input) and ALSA loopback device (output).
- **Rust**: Version 1.70+ (latest stable recommended)

### System Dependencies

**Debian / Ubuntu:**
```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential \
  libasound2-dev \
  libportaudio2 \
  libportaudiocpp0 \
  pkg-config
```

**Arch / Manjaro:**
```bash
sudo pacman -Syu
sudo pacman -S base-devel portaudio
```

### Install Rust

If you haven't installed Rust yet:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

## Building

### Quick Start

1. **Clone the repository**:
   ```bash
   git clone https://github.com/chaleyeah/hd-linux-voice.git
   cd hd-linux-voice
   ```

2. **Build the project**:
   ```bash
   cargo build --release
   ```

The compiled binary will be available at `./target/release/hd-linux-voice`.

### Build Options

**Debug build** (faster compilation, slower runtime):
```bash
cargo build
./target/debug/hd-linux-voice
```

**Release build** (slower compilation, optimized runtime):
```bash
cargo build --release
./target/release/hd-linux-voice
```

### Running Tests

Run the full test suite:
```bash
cargo test
```

Run tests with output:
```bash
cargo test -- --nocapture
```

Run specific test:
```bash
cargo test test_name
```

### Troubleshooting Build Issues

**ALSA dependency conflict**: If you see an error about `alsa-sys` version conflict, this is due to incompatible versions of `cpal` in the dependency tree. This requires updating the `Cargo.toml` to use compatible versions:

- Update `rodio` to a version compatible with `cpal v0.17.3`, or
- Downgrade `cpal` to match `rodio`'s requirements

This is a pre-existing issue in the project that needs to be resolved before tests can run.

## Running the Application

### Quick Start

```bash
./target/release/hd-linux-voice run
```

### Available Commands

- **Run**: Starts the voice recognition service in the foreground.
  ```bash
  ./target/release/hd-linux-voice run
  ```

- **Import**: Imports a `.hdpack` file into the profiles directory.
  ```bash
  ./target/release/hd-linux-voice import path/to/pack.hdpack
  ```

- **Export**: Exports a profile to a `.hdpack` file.
  ```bash
  ./target/release/hd-linux-voice export profile-name path/to/output.hdpack
  ```

- **Edit**: Opens the interactive TUI to edit macros.
  ```bash
  ./target/release/hd-linux-voice edit
  ```

## Usage

### First Run

When you run the application for the first time, it will initialize the configuration directory at `~/.config/hd-linux-voice/` with default settings.

## Profile & Pack Management

### Understanding Profiles
Profiles are sets of macro configurations stored in `~/.config/hd-linux-voice/profiles/`.

### Creating a Custom Pack
1. Create a profile configuration file.
2. Add custom macros and sounds.
3. Export it to a `.hdpack` file.
4. Share or use it with the `import` command.

## Configuration

The application uses a `config.yaml` file to store persistent settings like wake word, activation mode, and volume.

See [Configuration Guide](CONFIG.md) for detailed information.