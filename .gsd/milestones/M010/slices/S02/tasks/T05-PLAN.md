---
estimated_steps: 9
estimated_files: 1
skills_used: []
---

# T05: Add tests/wizard_proofs.rs structural assertions for wizard transcripts

Mirror the structure of `tests/distribution_proofs.rs` (which validates AppImage transcripts) for the wizard transcripts created in T04. The test file is `tests/wizard_proofs.rs` (new).

Requirements:

1. Define `REQUIRED_FIELDS` slice including: `STATUS:`, `DISTRO:`, `KERNEL:`, `BINARY:`, `BINARY_VERSION:`, `SCENARIO_A:`, `SCENARIO_B:`, `SCENARIO_C:`, `SCENARIO_D:`, `STRATAGEM_FIRED:`.

2. Define `VALID_STATUSES` slice: `STATUS: ok`, `STATUS: pending VM run`, plus the four `STATUS: failed:scenario-{A,B,C,D}` values.

3. One test per distro (`debian12_wizard_transcript_has_required_fields`, `fedora39_*`, `arch_*`) — each reads the transcript and asserts every REQUIRED_FIELD substring is present and at least one VALID_STATUS line is present.

4. One test asserting `docs/distribution-proofs/wizard/README.md` exists and contains the substrings `Scenario A`, `Scenario B`, `Scenario C`, and `Scenario D` so future testers cannot accidentally drop the four-scenario structure.

5. Use `env!("CARGO_MANIFEST_DIR")` to locate the project root; do NOT depend on the test's CWD. Match the helper-function shape of `tests/distribution_proofs.rs` exactly (same `project_root`, `read_file`, `assert_transcript` pattern).

6. Per MEM080: this test file will run with `--test-threads=1` along with the other distribution_proofs/packaging/ui_distribution suites. It is purely file-IO so it will not flake, but follow the convention.

After writing, run `cargo test --test wizard_proofs -- --test-threads=1` and confirm all tests pass against the pending placeholders from T04.

## Inputs

- ``tests/distribution_proofs.rs` — structural blueprint to mirror (REQUIRED_FIELDS / VALID_STATUSES / assert_transcript pattern)`
- ``docs/distribution-proofs/wizard/debian12/transcript.md` — produced by T04, must validate against new tests`
- ``docs/distribution-proofs/wizard/fedora39/transcript.md` — produced by T04`
- ``docs/distribution-proofs/wizard/arch/transcript.md` — produced by T04`
- ``docs/distribution-proofs/wizard/README.md` — produced by T04, asserted to contain four-scenario substrings`

## Expected Output

- ``tests/wizard_proofs.rs` — new integration test file with 4+ tests (3 transcripts + 1 README structure check); all pass under --test-threads=1`

## Verification

cargo test --test wizard_proofs -- --test-threads=1 && cargo test --test ui_distribution -- --test-threads=1 && cargo test --test distribution_proofs -- --test-threads=1
