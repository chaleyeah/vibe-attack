---
estimated_steps: 1
estimated_files: 7
skills_used: []
---

# T01: Capture appimage proofs on all four distros and verify structural tests still pass

Drive the human operator through running `bash packaging/appimage/build.sh && bash scripts/verify-appimage.sh docs/distribution-proofs/appimage/<distro>/transcript.md` on each of Debian 13, Ubuntu 26.04, Fedora 44, and CachyOS VMs. The script writes all 7 transcript fields (STATUS, DISTRO, KERNEL, SIZE_BYTES, SHA256, EXIT_CODE, VERSION_OUTPUT) automatically, so the agent's role is (a) confirm the four `appimage/<distro>/transcript.md` files still pre-flight clean, (b) hand the operator the exact dependency-install + script command from `docs/distribution-proofs/appimage/README.md`, (c) verify the four resulting transcripts carry `STATUS: ok` and pass `cargo test --test distribution_proofs -- --test-threads=1`. If any distro returns `STATUS: failed:<reason>`, file a follow-up note in the slice's blocker log and surface the FAILURE_REASON field — do NOT downgrade the slice goal silently. Per-distro dependency commands are already correct in `docs/distribution-proofs/appimage/README.md` (apt-get for Debian/Ubuntu, dnf for Fedora 44, pacman for CachyOS). MEM094 reminder: Debian/Ubuntu need libfuse2 (NOT libfuse3). MEM081/MEM099 reminder: verify-appimage.sh always writes a transcript even on failure — failed runs leave inspectable proof. The script's 50 MB size guard (MEM081) means an oversize build trips `STATUS: failed:too-large` — confirm `--release` (the default in build.sh) is used.

## Inputs

- `docs/distribution-proofs/appimage/debian13/transcript.md`
- `docs/distribution-proofs/appimage/ubuntu2604/transcript.md`
- `docs/distribution-proofs/appimage/fedora44/transcript.md`
- `docs/distribution-proofs/appimage/cachyos/transcript.md`
- `docs/distribution-proofs/appimage/README.md`
- `scripts/verify-appimage.sh`
- `packaging/appimage/build.sh`
- `tests/distribution_proofs.rs`

## Expected Output

- `docs/distribution-proofs/appimage/debian13/transcript.md`
- `docs/distribution-proofs/appimage/ubuntu2604/transcript.md`
- `docs/distribution-proofs/appimage/fedora44/transcript.md`
- `docs/distribution-proofs/appimage/cachyos/transcript.md`

## Verification

All four appimage/<distro>/transcript.md files have STATUS: ok (or a documented STATUS: failed:<reason> with FAILURE_REASON), and `cargo test --test distribution_proofs -- --test-threads=1` passes 11/11 tests. Verify with: `for d in debian13 ubuntu2604 fedora44 cachyos; do head -1 docs/distribution-proofs/appimage/$d/transcript.md; done | grep -c '^STATUS: ok$'` returns 4, AND `cargo test --test distribution_proofs -- --test-threads=1 2>&1 | grep -E 'test result: ok\. 11 passed'` matches.

## Observability Impact

Signals added/changed: real STATUS, EXIT_CODE, SHA256, SIZE_BYTES values overwrite the `pending` placeholders. How a future agent inspects this: read the four transcript files; on failure verify-appimage.sh leaves STATUS: failed:<reason> + FAILURE_REASON visible. Failure state exposed: per-distro build/package failures are captured in-file.
