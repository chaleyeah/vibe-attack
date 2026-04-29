STATUS: pending VM run
DISTRO: pending
KERNEL: pending
BINARY: vibe-attack-config
BINARY_VERSION: pending
SCENARIO_A: pending
SCENARIO_B: pending
SCENARIO_C: pending
SCENARIO_D: pending
STRATAGEM_FIRED: pending

## Reproduction Notes

- Boot a Fedora 44 VM or bare-metal installation with a full desktop session (GNOME or KDE) so that polkit dialogs render correctly.
- Install dependencies: `sudo dnf install -y alsa-lib-devel clang-devel librsvg2-tools fuse-libs wget`
- Clone the repo and build: `cargo build --release --bin vibe-attack-config`
- Add the binary to PATH: `export PATH="$PWD/target/release:$PATH"`
- Confirm polkit agent is running: `ps aux | grep polkit`
- **Scenario A (fresh install):** `rm -rf ~/.config/vibe-attack ~/.local/share/vibe-attack && vibe-attack-config` — step through all four wizard steps; record each step result; fire a stratagem by voice after wizard completes.
- **Scenario B (model pre-placed):** Place `ggml-tiny.en.bin` at `~/.local/share/vibe-attack/models/whisper/ggml-tiny.en.bin`; remove config only; relaunch — wizard must skip the InstallModel step.
- **Scenario C (relaunch):** After Scenario A completes successfully, relaunch `vibe-attack-config` — main config screen must appear with no wizard.
- **Scenario D (--skip-wizard):** `rm -rf ~/.config/vibe-attack ~/.local/share/vibe-attack && vibe-attack-config --skip-wizard` — main config screen must appear immediately.
- Fill in the STATUS and SCENARIO_* fields above with observed results (`ok` or `failed:<scenario-letter>`).
- Record DISTRO from `/etc/os-release` PRETTY_NAME, KERNEL from `uname -r`, BINARY_VERSION from `vibe-attack-config --version`.
