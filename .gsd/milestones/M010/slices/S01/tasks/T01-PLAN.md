---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Build AppImage on host and write verify-appimage.sh wrapper

Run a clean release build with the gui feature, execute packaging/appimage/build.sh, and add a portable shell script (scripts/verify-appimage.sh) that builds the AppImage, runs it with --version, and writes a structured transcript (distro, kernel, size, sha256, exit code, --version stdout, STATUS line) to a path passed as $1. The script is the canonical proof-capture tool used by this slice on the build host and by future VM runs in S06. Use only POSIX-portable shell so it runs on Debian, Fedora, and Arch without modification. Do NOT modify build.sh — research showed it is production-quality. The script must `set -euo pipefail`, fail loudly if the AppImage is missing or > 50 MB, and emit the transcript even on failure (with STATUS: failed:<reason>) so partial proof is still inspectable. Assumption (auto-mode): linuxdeploy and appimagetool may not be installed on the build host; the script must detect this and emit STATUS: skipped:tools-missing rather than fail — the static `cargo test --test packaging` tests still cover build.sh structure in that case.

## Inputs

- ``packaging/appimage/build.sh` — existing AppImage build pipeline; must be invoked unchanged`
- ``Cargo.toml` — release build target; provides `vibe-attack` and `vibe-attack-config` binaries with version 0.1.0`
- ``src/main.rs` — clap CLI with `version` attribute that produces `vibe-attack 0.1.0` for --version`

## Expected Output

- ``scripts/verify-appimage.sh` — new portable shell script that builds + runs the AppImage and emits a structured transcript`
- ``/tmp/host-transcript.md` — transient transcript proving the script works end-to-end on the build host (used as input for T02)`

## Verification

bash scripts/verify-appimage.sh /tmp/host-transcript.md && grep -q '^STATUS: ' /tmp/host-transcript.md && grep -q 'vibe-attack 0.1.0\|STATUS: skipped' /tmp/host-transcript.md

## Observability Impact

Signals added/changed: structured transcript format (STATUS, DISTRO, KERNEL, SIZE_BYTES, SHA256, VERSION_OUTPUT, EXIT_CODE). How a future agent inspects this: `cat docs/distribution-proofs/appimage/<distro>/transcript.md`. Failure state exposed: STATUS line at the top of the transcript distinguishes ok / failed / skipped, plus the actual error message in a FAILURE_REASON field when applicable.
