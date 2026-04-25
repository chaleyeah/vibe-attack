#!/usr/bin/env bash
set -euo pipefail

PKGNAME="vibe-attack"
APPDIR="AppDir"

echo "Building $PKGNAME release binary..."
cargo build --release

echo "Creating AppDir structure..."
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/lib"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"

echo "Copying binary..."
cp "target/release/$PKGNAME" "$APPDIR/usr/bin/$PKGNAME"

echo "Copying .desktop file..."
cp "packaging/appimage/vibe-attack.desktop" "$APPDIR/usr/share/applications/"
cp "packaging/appimage/vibe-attack.desktop" "$APPDIR/$PKGNAME.desktop"

echo "Copying icon (placeholder if not present)..."
if [ -f "assets/$PKGNAME.png" ]; then
    cp "assets/$PKGNAME.png" "$APPDIR/usr/share/icons/hicolor/256x256/apps/$PKGNAME.png"
    cp "assets/$PKGNAME.png" "$APPDIR/$PKGNAME.png"
fi

# Discover libonnxruntime.so.
# Priority: target/release/ -> $ORT_DYLIB_PATH -> ldconfig -> /usr search.
ORT_SO=""
if [ -f "target/release/libonnxruntime.so" ]; then
    ORT_SO="target/release/libonnxruntime.so"
elif [ -n "${ORT_DYLIB_PATH:-}" ] && [ -f "$ORT_DYLIB_PATH" ]; then
    ORT_SO="$ORT_DYLIB_PATH"
else
    LDCONFIG_PATH="$(ldconfig -p 2>/dev/null | awk '/libonnxruntime\.so/{print $NF}' | head -1)"
    if [ -n "$LDCONFIG_PATH" ] && [ -f "$LDCONFIG_PATH" ]; then
        ORT_SO="$LDCONFIG_PATH"
    else
        FIND_PATH="$(find /usr -name 'libonnxruntime.so' 2>/dev/null | head -1)"
        if [ -n "$FIND_PATH" ] && [ -f "$FIND_PATH" ]; then
            ORT_SO="$FIND_PATH"
        fi
    fi
fi

if [ -z "$ORT_SO" ]; then
    echo "ERROR: libonnxruntime.so not found." >&2
    echo "  Build with: cargo build --release (copies it to target/release/)" >&2
    echo "  Or set ORT_DYLIB_PATH to the full path of the .so" >&2
    exit 1
fi

echo "Bundling ORT library from: $ORT_SO"
cp "$ORT_SO" "$APPDIR/usr/lib/libonnxruntime.so"

# Write AppRun — sets LD_LIBRARY_PATH so dlopen finds libonnxruntime.so inside
# the FUSE mount before any system paths. Without this, inference startup fails
# silently even though the .so is present in the AppDir.
cat > "$APPDIR/AppRun" << 'EOF'
#!/usr/bin/env bash
HERE="$(dirname "$(readlink -f "$0")")"
export LD_LIBRARY_PATH="${HERE}/usr/lib${LD_LIBRARY_PATH:+:${LD_LIBRARY_PATH}}"
exec "${HERE}/usr/bin/vibe-attack" "$@"
EOF
chmod +x "$APPDIR/AppRun"

echo "AppDir prepared at ./$APPDIR"

# Assemble the final AppImage if tools are available.
if command -v linuxdeploy > /dev/null 2>&1 && command -v appimagetool > /dev/null 2>&1; then
    echo "Running linuxdeploy..."
    linuxdeploy --appdir "$APPDIR" --output appimage
    echo "Running appimagetool..."
    appimagetool "$APPDIR" "${PKGNAME}-x86_64.AppImage"
    echo "Done: ${PKGNAME}-x86_64.AppImage"
else
    echo "linuxdeploy/appimagetool not found — skipping final AppImage assembly."
    echo "Install both tools and re-run to produce ${PKGNAME}-x86_64.AppImage"
fi
