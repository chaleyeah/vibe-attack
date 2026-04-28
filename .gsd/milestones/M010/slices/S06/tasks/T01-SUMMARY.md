---
id: T01
parent: S06
milestone: M010
key_files:
  - docs/distribution-proofs/final/README.md
  - docs/distribution-proofs/final/debian12/transcript.md
  - docs/distribution-proofs/final/fedora39/transcript.md
  - docs/distribution-proofs/final/arch/transcript.md
  - tests/distribution_proofs.rs
key_decisions:
  - assert_final_transcript uses a distinct 8-field FINAL_REQUIRED_FIELDS constant rather than extending the existing 7-field REQUIRED_FIELDS, keeping the two proof levels independently testable without coupling.
  - STATUS: failed:<reason> acceptance is implemented via a line-prefix check (l.starts_with('STATUS: failed:')) rather than a substring match, matching the pattern used for the existing VALID_STATUSES array.
duration: 
verification_result: passed
completed_at: 2026-04-28T11:22:36.877Z
blocker_discovered: false
---

# T01: Scaffold docs/distribution-proofs/final/ with three pending VM-run transcripts and add assert_final_transcript helper + 3 structural tests to distribution_proofs.rs

**Scaffold docs/distribution-proofs/final/ with three pending VM-run transcripts and add assert_final_transcript helper + 3 structural tests to distribution_proofs.rs**

## What Happened

Created the final-UAT proof directory mirroring the existing appimage/ and wizard/ patterns.

Step 1 — README: Wrote `docs/distribution-proofs/final/README.md` documenting the directory layout, all 8 transcript fields with how-to-obtain instructions, STATUS values (ok / pending VM run / failed:<reason>), per-distro reproduction recipes (including the correct FUSE2 package name per distro: libfuse2 on Debian, fuse-libs on Fedora, fuse2 on Arch), and the MEM079 pending-transcript policy.

Step 2 — Transcripts: Wrote three files, each with exactly the 8 required fields in order. STATUS is `pending VM run` per MEM079; INSTALL_METHOD is `appimage` (the one non-pending literal field per the plan). Each file includes a `## Reproduction Notes` section with per-distro commands for obtaining each field value from a real VM run.

Step 3 — Tests: Appended to `tests/distribution_proofs.rs` without touching any existing helpers or tests. Added `FINAL_REQUIRED_FIELDS` constant (8 fields), `assert_final_transcript(rel)` helper that checks all 8 fields and accepts `STATUS: ok`, `STATUS: pending VM run`, or any line starting with `STATUS: failed:`, and three `#[test]` functions (`debian12_final_transcript_has_required_fields`, `fedora39_final_transcript_has_required_fields`, `arch_final_transcript_has_required_fields`). Reused existing `project_root()` and `read_file()` helpers.

Step 4 — Verification: All 9 tests in distribution_proofs (6 prior + 3 new) pass. Full regression suite (44 tests across distribution_proofs, packaging, wizard_proofs, ui_distribution) passes. All five grep checks from the slice verification spec also pass.

## Verification

Ran `cargo test --test distribution_proofs -- --test-threads=1`: 9/9 tests pass (6 pre-existing + 3 new final-UAT tests). Ran full regression guard `cargo test --test distribution_proofs --test packaging --test wizard_proofs --test ui_distribution -- --test-threads=1`: 44/44 tests pass. Ran all five grep checks from the slice verification spec: STATUS field, STRATAGEM_FIRED field, and INSTALL_METHOD: appimage all confirmed present in the correct transcripts.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test distribution_proofs -- --test-threads=1` | 0 | ✅ pass | 260ms |
| 2 | `cargo test --test distribution_proofs --test packaging --test wizard_proofs --test ui_distribution -- --test-threads=1` | 0 | ✅ pass | 90ms |
| 3 | `grep -q 'STATUS: pending VM run' docs/distribution-proofs/final/debian12/transcript.md` | 0 | ✅ pass | 5ms |
| 4 | `grep -q 'STATUS: pending VM run' docs/distribution-proofs/final/fedora39/transcript.md` | 0 | ✅ pass | 5ms |
| 5 | `grep -q 'STATUS: pending VM run' docs/distribution-proofs/final/arch/transcript.md` | 0 | ✅ pass | 5ms |
| 6 | `grep -q 'STRATAGEM_FIRED' docs/distribution-proofs/final/debian12/transcript.md` | 0 | ✅ pass | 5ms |
| 7 | `grep -q 'INSTALL_METHOD: appimage' docs/distribution-proofs/final/arch/transcript.md` | 0 | ✅ pass | 5ms |

## Deviations

none

## Known Issues

none — all three transcripts carry STATUS: pending VM run; converting to ok requires human operator VM runs per MEM079 policy.

## Files Created/Modified

- `docs/distribution-proofs/final/README.md`
- `docs/distribution-proofs/final/debian12/transcript.md`
- `docs/distribution-proofs/final/fedora39/transcript.md`
- `docs/distribution-proofs/final/arch/transcript.md`
- `tests/distribution_proofs.rs`
