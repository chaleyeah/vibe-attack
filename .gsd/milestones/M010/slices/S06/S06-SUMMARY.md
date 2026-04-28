---
id: S06
parent: M010
milestone: M010
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - ["docs/distribution-proofs/final/README.md", "docs/distribution-proofs/final/debian12/transcript.md", "docs/distribution-proofs/final/fedora39/transcript.md", "docs/distribution-proofs/final/arch/transcript.md", "tests/distribution_proofs.rs"]
key_decisions:
  - ["assert_final_transcript uses a distinct 8-field FINAL_REQUIRED_FIELDS constant rather than extending the existing 7-field REQUIRED_FIELDS, keeping the two proof levels independently testable without coupling.", "STATUS: failed:<reason> acceptance is implemented via a line-prefix check (starts_with('STATUS: failed:')) rather than substring match, consistent with the existing VALID_STATUSES pattern.", "INSTALL_METHOD: appimage is the only non-pending literal field in pending-VM-run transcripts, since it is known statically regardless of VM state."]
patterns_established:
  - ["Proof directory scaffold pattern: seed transcripts with STATUS: pending VM run + structural tests; human operators convert to ok after real VM runs (MEM079 policy). Applied across appimage/, wizard/, and now final/ subdirectories.", "Separate field-set constants and helpers per proof level (assert_transcript for AppImage, assert_wizard_transcript for wizard, assert_final_transcript for final UAT) — each proof level is independently extensible."]
observability_surfaces:
  - ["docs/distribution-proofs/final/{distro}/transcript.md STATUS field — read STATUS: ok to confirm full UAT loop ran; STATUS: pending VM run means awaiting human operator VM run; STATUS: failed:<reason> localizes which step failed."]
drill_down_paths:
  - [".gsd/milestones/M010/slices/S06/tasks/T01-SUMMARY.md"]
duration: ""
verification_result: passed
completed_at: 2026-04-28T11:24:20.571Z
blocker_discovered: false
---

# S06: Final distribution UAT

**Scaffolded docs/distribution-proofs/final/ with three pending-VM-run transcripts (Debian 12, Fedora 39, Arch) and added 3 structural tests to distribution_proofs.rs; all 44 regression tests pass.**

## What Happened

S06 closes the M010 distribution proof chain by adding the final-UAT proof layer on top of the existing AppImage (S01) and wizard (S02) proof directories.

**T01** created the complete `docs/distribution-proofs/final/` directory scaffold:

- `README.md` documents the directory layout, all 8 transcript fields with how-to-obtain commands (`os-release` PRETTY_NAME for DISTRO, `uname -r` for KERNEL, `--version` for APPIMAGE_VERSION, `stat -c %s` for APPIMAGE_SIZE_BYTES), STATUS values (`ok` / `pending VM run` / `failed:<reason>`), per-distro FUSE2 package names (libfuse2 on Debian, fuse-libs on Fedora, fuse2 on Arch), and the MEM079 pending-transcript policy.

- Three transcript files (`debian12/transcript.md`, `fedora39/transcript.md`, `arch/transcript.md`) each contain exactly the 8 required fields in order: STATUS set to `pending VM run`, INSTALL_METHOD set to the literal `appimage`, all other fields set to `pending`. Each includes a `## Reproduction Notes` section with per-distro shell commands for obtaining each field value from a real VM run.

- `tests/distribution_proofs.rs` was extended (without modifying any existing helper or test) with a `FINAL_REQUIRED_FIELDS` constant (8 fields), an `assert_final_transcript(rel)` helper accepting `STATUS: ok`, `STATUS: pending VM run`, or any `STATUS: failed:` prefix, and three `#[test]` functions (`debian12_final_transcript_has_required_fields`, `fedora39_final_transcript_has_required_fields`, `arch_final_transcript_has_required_fields`). Existing `project_root()` and `read_file()` helpers were reused.

The architectural decision to use a separate `FINAL_REQUIRED_FIELDS` constant and `assert_final_transcript` helper (rather than extending the existing 7-field AppImage helper) keeps the two proof levels independently testable without coupling their field contracts.

Real VM runs converting `STATUS: pending VM run` → `STATUS: ok` are a human operator deliverable per MEM079 policy, as linuxdeploy/appimagetool are absent on this host.

## Verification

All 9 tests in `distribution_proofs` pass (6 pre-existing + 3 new final-UAT tests): `cargo test --test distribution_proofs -- --test-threads=1` exits 0. Full 44-test regression suite (`--test distribution_proofs --test packaging --test wizard_proofs --test ui_distribution -- --test-threads=1`) exits 0. All five slice-plan grep checks confirmed: `STATUS: pending VM run` present in all three transcripts, `STRATAGEM_FIRED` present in debian12 transcript, `INSTALL_METHOD: appimage` present in arch transcript.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

none

## Known Limitations

All three transcripts carry STATUS: pending VM run. Converting to ok requires human operator VM runs on real Debian 12, Fedora 39, and Arch systems with a published release AppImage — this cannot be automated on this host (linuxdeploy/appimagetool absent, MEM078).

## Follow-ups

Human operator task: on each target distro, follow the Reproduction Notes in the respective transcript.md to perform a real AppImage install end-to-end, fire a stratagem by voice, and update STATUS + all pending fields to their observed values. This converts the proof from structural to runtime-verified.

## Files Created/Modified

- `docs/distribution-proofs/final/README.md` — New: documents final-UAT proof directory layout, 8-field transcript format, STATUS values, per-distro FUSE2 package names, and reproduction steps
- `docs/distribution-proofs/final/debian12/transcript.md` — New: 8-field pending-VM-run transcript for Debian 12 with reproduction notes
- `docs/distribution-proofs/final/fedora39/transcript.md` — New: 8-field pending-VM-run transcript for Fedora 39 with reproduction notes
- `docs/distribution-proofs/final/arch/transcript.md` — New: 8-field pending-VM-run transcript for Arch with reproduction notes
- `tests/distribution_proofs.rs` — Extended: added FINAL_REQUIRED_FIELDS constant, assert_final_transcript helper, and 3 structural test functions
