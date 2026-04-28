# S06: Final distribution UAT — UAT

**Milestone:** M010
**Written:** 2026-04-28T11:24:20.571Z

# S06: Final distribution UAT

**Milestone:** M010
**Written:** 2026-04-28

## UAT Type

- UAT mode: artifact-driven
- Why this mode is sufficient: This slice delivers structural scaffolding (proof directory + tests) rather than runtime behavior. The proof-directory contract (fields present, STATUS values valid) is fully verifiable by reading the transcript files and running the structural tests. Real end-to-end VM runtime proof (STATUS: ok) is a human operator deliverable deferred per MEM079.

## Preconditions

- Repo checked out at `/home/chadmin/Github/hd-linux-voice`
- `cargo` available (Rust toolchain present)
- `docs/distribution-proofs/final/` directory exists with three distro subdirs

## Smoke Test

Run `cargo test --test distribution_proofs -- --test-threads=1` and confirm all 9 tests pass (6 pre-existing + 3 new `*_final_transcript_has_required_fields`).

## Test Cases

### 1. Final transcript files exist with correct structure

1. Check `docs/distribution-proofs/final/debian12/transcript.md` exists and is non-empty.
2. Check `docs/distribution-proofs/final/fedora39/transcript.md` exists and is non-empty.
3. Check `docs/distribution-proofs/final/arch/transcript.md` exists and is non-empty.
4. **Expected:** All three files present; each contains exactly 8 structured fields in order.

### 2. STATUS field is exactly `pending VM run` in all three transcripts

1. Run: `grep 'STATUS:' docs/distribution-proofs/final/debian12/transcript.md`
2. Run: `grep 'STATUS:' docs/distribution-proofs/final/fedora39/transcript.md`
3. Run: `grep 'STATUS:' docs/distribution-proofs/final/arch/transcript.md`
4. **Expected:** Each outputs `STATUS: pending VM run` (exact string, lowercase, space before "VM").

### 3. INSTALL_METHOD is `appimage` in all three transcripts

1. Run: `grep 'INSTALL_METHOD:' docs/distribution-proofs/final/debian12/transcript.md`
2. **Expected:** `INSTALL_METHOD: appimage`
3. Repeat for fedora39 and arch.
4. **Expected:** Same result for all three.

### 4. All 8 required fields present per transcript

1. For each transcript, grep for: `STATUS:`, `DISTRO:`, `KERNEL:`, `APPIMAGE_VERSION:`, `APPIMAGE_SIZE_BYTES:`, `WIZARD_COMPLETED:`, `STRATAGEM_FIRED:`, `INSTALL_METHOD:`
2. **Expected:** Each field appears exactly once per file; no field missing.

### 5. Structural tests pass

1. Run: `cargo test --test distribution_proofs -- --test-threads=1`
2. **Expected:** 9/9 tests pass — including `debian12_final_transcript_has_required_fields`, `fedora39_final_transcript_has_required_fields`, `arch_final_transcript_has_required_fields`.

### 6. Full regression guard passes

1. Run: `cargo test --test distribution_proofs --test packaging --test wizard_proofs --test ui_distribution -- --test-threads=1`
2. **Expected:** 44/44 tests pass; exit code 0.

### 7. README documents the proof format

1. Read `docs/distribution-proofs/final/README.md`.
2. **Expected:** File documents directory layout (debian12/, fedora39/, arch/), all 8 field names with how-to-obtain instructions, STATUS values (ok / pending VM run / failed:<reason>), per-distro FUSE2 package names, and reproduction steps per distro.

## Edge Cases

### STATUS: failed:<reason> is accepted by assert_final_transcript

1. Temporarily edit a transcript's STATUS line to `STATUS: failed: mic unavailable`.
2. Run `cargo test --test distribution_proofs -- --test-threads=1`.
3. **Expected:** The structural test for that transcript still passes (failed: prefix is accepted).
4. Revert the edit.

### Existing AppImage/wizard tests are unaffected

1. Run: `cargo test --test distribution_proofs -- --test-threads=1`
2. **Expected:** All 6 pre-existing tests (appimage + wizard) still pass unchanged; `assert_transcript` and `assert_wizard_transcript` helpers are unmodified.

## Failure Signals

- Any `*_final_transcript_has_required_fields` test PANIC indicates a missing or misspelled field in the corresponding transcript.
- `grep` returning exit 1 on `STATUS: pending VM run` indicates STATUS was changed or has a typo.
- Tests pass but `INSTALL_METHOD: appimage` missing would indicate the one non-pending literal field was accidentally set to `pending`.

## Not Proven By This UAT

- Actual end-to-end AppImage download → wizard → stratagem-fired-by-voice on real Debian 12, Fedora 39, or Arch VMs (requires human operator with real VMs and a published release tag).
- Runtime behavior of the AppImage itself on any distro.
- Wizard completion, PTT key capture, or Whisper model download on target distros.

## Notes for Tester

Converting `STATUS: pending VM run` to `STATUS: ok` requires: a real VM matching the distro, libfuse2/fuse-libs/fuse2 installed, a published AppImage downloaded from GitHub Releases, `chmod +x`, running through the wizard end-to-end, and firing a stratagem by voice. The `## Reproduction Notes` section in each transcript provides the exact commands for obtaining each field value from the VM run.
