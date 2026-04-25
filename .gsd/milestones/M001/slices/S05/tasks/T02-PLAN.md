---
estimated_steps: 42
estimated_files: 4
skills_used: []
---

# T02: Create packaging scaffolding (PKGBUILD, AppImage, .desktop) and add structural tests

Create the `packaging/` directory tree with three distribution artifacts and add 4 structural tests to `tests/ui_distribution.rs`.

**Files to create:**

1. `packaging/PKGBUILD` — AUR-style PKGBUILD template with required fields:
   - pkgname=hd-linux-voice
   - pkgver=0.1.0
   - pkgrel=1
   - pkgdesc='Voice-macro daemon for Helldivers 2 on Linux'
   - arch=('x86_64')
   - url='https://github.com/yourusername/hd-linux-voice'
   - license=('AGPL-3.0-only')
   - depends=('alsa-lib')
   - makedepends=('rust' 'cargo')
   - build() function calling cargo build --release
   - package() function installing binary + .desktop + docs

2. `packaging/appimage/hd-linux-voice.desktop` — Freedesktop .desktop file:
   - [Desktop Entry] section
   - Name=HD Linux Voice
   - Exec=hd-linux-voice
   - Type=Application
   - Icon=hd-linux-voice
   - Comment=Voice macro daemon for Helldivers 2
   - Categories=Game;Utility;
   - Terminal=false

3. `packaging/appimage/build.sh` — AppImage build script (executable):
   - Shebang #!/usr/bin/env bash
   - set -euo pipefail
   - cargo build --release
   - Creates AppDir structure
   - Copies binary, .desktop, icon into AppDir
   - Sets LD_LIBRARY_PATH for ORT .so (per research ORT AppImage constraint)
   - Calls linuxdeploy + appimagetool (commented-out final steps since tools may not be installed)

**Tests to ADD to existing tests/ui_distribution.rs (append after T01's tests):**

12. `pkgbuild_file_exists_and_has_required_fields` — reads `packaging/PKGBUILD`, asserts pkgname=, pkgver=, url=, license= lines present
13. `desktop_file_exists_and_has_required_keys` — reads `packaging/appimage/hd-linux-voice.desktop`, asserts Name=, Exec=, Type= lines present
14. `appimage_build_script_exists` — asserts `packaging/appimage/build.sh` exists and is non-empty
15. `appimage_build_script_has_shebang` — reads first line of build.sh, asserts starts with #!/

IMPORTANT CONSTRAINTS:
- build.sh must be created with execute permission (chmod +x after writing, or use std::os::unix::fs::PermissionsExt in test if checking permissions)
- PKGBUILD must use AGPL-3.0-only license (matching Cargo.toml)
- .desktop file must NOT include a full path in Exec= (just the binary name)
- Tests read files relative to env!("CARGO_MANIFEST_DIR") to find project root
- The build.sh script must include a comment about LD_LIBRARY_PATH for ORT .so in AppImage FUSE mount (per research constraint)

## Inputs

- `tests/ui_distribution.rs`
- `Cargo.toml`

## Expected Output

- `packaging/PKGBUILD`
- `packaging/appimage/hd-linux-voice.desktop`
- `packaging/appimage/build.sh`
- `tests/ui_distribution.rs`

## Verification

test -f packaging/PKGBUILD && test -f packaging/appimage/hd-linux-voice.desktop && test -f packaging/appimage/build.sh && grep -q 'pkgname=' packaging/PKGBUILD && grep -q 'Name=' packaging/appimage/hd-linux-voice.desktop && cargo test --test ui_distribution 2>&1 | grep -E 'test result|running'
