STATUS: pending VM run
DISTRO: Arch Linux
KERNEL: pending
SIZE_BYTES: pending
SHA256: pending
EXIT_CODE: pending
VERSION_OUTPUT: pending
FAILURE_REASON: VM run not yet executed

# Reproduction Instructions

## Required system packages

```
sudo pacman -Sy --noconfirm alsa-lib clang librsvg fuse2 wget
```

## Install packaging tools

```
wget -q https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
wget -q https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
chmod +x linuxdeploy-x86_64.AppImage appimagetool-x86_64.AppImage
sudo mv linuxdeploy-x86_64.AppImage /usr/local/bin/linuxdeploy
sudo mv appimagetool-x86_64.AppImage /usr/local/bin/appimagetool
```

## Capture transcript

```
bash packaging/appimage/build.sh && bash scripts/verify-appimage.sh docs/distribution-proofs/appimage/arch/transcript.md
```

This transcript is acceptable as `STATUS: pending VM run` until the Arch Linux VM run is completed in S06.
