STATUS: pending VM run
DISTRO: pending
KERNEL: pending
SIZE_BYTES: pending
SHA256: pending
EXIT_CODE: pending
VERSION_OUTPUT: pending

## Reproduction Notes

- Boot a Debian 13 (Trixie) VM or bare-metal installation.
- Install dependencies: `sudo apt-get install -y libasound2-dev libclang-dev librsvg2-bin libfuse2 wget`
- Install packaging tools:
  ```bash
  wget -q https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
  wget -q https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
  chmod +x linuxdeploy-x86_64.AppImage appimagetool-x86_64.AppImage
  sudo mv linuxdeploy-x86_64.AppImage /usr/local/bin/linuxdeploy
  sudo mv appimagetool-x86_64.AppImage /usr/local/bin/appimagetool
  ```
- Capture transcript:
  ```bash
  bash packaging/appimage/build.sh && bash scripts/verify-appimage.sh docs/distribution-proofs/appimage/debian13/transcript.md
  ```
- Fill in STATUS, DISTRO, KERNEL, SIZE_BYTES, SHA256, EXIT_CODE, and VERSION_OUTPUT from the script output.
