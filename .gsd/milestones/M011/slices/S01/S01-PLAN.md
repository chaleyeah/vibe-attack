# S01: Rename proof directories and update test harness

**Goal:** Update tests/wizard_proofs.rs so its per-distro test functions match the four new M011 target distros (Debian 13, Ubuntu 26.04, Fedora 44, CachyOS), and confirm the full distribution-proof test surface (wizard + appimage + final) passes against the existing four-distro proof directory layout.
**Demo:** `cargo test --test distribution_proofs --test-threads=1` passes with the new four-distro names; old three-distro directories are removed; test function names and README per-distro sections updated.

## Must-Haves

- `cargo test --test distribution_proofs --test wizard_proofs -- --test-threads=1` passes. `tests/wizard_proofs.rs` defines exactly four per-distro test functions named `debian13_*`, `ubuntu2604_*`, `fedora44_*`, `cachyos_*`, with no remaining references to `debian12`, `fedora39`, or `arch`. The `wizard_readme_contains_four_scenario_headings` test still passes unchanged.

## Proof Level

- This slice proves: contract; real runtime not required (static structural tests over checked-in transcripts); no human/UAT required for this slice.

## Integration Closure

Upstream consumed: `docs/distribution-proofs/wizard/{debian13,ubuntu2604,fedora44,cachyos}/transcript.md` (already on disk with STATUS: pending VM run). New wiring: test function names in `tests/wizard_proofs.rs` reference the new four-distro paths. Remaining for milestone end-to-end: S02 will fill in real STATUS: ok values via VM runs; S03–S05 handle UI polish, version bump, and release.

## Verification

- Not provided.

## Tasks

- [x] **T01: Rewrite wizard_proofs.rs per-distro tests for the four new M011 distros** `est:30m`
  Replace the three stale per-distro test functions in `tests/wizard_proofs.rs` (`debian12_wizard_transcript_has_required_fields`, `fedora39_wizard_transcript_has_required_fields`, `arch_wizard_transcript_has_required_fields`) with four new ones — `debian13_wizard_transcript_has_required_fields`, `ubuntu2604_wizard_transcript_has_required_fields`, `fedora44_wizard_transcript_has_required_fields`, `cachyos_wizard_transcript_has_required_fields` — each pointing at the matching `docs/distribution-proofs/wizard/{distro}/transcript.md` path. Reuse the existing `assert_transcript` helper, `REQUIRED_FIELDS`, and `VALID_STATUSES` constants without modification. Keep `wizard_readme_contains_four_scenario_headings` unchanged. Update the leading comment block to reference the four M011 target distros (mirroring the comment style already in `tests/distribution_proofs.rs`: 'Supported distros (M011): Debian 13, Ubuntu 26.04, Fedora 44, CachyOS'). Then run the full distribution-proof test surface to confirm everything passes against the existing transcripts.

Notes/assumptions:
- The four new wizard transcripts already exist at `docs/distribution-proofs/wizard/{debian13,ubuntu2604,fedora44,cachyos}/transcript.md` with STATUS: pending VM run and all 10 required fields present (verified at planning time).
- The old wizard directories (debian12, fedora39, arch) have already been removed from the working tree, so those tests would currently fail. This task makes the test file honest about that reality.
- `--test-threads=1` is required (per MEM080) to avoid shared-/tmp flakes in the distribution-proof integration suites.
- No transcript content is modified in this task; S02 owns transcript content updates.
  - Files: `tests/wizard_proofs.rs`
  - Verify: cd /home/chadmin/Github/hd-linux-voice && cargo test --test distribution_proofs --test wizard_proofs -- --test-threads=1 2>&1 | tail -40 — all tests must pass, with exactly four wizard per-distro tests reported (debian13, ubuntu2604, fedora44, cachyos) plus `wizard_readme_contains_four_scenario_headings`. Then run `! grep -E 'debian12|fedora39|arch' tests/wizard_proofs.rs` to confirm zero references to the old distro names remain.

## Files Likely Touched

- tests/wizard_proofs.rs
