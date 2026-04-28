# S06: Final Distribution UAT — Research

**Date:** 2026-04-28
**Scope:** Three final UAT transcripts under `docs/distribution-proofs/final/` — one per distro — each showing the full loop: clean AppImage install → first-run wizard → stratagem fired by voice.

## Summary

S06 is the final integration gate for M010. Its job is to produce three `docs/distribution-proofs/final/` transcripts (Debian 12, Fedora 39, Arch) confirming the complete end-user flow works on each target distro. It depends on S03 (release CI producing a downloadable AppImage) and S05 (README updated).

All prior slices (S01–S05) have already established:
- The appimage transcript structure (7 fields, `STATUS: skipped:tools-missing` is valid on this host — MEM081)
- The wizard UAT transcript structure (10 fields, MEM085)
- The proof test harnesses (`tests/distribution_proofs.rs`, `tests/wizard_proofs.rs`)
- The `scripts/verify-appimage.sh` canonical proof-capture script (MEM081)

What is **missing** is:
1. A `docs/distribution-proofs/final/` directory with per-distro `transcript.md` files
2. A test in an appropriate test file asserting the `final/` transcript structure
3. A README or documentation index for the `final/` proof directory

This is straightforward structural work: mirror the existing `appimage/` and `wizard/` proof directory patterns into a new `final/` directory, add tests following the same `assert_transcript` pattern, and seed `pending VM run` transcripts for all three distros.

The actual VM runs (converting `STATUS: pending VM run` to `STATUS: ok`) are a human operator task that cannot be automated on this host. The code deliverable is the structural scaffolding that passes the test suite.

## Recommendation

Create `docs/distribution-proofs/final/` with three `transcript.md` files plus a `README.md`, extend `tests/distribution_proofs.rs` (or add a new `tests/final_proofs.rs`) with structural assertions for the final transcripts, and seed all three transcripts as `STATUS: pending VM run` per MEM079 policy. The single task is self-contained.

The final transcript format should combine the most relevant fields from both the AppImage and wizard transcripts, since S06 validates the full loop (AppImage download → run → wizard → stratagem). Proposed fields:

```
STATUS: <ok | pending VM run | failed:<reason>>
DISTRO: <os-release PRETTY_NAME>
KERNEL: <uname -r>
APPIMAGE_VERSION: <--version output>
APPIMAGE_SIZE_BYTES: <bytes>
WIZARD_COMPLETED: <yes|no|pending>
STRATAGEM_FIRED: <yes|no|pending>
INSTALL_METHOD: appimage
```

## Implementation Landscape

### Key Files

- `docs/distribution-proofs/appimage/README.md` — canonical pattern for proof directory layout; new `final/README.md` mirrors this
- `docs/distribution-proofs/wizard/README.md` — canonical pattern for wizard transcript format; new `final/` transcripts merge AppImage + wizard fields
- `docs/distribution-proofs/appimage/{debian12,fedora39,arch}/transcript.md` — existing seeded transcripts at `STATUS: skipped:tools-missing` or `pending VM run`; final transcripts start at `STATUS: pending VM run`
- `tests/distribution_proofs.rs` — existing AppImage structural tests; new final-UAT assertions can go here or in a new `tests/final_proofs.rs`
- `tests/wizard_proofs.rs` — wizard structural tests (same pattern); reference for `assert_transcript` helper
- `scripts/verify-appimage.sh` — canonical proof-capture script for AppImage; the final UAT reproducer can reference it

### Build Order

1. **Create `docs/distribution-proofs/final/` scaffolding**: `README.md` + `debian12/transcript.md` + `fedora39/transcript.md` + `arch/transcript.md` — all seeded as `STATUS: pending VM run`
2. **Add structural tests**: Extend `tests/distribution_proofs.rs` with three new `assert_final_transcript` tests covering the 8 combined fields
3. **Verify tests pass**: `cargo test --test distribution_proofs --test-threads=1` (mandatory serial per MEM080)

Step 1 unblocks step 2; both are in a single task.

### Verification Approach

```bash
# Must run serial (MEM080)
cargo test --test distribution_proofs -- --test-threads=1

# Structural grep checks on new transcripts
grep -q "STATUS:" docs/distribution-proofs/final/debian12/transcript.md
grep -q "STRATAGEM_FIRED:" docs/distribution-proofs/final/debian12/transcript.md
```

All three new test functions must pass (they will, since `pending VM run` is a valid STATUS).

## Constraints

- `--test-threads=1` is mandatory for all proof/packaging/distribution test suites (MEM080)
- AppImage cannot be built on this host (linuxdeploy/appimagetool missing — MEM078); transcripts must use `STATUS: pending VM run`, not attempt a live build
- `cargo clippy` is unavailable on this host; use `cargo build` as the build-clean gate (MEM038/MEM073)
- The `docs/distribution-proofs/final/` directory does not exist yet; it must be created
- The S06 roadmap description says "Three transcripts under docs/distribution-proofs/final/ — one per distro — each showing stratagem fired by voice from a clean AppImage install"; the deliverable is these three transcript files

## Common Pitfalls

- **Wrong STATUS for pending state** — use `STATUS: pending VM run` exactly (MEM079); not `pending` or `STATUS: pending`
- **Missing fields cause test failures** — the `assert_transcript` helper panics on any missing field; all 8 combined fields must be present in every transcript
- **Running tests in parallel** — always pass `--test-threads=1` (MEM080)
- **Attempting a live AppImage build** — this host lacks linuxdeploy/appimagetool; do not call `build.sh`; transcripts are seeded as `pending VM run`
