---
id: T04
parent: S05
milestone: M007
key_files:
  - docs/troubleshooting.md
key_decisions:
  - No udev rule section added to uinput-setup.md — packaging does not ship any .rules files, so documenting a non-existent approach would create confusion. The group membership approach is sufficient and already documented.
  - Used 'vibe-attack > /dev/null 2>&1 &' for daemon restart example — redirects stdout/stderr to prevent shell hang when used interactively, consistent with background process conventions.
duration: 
verification_result: passed
completed_at: 2026-04-27T12:31:30.545Z
blocker_discovered: false
---

# T04: docs: Corrected three drift items in docs/troubleshooting.md — removed non-existent 'daemon' subcommand, fixed ping response casing to 'Pong', added missing libclang-dev/clang to build deps, and updated udev rule cross-reference to accurately describe uinput-setup.md contents

**docs: Corrected three drift items in docs/troubleshooting.md — removed non-existent 'daemon' subcommand, fixed ping response casing to 'Pong', added missing libclang-dev/clang to build deps, and updated udev rule cross-reference to accurately describe uinput-setup.md contents**

## What Happened

Cross-referenced docs/troubleshooting.md and docs/uinput-setup.md against Cargo.toml [[bin]] entries, src/error.rs Display impls, src/main.rs Commands enum, and packaging files (PKGBUILD, .spec, debian/rules).

**uinput-setup.md:** Fully accurate — uses 'input' group (not 'uinput'), systemd v258+ note present, no udev rule section (matches reality: no .rules files exist in packaging). No changes needed.

**troubleshooting.md — three drift items fixed:**

1. **Daemon subcommand**: The Daemon section showed `vibe-attack daemon &` to restart the daemon. There is no `daemon` subcommand in the Commands enum (src/main.rs:30-43 — only Ping, Switch, Test, Import, Export, Edit). The binary IS the daemon; you run `vibe-attack` directly. Fixed to `vibe-attack > /dev/null 2>&1 &` (redirect required to avoid hanging shell, consistent with background process convention).

2. **Ping response casing**: The doc said 'A healthy daemon prints `pong`' but the actual output is `println!("{resp:?}")` on `ControlResponse::Pong` which formats as `Pong` (Debug repr). Fixed to `Pong`.

3. **Build dependencies**: Debian/Ubuntu build fix listed `libasound2-dev pkg-config cmake` but omitted `libclang-dev` which sherpa-onnx-sys bindgen requires (added in T02 to CONTRIBUTING.md). Arch/CachyOS listed `alsa-lib pkgconf cmake` but omitted `clang`. Both updated to match CONTRIBUTING.md and packaging/vibe-attack.spec (which already lists `clang-devel` for Fedora).

4. **udev cross-reference**: The uinput section said 'See docs/uinput-setup.md for the full udev rule approach' but uinput-setup.md contains no udev rule section — only group membership and module-load steps. Changed to 'the full module-load and group setup steps'.

Binary names confirmed correct: `vibe-attack` and `vibe-attack-config` both appear in Cargo.toml [[bin]] and in all packaging files.

Error messages confirmed: `DaemonError::UinputPermissionDenied` and `DaemonError::InputGroupMissing` match the symptom/fix text in troubleshooting.md. `NoPttDevice` is not surfaced in troubleshooting.md (no PTT section) — acceptable scope gap not introduced by this task.

## Verification

Manual cross-reference of every claim in both docs against: Cargo.toml [[bin]] names, src/main.rs Commands enum, src/error.rs Display impls, control/protocol.rs ControlResponse variants, packaging/PKGBUILD makedepends, packaging/vibe-attack.spec BuildRequires, packaging/debian/rules, CONTRIBUTING.md prerequisites. cargo test passed (1 pass, 0 fail, 2 ignored requiring hardware). cargo clippy unavailable in this environment (no rustup/clippy component installed) — docs-only changes have no impact on lint results.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test` | 0 | ✅ pass | 12400ms |
| 2 | `grep -c 'daemon &' docs/troubleshooting.md` | 1 | ✅ pass — no stale 'daemon' subcommand reference remains | 50ms |
| 3 | `grep 'vibe-attack.*&' docs/troubleshooting.md` | 0 | ✅ pass — restart command now uses output redirect | 40ms |

## Deviations

None — the task plan called for line-by-line verification and updating drift; all four items found and fixed are documentation accuracy corrections within the stated scope.

## Known Issues

packaging/PKGBUILD is missing 'clang' from makedepends (sherpa-onnx-sys bindgen requires it) — out of scope for M007 (doc-only milestone). cargo clippy not available in this build environment; docs-only changes carry no lint risk.

## Files Created/Modified

- `docs/troubleshooting.md`
