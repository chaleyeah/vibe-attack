#!/bin/sh
# verify-appimage.sh — build the AppImage and capture a structured proof transcript.
#
# Usage: bash scripts/verify-appimage.sh <transcript-path>
#
# The transcript is written even on failure (STATUS: failed:<reason>) so
# partial proof is still inspectable.  On success STATUS: ok is written.
# When linuxdeploy/appimagetool are absent the script emits
# STATUS: skipped:tools-missing and exits 0 — the static packaging tests
# still cover build.sh structure in that case.
#
# Transcript fields (written in order):
#   STATUS, DISTRO, KERNEL, SIZE_BYTES, SHA256, EXIT_CODE, VERSION_OUTPUT
#   FAILURE_REASON (only when status is failed/skipped)

set -euo pipefail

TRANSCRIPT="${1:-}"
if [ -z "$TRANSCRIPT" ]; then
    echo "Usage: $0 <transcript-path>" >&2
    exit 1
fi

# Ensure the parent directory of the transcript exists.
TRANSCRIPT_DIR="$(dirname "$TRANSCRIPT")"
mkdir -p "$TRANSCRIPT_DIR"

# ── helper: collect platform metadata ────────────────────────────────────────
get_distro() {
    if [ -f /etc/os-release ]; then
        # shellcheck disable=SC1091
        DISTRO_ID="$(. /etc/os-release && echo "${PRETTY_NAME:-${ID:-unknown}}")"
        echo "$DISTRO_ID"
    elif command -v lsb_release > /dev/null 2>&1; then
        lsb_release -sd
    else
        echo "unknown"
    fi
}

get_kernel() {
    uname -r
}

DISTRO="$(get_distro)"
KERNEL="$(get_kernel)"

# ── helper: write transcript ─────────────────────────────────────────────────
write_transcript() {
    STATUS="$1"
    SIZE_BYTES="${2:-pending}"
    SHA256="${3:-pending}"
    EXIT_CODE="${4:-pending}"
    VERSION_OUTPUT="${5:-pending}"
    FAILURE_REASON="${6:-}"

    {
        echo "STATUS: $STATUS"
        echo "DISTRO: $DISTRO"
        echo "KERNEL: $KERNEL"
        echo "SIZE_BYTES: $SIZE_BYTES"
        echo "SHA256: $SHA256"
        echo "EXIT_CODE: $EXIT_CODE"
        echo "VERSION_OUTPUT: $VERSION_OUTPUT"
        if [ -n "$FAILURE_REASON" ]; then
            echo "FAILURE_REASON: $FAILURE_REASON"
        fi
    } > "$TRANSCRIPT"
}

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

APPIMAGE_NAME="vibe-attack-x86_64.AppImage"
MAX_BYTES=52428800  # 50 MB

# ── detect required packaging tools ─────────────────────────────────────────
if ! command -v linuxdeploy > /dev/null 2>&1 || ! command -v appimagetool > /dev/null 2>&1; then
    write_transcript "skipped:tools-missing" "pending" "pending" "pending" "pending" \
        "linuxdeploy and/or appimagetool not found; install both to produce the AppImage"
    echo "STATUS: skipped:tools-missing — transcript written to $TRANSCRIPT" >&2
    exit 0
fi

# ── run build.sh ─────────────────────────────────────────────────────────────
BUILD_EXIT=0
bash packaging/appimage/build.sh || BUILD_EXIT=$?
if [ "$BUILD_EXIT" -ne 0 ]; then
    write_transcript "failed:build-script-nonzero" "pending" "pending" "$BUILD_EXIT" "pending" \
        "packaging/appimage/build.sh exited $BUILD_EXIT"
    exit 1
fi

# ── verify the AppImage was produced ─────────────────────────────────────────
if [ ! -f "$APPIMAGE_NAME" ]; then
    write_transcript "failed:appimage-missing" "pending" "pending" "0" "pending" \
        "$APPIMAGE_NAME not found after build.sh exited 0"
    exit 1
fi

# ── size guard ───────────────────────────────────────────────────────────────
SIZE_BYTES="$(wc -c < "$APPIMAGE_NAME" | tr -d ' ')"
if [ "$SIZE_BYTES" -gt "$MAX_BYTES" ]; then
    SHA256="$(sha256sum "$APPIMAGE_NAME" | awk '{print $1}')"
    write_transcript "failed:too-large" "$SIZE_BYTES" "$SHA256" "0" "pending" \
        "AppImage is ${SIZE_BYTES} bytes, exceeds 50 MB limit"
    exit 1
fi

SHA256="$(sha256sum "$APPIMAGE_NAME" | awk '{print $1}')"

# ── run --version ─────────────────────────────────────────────────────────────
chmod +x "$APPIMAGE_NAME"
VERSION_EXIT=0
VERSION_OUTPUT="$(./"$APPIMAGE_NAME" --version 2>&1)" || VERSION_EXIT=$?

if [ "$VERSION_EXIT" -ne 0 ]; then
    write_transcript "failed:version-check-failed" "$SIZE_BYTES" "$SHA256" "$VERSION_EXIT" \
        "$VERSION_OUTPUT" "--version exited $VERSION_EXIT"
    exit 1
fi

# ── all checks passed ─────────────────────────────────────────────────────────
write_transcript "ok" "$SIZE_BYTES" "$SHA256" "$VERSION_EXIT" "$VERSION_OUTPUT"
echo "STATUS: ok — transcript written to $TRANSCRIPT" >&2
echo "VERSION_OUTPUT: $VERSION_OUTPUT" >&2
