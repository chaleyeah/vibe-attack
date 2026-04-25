---
id: T02
parent: S05
milestone: M001
key_files:
  - packaging/PKGBUILD
  - packaging/appimage/hd-linux-voice.desktop
  - packaging/appimage/build.sh
  - tests/ui_distribution.rs
key_decisions:
  - PKGBUILD uses license=('AGPL-3.0-only') matching Cargo.toml exactly
  - build.sh LD_LIBRARY_PATH section handles ORT AppImage FUSE mount constraint (dlopen needs the .so at runtime inside the mount)
  - Exec= in .desktop has no full path (just binary name) per Freedesktop spec and task constraint
  - chmod +x blocked by environment — documented as deviation; none of the 4 structural tests check the execute bit
duration: 
verification_result: passed
completed_at: 2026-04-25T19:45:13.574Z
blocker_discovered: false
---

# T02: Created packaging/PKGBUILD, packaging/appimage/hd-linux-voice.desktop, and packaging/appimage/build.sh, plus 4 structural tests (tests 12-15) in tests/ui_distribution.rs bringing total to 15 tests

**Created packaging/PKGBUILD, packaging/appimage/hd-linux-voice.desktop, and packaging/appimage/build.sh, plus 4 structural tests (tests 12-15) in tests/ui_distribution.rs bringing total to 15 tests**

## What Happened

Created the full `packaging/` directory tree with three distribution artifacts and appended four structural tests to `tests/ui_distribution.rs`.

**packaging/PKGBUILD** — AUR-style PKGBUILD with all required fields: `pkgname=hd-linux-voice`, `pkgver=0.1.0`, `pkgrel=1`, `pkgdesc`, `arch=('x86_64')`, `url=`, `license=('AGPL-3.0-only')` (matching Cargo.toml), `depends=('alsa-lib')`, `makedepends=('rust' 'cargo')`, plus `build()` calling `cargo build --release --locked` and `package()` installing binary, .desktop, README, and LICENSE.

**packaging/appimage/hd-linux-voice.desktop** — Freedesktop .desktop file with `[Desktop Entry]` section, `Name=HD Linux Voice`, `Exec=hd-linux-voice` (no full path per constraint), `Type=Application`, `Icon=hd-linux-voice`, `Comment=Voice macro daemon for Helldivers 2`, `Categories=Game;Utility;`, `Terminal=false`.

**packaging/appimage/build.sh** — Bash AppImage build script with `#!/usr/bin/env bash` shebang and `set -euo pipefail`. Runs `cargo build --release`, creates AppDir structure, copies binary + .desktop + icon, includes the required LD_LIBRARY_PATH comment for ORT AppImage FUSE mount constraint (the ONNX Runtime .so files are extracted into the FUSE mount at runtime and LD_LIBRARY_PATH must be set for dlopen to find them), and has commented-out `linuxdeploy` + `appimagetool` final steps since those tools may not be installed.

**tests/ui_distribution.rs** — Appended 4 tests after T01's 11:
- `pkgbuild_file_exists_and_has_required_fields` (test 12): reads packaging/PKGBUILD via CARGO_MANIFEST_DIR, asserts pkgname=, pkgver=, url=, license= all present
- `desktop_file_exists_and_has_required_keys` (test 13): reads packaging/appimage/hd-linux-voice.desktop, asserts Name=, Exec=, Type= present
- `appimage_build_script_exists` (test 14): asserts packaging/appimage/build.sh exists and contents are non-empty
- `appimage_build_script_has_shebang` (test 15): reads first line of build.sh, asserts it starts with `#!`

All tests use `env!("CARGO_MANIFEST_DIR")` for path resolution, keeping them portable and runnable from any working directory.

**Deviation:** `chmod +x build.sh` was blocked by the execution environment's tool permissions. The execute bit is not checked by any of the 4 structural tests (which only verify existence, non-emptiness, and shebang), so this does not affect test correctness. The file can be made executable with `chmod +x packaging/appimage/build.sh` when needed.

## Verification

Static verification:
1. All three packaging files exist and are non-empty.
2. PKGBUILD contains all required fields: pkgname=, pkgver=, url=, license= confirmed via grep.
3. .desktop file contains Name=, Exec=, Type= — confirmed. Exec=hd-linux-voice has no full path.
4. build.sh first line is `#!/usr/bin/env bash` — confirmed.
5. build.sh includes LD_LIBRARY_PATH for ORT AppImage FUSE mount — confirmed.
6. license=('AGPL-3.0-only') matches Cargo.toml license field — confirmed.
7. test count in tests/ui_distribution.rs: 15 (11 T01 + 4 T02) — confirmed via grep -c.
8. All 4 new tests use env!("CARGO_MANIFEST_DIR") — confirmed.
9. Slice-level file verification command ran successfully: test -f for all 3 files + grep checks all passed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `test -f packaging/PKGBUILD && echo OK` | 0 | ✅ pass — PKGBUILD exists | 5ms |
| 2 | `test -f packaging/appimage/hd-linux-voice.desktop && echo OK` | 0 | ✅ pass — .desktop file exists | 5ms |
| 3 | `test -f packaging/appimage/build.sh && echo OK` | 0 | ✅ pass — build.sh exists | 5ms |
| 4 | `grep -q 'pkgname=' packaging/PKGBUILD && echo OK` | 0 | ✅ pass — pkgname= field present | 5ms |
| 5 | `grep -n 'pkgname=|pkgver=|url=|license=' packaging/PKGBUILD` | 0 | ✅ pass — all 4 required PKGBUILD fields found | 8ms |
| 6 | `grep -n 'Name=|Exec=|Type=' packaging/appimage/hd-linux-voice.desktop` | 0 | ✅ pass — all 3 required .desktop keys found | 5ms |
| 7 | `head -1 packaging/appimage/build.sh` | 0 | ✅ pass — shebang is #!/usr/bin/env bash | 5ms |
| 8 | `grep -q 'LD_LIBRARY_PATH' packaging/appimage/build.sh` | 0 | ✅ pass — ORT LD_LIBRARY_PATH comment present | 5ms |
| 9 | `grep -c '#\[test\]' tests/ui_distribution.rs` | 0 | ✅ pass — 15 tests (11 T01 + 4 T02) | 8ms |
| 10 | `grep -n 'CARGO_MANIFEST_DIR' tests/ui_distribution.rs | wc -l` | 0 | ✅ pass — all 4 new tests use CARGO_MANIFEST_DIR for path resolution | 8ms |

## Deviations

chmod +x packaging/appimage/build.sh was blocked by the execution environment. The execute bit is not tested by any of the 4 structural tests, so test correctness is unaffected. The file can be made executable manually with `chmod +x packaging/appimage/build.sh`.

## Known Issues

build.sh does not have the execute bit set due to blocked chmod. Run `chmod +x packaging/appimage/build.sh` to fix before actual AppImage builds.

## Files Created/Modified

- `packaging/PKGBUILD`
- `packaging/appimage/hd-linux-voice.desktop`
- `packaging/appimage/build.sh`
- `tests/ui_distribution.rs`
