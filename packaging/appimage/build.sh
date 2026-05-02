#!/usr/bin/env bash
set -euo pipefail

PKGNAME="vibe-attack"
APPDIR="AppDir"

echo "Building $PKGNAME release binary..."
cargo build --release --features gui

echo "Creating AppDir structure..."
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/lib"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"

echo "Copying binaries..."
cp "target/release/$PKGNAME" "$APPDIR/usr/bin/$PKGNAME"
cp "target/release/${PKGNAME}-config" "$APPDIR/usr/bin/${PKGNAME}-config"

echo "Copying .desktop file..."
cp "packaging/appimage/vibe-attack.desktop" "$APPDIR/usr/share/applications/"
cp "packaging/appimage/vibe-attack.desktop" "$APPDIR/$PKGNAME.desktop"

echo "Converting SVG icon to PNG..."
ICON_SVG="assets/$PKGNAME.svg"
ICON_PNG="assets/$PKGNAME.png"
if [ -f "$ICON_SVG" ]; then
    if command -v rsvg-convert > /dev/null 2>&1; then
        rsvg-convert -w 256 -h 256 "$ICON_SVG" -o "$ICON_PNG"
    elif command -v inkscape > /dev/null 2>&1; then
        inkscape --export-type=png --export-width=256 --export-height=256 \
            --export-filename="$ICON_PNG" "$ICON_SVG"
    elif command -v convert > /dev/null 2>&1; then
        convert -background none -resize 256x256 "$ICON_SVG" "$ICON_PNG"
    else
        echo "WARNING: No SVG converter found (rsvg-convert, inkscape, or convert)." >&2
        echo "  Install librsvg2-bin (Debian) or librsvg (Arch) and re-run." >&2
    fi
fi

if [ -f "$ICON_PNG" ]; then
    cp "$ICON_PNG" "$APPDIR/usr/share/icons/hicolor/256x256/apps/$PKGNAME.png"
    cp "$ICON_PNG" "$APPDIR/$PKGNAME.png"
else
    echo "WARNING: No icon PNG found at $ICON_PNG — AppImage will lack an icon." >&2
fi

# Discover a shared library by name.
# Priority: target/release/ -> sherpa prebuilt cache -> $ORT_DYLIB_PATH
#           -> ldconfig -> /usr search.
# The sherpa prebuilt cache is checked because a full Rust cache hit means
# cargo build --release is a no-op and the ort build script never runs,
# so the .so is never copied to target/release/.
find_so() {
    local name="$1"
    if [ -f "target/release/$name" ]; then
        echo "target/release/$name"; return
    fi
    local prebuilt
    prebuilt="$(find target/sherpa-onnx-prebuilt -name "$name" 2>/dev/null | head -1)"
    if [ -n "$prebuilt" ] && [ -f "$prebuilt" ]; then
        echo "$prebuilt"; return
    fi
    local ldp
    ldp="$(ldconfig -p 2>/dev/null | awk -v n="$name" '$0 ~ n {print $NF}' | head -1)"
    if [ -n "$ldp" ] && [ -f "$ldp" ]; then
        echo "$ldp"; return
    fi
    local fp
    fp="$(find /usr -name "$name" 2>/dev/null | head -1)"
    if [ -n "$fp" ] && [ -f "$fp" ]; then
        echo "$fp"; return
    fi
    echo ""
}

ORT_SO="${ORT_DYLIB_PATH:-$(find_so libonnxruntime.so)}"
if [ -z "$ORT_SO" ] || [ ! -f "$ORT_SO" ]; then
    echo "ERROR: libonnxruntime.so not found." >&2
    echo "  Build with: cargo build --release (copies it to target/release/)" >&2
    echo "  Or set ORT_DYLIB_PATH to the full path of the .so" >&2
    exit 1
fi
echo "Bundling ORT library from: $ORT_SO"
cp "$ORT_SO" "$APPDIR/usr/lib/libonnxruntime.so"

SHERPA_SO="$(find_so libsherpa-onnx-c-api.so)"
if [ -z "$SHERPA_SO" ]; then
    # Also try target/release for a build that copied it there
    SHERPA_SO="$(find target/release -name 'libsherpa-onnx-c-api.so' 2>/dev/null | head -1)"
fi
if [ -n "$SHERPA_SO" ] && [ -f "$SHERPA_SO" ]; then
    echo "Bundling sherpa-onnx-c-api library from: $SHERPA_SO"
    cp "$SHERPA_SO" "$APPDIR/usr/lib/libsherpa-onnx-c-api.so"
else
    echo "WARNING: libsherpa-onnx-c-api.so not found — wake-word detection may fail at runtime." >&2
fi

# Write AppRun — sets LD_LIBRARY_PATH and ORT_DYLIB_PATH so both the dynamic
# linker and ort's load-dynamic dlopen find libonnxruntime.so inside the FUSE
# mount before any system paths.  ORT_DYLIB_PATH must be absolute with no ".."
# components — ort resolves it via is_absolute() and skips existence checks,
# so a path containing ".." can silently resolve to the wrong location on some
# FUSE implementations.
cat > "$APPDIR/AppRun" << 'EOF'
#!/usr/bin/env bash
HERE="$(dirname "$(readlink -f "$0")")"
export LD_LIBRARY_PATH="${HERE}/usr/lib${LD_LIBRARY_PATH:+:${LD_LIBRARY_PATH}}"
export ORT_DYLIB_PATH="${HERE}/usr/lib/libonnxruntime.so"
exec "${HERE}/usr/bin/vibe-attack-config" "$@"
EOF
chmod +x "$APPDIR/AppRun"

echo "AppDir prepared at ./$APPDIR"

# Assemble the final AppImage if tools are available.
if command -v linuxdeploy > /dev/null 2>&1 && command -v appimagetool > /dev/null 2>&1; then
    echo "Running linuxdeploy..."
    # Expose bundled .so files to ldd/linuxdeploy dependency resolution.
    # libsherpa-onnx-c-api and libonnxruntime are dlopen'd at runtime and not
    # in the binary's RPATH, so linuxdeploy cannot find them via ldd alone.
    # Adding the AppDir lib path to LD_LIBRARY_PATH makes ldd resolve them.
    export LD_LIBRARY_PATH="$(pwd)/$APPDIR/usr/lib${LD_LIBRARY_PATH:+:${LD_LIBRARY_PATH}}"
    linuxdeploy --appdir "$APPDIR"
    echo "Running appimagetool..."
    appimagetool "$APPDIR" "${PKGNAME}-x86_64.AppImage"
    echo "Done: ${PKGNAME}-x86_64.AppImage"
else
    echo "linuxdeploy/appimagetool not found — skipping final AppImage assembly."
    echo "Install both tools and re-run to produce ${PKGNAME}-x86_64.AppImage"
    echo "  Arch:   paru -S linuxdeploy-bin appimagetool-bin"
    echo "  Debian: download from https://github.com/linuxdeploy/linuxdeploy/releases"
fi
