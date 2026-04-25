#!/usr/bin/env bash
set -euo pipefail

PKGNAME="hd-linux-voice"
APPDIR="AppDir"

echo "Building $PKGNAME release binary..."
cargo build --release

echo "Creating AppDir structure..."
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"

echo "Copying binary..."
cp "target/release/$PKGNAME" "$APPDIR/usr/bin/$PKGNAME"

echo "Copying .desktop file..."
cp "packaging/appimage/hd-linux-voice.desktop" "$APPDIR/usr/share/applications/"
cp "packaging/appimage/hd-linux-voice.desktop" "$APPDIR/$PKGNAME.desktop"

echo "Copying icon (placeholder if not present)..."
if [ -f "assets/$PKGNAME.png" ]; then
    cp "assets/$PKGNAME.png" "$APPDIR/usr/share/icons/hicolor/256x256/apps/$PKGNAME.png"
    cp "assets/$PKGNAME.png" "$APPDIR/$PKGNAME.png"
fi

# ORT AppImage constraint: the ONNX Runtime .so files are extracted into the
# AppImage FUSE mount at runtime. LD_LIBRARY_PATH must include the AppDir lib
# directory so the dynamic linker can find libonnxruntime.so when the binary
# runs inside the FUSE mount. Without this, dlopen() fails at inference startup.
ORT_LIB_DIR="$(dirname "$0")/../../target/release"
export LD_LIBRARY_PATH="${ORT_LIB_DIR}:${LD_LIBRARY_PATH:-}"

echo "LD_LIBRARY_PATH set to: $LD_LIBRARY_PATH"

# Final AppImage assembly (requires linuxdeploy and appimagetool in PATH).
# Uncomment the lines below once both tools are installed:
# linuxdeploy --appdir "$APPDIR" --output appimage
# appimagetool "$APPDIR" "${PKGNAME}-x86_64.AppImage"

echo "AppDir prepared at ./$APPDIR — run linuxdeploy + appimagetool to produce the final .AppImage"
