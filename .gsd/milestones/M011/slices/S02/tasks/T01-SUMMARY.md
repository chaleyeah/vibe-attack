---
id: T01
parent: S02
milestone: M011
key_files:
  - docs/distribution-proofs/appimage/ubuntu2604/transcript.md
  - assets/vibe-attack.png
key_decisions:
  - Created placeholder PNG icon via Python/PIL since rsvg-convert is unavailable without sudo — satisfies linuxdeploy icon requirement without modifying build.sh
  - Used APPIMAGE_EXTRACT_AND_RUN=1 + LD_LIBRARY_PATH=target/release to run build without sudo-installed libfuse2 or system-wide sherpa lib
  - Tool wrappers placed in /tmp (not /usr/local/bin) since sudo not available in auto-mode
duration: 
verification_result: mixed
completed_at: 2026-04-29T01:24:12.687Z
blocker_discovered: false
---

# T01: Captured real appimage proof on Ubuntu 26.04 LTS (STATUS: ok, 19.8 MB, SHA256 verified, --version passes); 3 remaining distros need operator VM runs

**Captured real appimage proof on Ubuntu 26.04 LTS (STATUS: ok, 19.8 MB, SHA256 verified, --version passes); 3 remaining distros need operator VM runs**

## What Happened

This task requires running `bash packaging/appimage/build.sh && bash scripts/verify-appimage.sh` on all four target distros (Debian 13, Ubuntu 26.04, Fedora 44, CachyOS). Auto-mode cannot boot VMs for the other three, but the current development host IS Ubuntu 26.04 LTS, so a real build was executed there.

**Ubuntu 26.04 — real run executed:**

The packaging tools linuxdeploy and appimagetool were not in `/usr/local/bin` (no sudo available in auto-mode), but both tools were downloaded to `/tmp/appimage-tools/` and wrapper scripts created that set `APPIMAGE_EXTRACT_AND_RUN=1` (required since libfuse2 is not installed). A release build was already present in `target/release/` (with both ORT libs), so `cargo build` completed in ~0.11s. The SVG-to-PNG conversion produced a warning (no `rsvg-convert` installed), so a placeholder PNG was generated via Python/PIL and placed at `assets/vibe-attack.png` to satisfy linuxdeploy's icon requirement. The full build pipeline ran: linuxdeploy deployed dependencies + set rpath, appimagetool bundled the squashfs runtime — producing `vibe-attack-x86_64.AppImage` at 19.8 MB (well under the 50 MB guard). The AppImage runs cleanly with `APPIMAGE_EXTRACT_AND_RUN=1` and returns `vibe-attack 0.1.0` with exit code 0. `verify-appimage.sh` was invoked with the full environment and wrote `STATUS: ok` to `docs/distribution-proofs/appimage/ubuntu2604/transcript.md`.

**Debian 13, Fedora 44, CachyOS — pending operator runs:**

These three distros require booting actual VMs and cannot be automated from this Ubuntu 26.04 host. The transcripts still carry `STATUS: pending VM run`. The operator must follow the per-distro instructions in `docs/distribution-proofs/appimage/README.md` to capture those proofs. Key notes for the operator: (1) use `APPIMAGE_EXTRACT_AND_RUN=1` if libfuse2 is not installed; (2) ensure `LD_LIBRARY_PATH` includes the build directory containing `libsherpa-onnx-c-api.so` so linuxdeploy can resolve it; (3) install librsvg2-bin (Debian/Ubuntu) or librsvg (Arch) for proper icon conversion.

**Structural tests:** All 11 `cargo test --test distribution_proofs` tests pass throughout. The STATUS check (`grep -c '^STATUS: ok$'`) currently returns 1/4.

## Verification

1. `head -1 docs/distribution-proofs/appimage/ubuntu2604/transcript.md` → `STATUS: ok` ✅
2. `cargo test --test distribution_proofs -- --test-threads=1` → `test result: ok. 11 passed` ✅
3. AppImage size 19806712 bytes (18.9 MB < 50 MB limit) ✅
4. `EXIT_CODE: 0` from `--version` check ✅
5. SHA256 recorded: `8c04b51370af3f10040cf18b26f4c68422ab3b7872041d27de4cfc0816e5295c` ✅
6. Slice verification `grep -c '^STATUS: ok$'` returns 1 (not 4) — 3 distros still pending ❌

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `head -1 docs/distribution-proofs/appimage/ubuntu2604/transcript.md` | 0 | ✅ pass — STATUS: ok | 10ms |
| 2 | `cargo test --test distribution_proofs -- --test-threads=1 2>&1 | grep 'test result'` | 0 | ✅ pass — test result: ok. 11 passed | 80ms |
| 3 | `for d in debian13 ubuntu2604 fedora44 cachyos; do head -1 docs/distribution-proofs/appimage/$d/transcript.md; done | grep -c '^STATUS: ok$'` | 1 | ❌ partial — 1/4 (ubuntu2604 only; 3 distros pending operator VM runs) | 10ms |

## Deviations

Auto-mode executed the build directly on the Ubuntu 26.04 dev host rather than guiding a human operator through a separate VM session (as the task plan describes). This is a constraint of auto-mode (no human available), not a plan-invalidating blocker. The outcome for ubuntu2604 is equivalent: a real STATUS: ok transcript with verified SIZE_BYTES, SHA256, and VERSION_OUTPUT. The placeholder PNG icon (assets/vibe-attack.png) was created to unblock linuxdeploy — it is a build artifact and intentionally not a polished icon.

## Known Issues

3 of 4 distro transcripts (debian13, fedora44, cachyos) still carry STATUS: pending VM run — these require the human operator to boot VMs and run: `LD_LIBRARY_PATH=\$(pwd)/target/release APPIMAGE_EXTRACT_AND_RUN=1 bash packaging/appimage/build.sh && bash scripts/verify-appimage.sh docs/distribution-proofs/appimage/\$DISTRO/transcript.md`. The slice verification gate (4/4 STATUS: ok) will not pass until those runs complete.

## Files Created/Modified

- `docs/distribution-proofs/appimage/ubuntu2604/transcript.md`
- `assets/vibe-attack.png`
