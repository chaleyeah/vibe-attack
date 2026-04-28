---
estimated_steps: 19
estimated_files: 4
skills_used: []
---

# T04: Seed docs/distribution-proofs/wizard/ structure and three pending transcripts

Create the wizard UAT proof directory structure following the same MEM079/MEM081 convention used for `docs/distribution-proofs/appimage/`. The directory does not exist yet.

Create the following:

1. `docs/distribution-proofs/wizard/README.md` — copy the format/policy framing from `docs/distribution-proofs/appimage/README.md`, adapted for wizard scenarios. Document the four required UAT scenarios (A: fresh install, B: partial state with model pre-placed, C: relaunch / wizard skipped, D: --skip-wizard flag) per S02-RESEARCH.md. Document the per-distro reproduction steps. Document the STATUS values: `ok`, `pending VM run`, `failed:scenario-A`, `failed:scenario-B`, `failed:scenario-C`, `failed:scenario-D`.

2. `docs/distribution-proofs/wizard/debian12/transcript.md` — pending placeholder with these required structured fields (one per line):
   - `STATUS: pending VM run`
   - `DISTRO: pending`
   - `KERNEL: pending`
   - `BINARY: vibe-attack-config`
   - `BINARY_VERSION: pending`
   - `SCENARIO_A: pending` (fresh install)
   - `SCENARIO_B: pending` (model pre-placed)
   - `SCENARIO_C: pending` (relaunch skips wizard)
   - `SCENARIO_D: pending` (--skip-wizard flag)
   - `STRATAGEM_FIRED: pending` (whether at least one stratagem was fired by voice in scenario A)
   Followed by a free-form `## Reproduction Notes` section with bullet-point steps a tester would follow.

3. `docs/distribution-proofs/wizard/fedora39/transcript.md` — same structure, pending placeholders.

4. `docs/distribution-proofs/wizard/arch/transcript.md` — same structure, pending placeholders.

IMPORTANT: do NOT mark these as `STATUS: ok` — they are pending real VM runs. Use `pending VM run` per MEM079. The tests in T05 will accept that status. Real `ok` runs are completed in S06 (final UAT) per the milestone roadmap.

All four files must be tracked in git (no .gitignore matches under docs/).

## Inputs

- ``docs/distribution-proofs/appimage/README.md` — existing format/policy template to adapt`
- ``.gsd/milestones/M010/slices/S02/S02-RESEARCH.md` — defines the four UAT scenarios A/B/C/D`
- ``tests/distribution_proofs.rs` — shows the pending-run policy (MEM079) the wizard transcripts must follow`

## Expected Output

- ``docs/distribution-proofs/wizard/README.md` — format spec, scenario list, per-distro reproduction commands`
- ``docs/distribution-proofs/wizard/debian12/transcript.md` — pending placeholder transcript with all required fields`
- ``docs/distribution-proofs/wizard/fedora39/transcript.md` — pending placeholder transcript with all required fields`
- ``docs/distribution-proofs/wizard/arch/transcript.md` — pending placeholder transcript with all required fields`

## Verification

test -f docs/distribution-proofs/wizard/README.md && for d in debian12 fedora39 arch; do test -f docs/distribution-proofs/wizard/$d/transcript.md && grep -q '^STATUS:' docs/distribution-proofs/wizard/$d/transcript.md && grep -q '^SCENARIO_A:' docs/distribution-proofs/wizard/$d/transcript.md && grep -q '^SCENARIO_D:' docs/distribution-proofs/wizard/$d/transcript.md; done
