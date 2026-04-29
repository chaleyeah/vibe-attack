---
id: S01
parent: M011
milestone: M011
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - ["tests/wizard_proofs.rs"]
key_decisions:
  - ["Kept REQUIRED_FIELDS, VALID_STATUSES, assert_transcript, and wizard_readme_contains_four_scenario_headings unchanged — task scope was strictly per-distro function replacement; these helpers are distro-agnostic"]
patterns_established:
  - ["tests/wizard_proofs.rs has a stable scaffold (shared helpers + README heading test) and a variable layer (four named per-distro test functions + their transcript paths) — only the variable layer changes on distro retargeting"]
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-29T01:13:38.341Z
blocker_discovered: false
---

# S01: Rename proof directories and update test harness

**Rewrote tests/wizard_proofs.rs to target the four M011 distros (Debian 13, Ubuntu 26.04, Fedora 44, CachyOS), removing all stale debian12/fedora39/arch references; all 16 distribution-proof tests pass.**

## What Happened

The wizard proof test suite (tests/wizard_proofs.rs) still referenced three distros that no longer exist in the working tree — debian12, fedora39, and arch. The four replacement wizard transcript files (debian13, ubuntu2604, fedora44, cachyos) were already in place at docs/distribution-proofs/wizard/{distro}/transcript.md with STATUS: pending VM run and all 10 required fields present.

Two surgical edits were made to tests/wizard_proofs.rs:
1. Added an M011 distro list comment (`// Supported distros (M011): Debian 13, Ubuntu 26.04, Fedora 44, CachyOS`) to the leading comment block, matching the convention already used in tests/distribution_proofs.rs.
2. Replaced the three stale per-distro #[test] functions (debian12_wizard_transcript_has_required_fields, fedora39_wizard_transcript_has_required_fields, arch_wizard_transcript_has_required_fields) with four new ones named debian13_wizard_transcript_has_required_fields, ubuntu2604_wizard_transcript_has_required_fields, fedora44_wizard_transcript_has_required_fields, cachyos_wizard_transcript_has_required_fields, each pointing at the matching transcript path.

The assert_transcript helper, REQUIRED_FIELDS, VALID_STATUSES constants, and the wizard_readme_contains_four_scenario_headings test were left entirely unchanged — they are distro-agnostic and needed no modification.

No transcript content was modified; that is S02's responsibility (filling in real STATUS: ok values via VM runs).

## Verification

Ran `cargo test --test distribution_proofs --test wizard_proofs -- --test-threads=1`: 16/16 tests passed — 11 distribution_proofs tests (covering appimage, final transcripts for all four distros, and build.sh checks) plus 5 wizard_proofs tests (four new per-distro functions + wizard_readme_contains_four_scenario_headings). Confirmed `! grep -E 'debian12|fedora39|arch' tests/wizard_proofs.rs` exits 0 — zero references to stale distro names remain.

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

All four wizard transcripts still carry STATUS: pending VM run — actual pass/fail proof requires real VM execution in S02. The structural test (field presence, valid status values) passes because VALID_STATUSES includes 'pending VM run' as an accepted value.

## Follow-ups

S02 must run wizard sessions on each of the four target distros and update transcript STATUS from 'pending VM run' to 'ok'.

## Files Created/Modified

- `tests/wizard_proofs.rs` — Replaced three stale per-distro test functions (debian12, fedora39, arch) with four new M011 ones (debian13, ubuntu2604, fedora44, cachyos); added M011 distro list comment
