STATUS: pending VM run
DISTRO: pending
KERNEL: pending
APPIMAGE_VERSION: pending
APPIMAGE_SIZE_BYTES: pending
WIZARD_COMPLETED: pending
STRATAGEM_FIRED: pending
INSTALL_METHOD: appimage

## Reproduction Notes

- Boot a CachyOS VM or bare-metal installation with a full desktop session (GNOME or KDE) so that polkit dialogs render correctly.
- Install the FUSE2 runtime dependency: `sudo pacman -Sy --noconfirm fuse2`
- Download the AppImage from the GitHub Releases page: `wget https://github.com/chaleyeah/vibe-attack/releases/latest/download/vibe-attack-x86_64.AppImage`
- Make it executable: `chmod +x vibe-attack-x86_64.AppImage`
- Confirm it launches: `./vibe-attack-x86_64.AppImage --version`
- Run the full wizard end-to-end: `./vibe-attack-x86_64.AppImage` — step through all wizard steps (CreateConfig → InstallModel → SetupUinput → ConfigurePtt); confirm the main config screen appears after the wizard clears.
- Fire at least one stratagem by voice to confirm the mic pipeline is live; record `STRATAGEM_FIRED: yes` on success.
- Fill in field values:
  - `DISTRO`: `grep PRETTY_NAME /etc/os-release | cut -d= -f2 | tr -d '"'`
  - `KERNEL`: `uname -r`
  - `APPIMAGE_VERSION`: `./vibe-attack-x86_64.AppImage --version 2>&1`
  - `APPIMAGE_SIZE_BYTES`: `stat -c %s vibe-attack-x86_64.AppImage`
  - `WIZARD_COMPLETED`: `yes` if the main config screen appeared; `no` otherwise
  - `STRATAGEM_FIRED`: `yes` if stratagem fired by voice; `no` otherwise
- Set `STATUS: ok` when all fields are confirmed; set `STATUS: failed:<reason>` if any step fails.
