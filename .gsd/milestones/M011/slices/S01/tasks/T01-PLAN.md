---
estimated_steps: 6
estimated_files: 1
skills_used: []
---

# T01: Rewrite wizard_proofs.rs per-distro tests for the four new M011 distros

Replace the three stale per-distro test functions in `tests/wizard_proofs.rs` (`debian12_wizard_transcript_has_required_fields`, `fedora39_wizard_transcript_has_required_fields`, `arch_wizard_transcript_has_required_fields`) with four new ones — `debian13_wizard_transcript_has_required_fields`, `ubuntu2604_wizard_transcript_has_required_fields`, `fedora44_wizard_transcript_has_required_fields`, `cachyos_wizard_transcript_has_required_fields` — each pointing at the matching `docs/distribution-proofs/wizard/{distro}/transcript.md` path. Reuse the existing `assert_transcript` helper, `REQUIRED_FIELDS`, and `VALID_STATUSES` constants without modification. Keep `wizard_readme_contains_four_scenario_headings` unchanged. Update the leading comment block to reference the four M011 target distros (mirroring the comment style already in `tests/distribution_proofs.rs`: 'Supported distros (M011): Debian 13, Ubuntu 26.04, Fedora 44, CachyOS'). Then run the full distribution-proof test surface to confirm everything passes against the existing transcripts.

Notes/assumptions:
- The four new wizard transcripts already exist at `docs/distribution-proofs/wizard/{debian13,ubuntu2604,fedora44,cachyos}/transcript.md` with STATUS: pending VM run and all 10 required fields present (verified at planning time).
- The old wizard directories (debian12, fedora39, arch) have already been removed from the working tree, so those tests would currently fail. This task makes the test file honest about that reality.
- `--test-threads=1` is required (per MEM080) to avoid shared-/tmp flakes in the distribution-proof integration suites.
- No transcript content is modified in this task; S02 owns transcript content updates.

## Inputs

- ``tests/wizard_proofs.rs` — current test file with stale debian12/fedora39/arch test functions`
- ``tests/distribution_proofs.rs` — reference for the four-distro naming pattern already in use for AppImage and final-UAT proofs`
- ``docs/distribution-proofs/wizard/debian13/transcript.md` — target transcript for the new debian13 test function`
- ``docs/distribution-proofs/wizard/ubuntu2604/transcript.md` — target transcript for the new ubuntu2604 test function`
- ``docs/distribution-proofs/wizard/fedora44/transcript.md` — target transcript for the new fedora44 test function`
- ``docs/distribution-proofs/wizard/cachyos/transcript.md` — target transcript for the new cachyos test function`

## Expected Output

- ``tests/wizard_proofs.rs` — rewritten so the four per-distro test functions reference the four M011 distros and the file contains zero references to debian12/fedora39/arch`

## Verification

cd /home/chadmin/Github/hd-linux-voice && cargo test --test distribution_proofs --test wizard_proofs -- --test-threads=1 2>&1 | tail -40 — all tests must pass, with exactly four wizard per-distro tests reported (debian13, ubuntu2604, fedora44, cachyos) plus `wizard_readme_contains_four_scenario_headings`. Then run `! grep -E 'debian12|fedora39|arch' tests/wizard_proofs.rs` to confirm zero references to the old distro names remain.
