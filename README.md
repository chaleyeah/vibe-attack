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

### Dependencies

#### System Packages

**Debian / Ubuntu:**
```bash
sudo apt-get update
sudo apt-get install -y nodejs git
```

**Arch / Manjaro:**
```bash
sudo pacman -Syu nodejs git
```

#### Rust
Install Rust and Cargo:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### PortAudio
Install the PortAudio development libraries:

**Debian / Ubuntu:**
```bash
sudo apt-get install -y libportaudio2 libportaudiocpp0
```

**Arch / Manjaro:**
```bash
sudo pacman -S portaudio
```

### Build and Run

1. **Clone the repository**:
   ```bash
   git clone https://github.com/chaleyeah/hd-linux-voice.git
   cd hd-linux-voice
   ```

2. **Build the project**:
   ```bash
   cargo build --release
   ```

3. **Run the application**:
   ```bash
   ./target/release/vibe-attack
   ```

## Usage

### Quick Start

Run the daemon with the default configuration:
```bash
./target/release/vibe-attack run
```

### Core Commands

- **Run**: Starts the voice recognition service in the foreground.
  ```bash
  ./target/release/vibe-attack run
  ```

- **Import**: Imports a `.hdpack` file into the profiles directory.
  ```bash
  ./target/release/vibe-attack import path/to/pack.hdpack
  ```

- **Export**: Exports a profile to a `.hdpack` file.
  ```bash
  ./target/release/vibe-attack export profile-name path/to/output.hdpack
  ```

- **Edit**: Opens the interactive TUI to edit macros.
  ```bash
  ./target/release/vibe-attack edit
  ```

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