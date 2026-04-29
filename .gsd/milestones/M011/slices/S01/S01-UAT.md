# S01: Rename proof directories and update test harness — UAT

**Milestone:** M011
**Written:** 2026-04-29T01:13:38.341Z

# S01: Rename proof directories and update test harness — UAT

**Milestone:** M011
**Written:** 2026-04-28

## UAT Type

- UAT mode: artifact-driven
- Why this mode is sufficient: This slice delivers static structural tests over checked-in transcripts. No runtime is required — correctness is proven by the test suite passing and the absence of stale distro references.

## Preconditions

- Working tree is at the state produced by T01 (tests/wizard_proofs.rs rewritten)
- All four wizard transcript files exist: docs/distribution-proofs/wizard/{debian13,ubuntu2604,fedora44,cachyos}/transcript.md
- Rust toolchain available (`cargo`)

## Smoke Test

Run `cargo test --test wizard_proofs -- --test-threads=1 2>&1 | tail -10` — must show `5 passed; 0 failed`.

## Test Cases

### 1. All four new per-distro wizard tests pass

```
cargo test --test distribution_proofs --test wizard_proofs -- --test-threads=1 2>&1 | tail -40
```

1. Run the command above
2. Observe test output for wizard_proofs binary
3. **Expected:** `running 5 tests` with `debian13_wizard_transcript_has_required_fields`, `ubuntu2604_wizard_transcript_has_required_fields`, `fedora44_wizard_transcript_has_required_fields`, `cachyos_wizard_transcript_has_required_fields`, and `wizard_readme_contains_four_scenario_headings` all reporting `ok`. Final line: `test result: ok. 5 passed; 0 failed`.

### 2. distribution_proofs tests still pass (no regression)

1. From the same test run output, inspect the distribution_proofs binary section
2. **Expected:** `running 11 tests` with all 11 reporting `ok`. Final line: `test result: ok. 11 passed; 0 failed`.

### 3. No stale distro names remain in wizard_proofs.rs

```
! grep -E 'debian12|fedora39|arch' tests/wizard_proofs.rs && echo clean
```

1. Run the command above
2. **Expected:** exits 0 and prints `clean`. Any match would indicate a leftover stale reference.

### 4. Four new test function names are present

```
grep -E 'fn (debian13|ubuntu2604|fedora44|cachyos)_wizard' tests/wizard_proofs.rs
```

1. Run the command above
2. **Expected:** Exactly 4 lines returned, one per new distro.

## Edge Cases

### wizard_readme_contains_four_scenario_headings still passes

1. Run `cargo test --test wizard_proofs wizard_readme_contains_four_scenario_headings -- --test-threads=1`
2. **Expected:** `test result: ok. 1 passed` — this test is invariant to distro renaming and must not have been touched.

### Transcripts with STATUS: pending VM run are accepted

1. Inspect any wizard transcript: `head -5 docs/distribution-proofs/wizard/debian13/transcript.md`
2. **Expected:** STATUS field is present and set to `pending VM run` — the test accepts this value as VALID_STATUSES includes it.

## Failure Signals

- Any `FAILED` in `cargo test` output indicates a broken transcript field or missing file
- `grep` finding debian12/fedora39/arch in tests/wizard_proofs.rs indicates incomplete replacement
- Fewer than 5 tests reported in wizard_proofs run indicates a missing test function

## Not Proven By This UAT

- That the wizard actually runs correctly on each distro (deferred to S02 VM runs)
- That transcripts contain accurate/complete content beyond required field presence
- That the AppImage or final proof paths are correctly populated (covered by distribution_proofs tests, not wizard_proofs)

## Notes for Tester

The four wizard transcripts all currently have `STATUS: pending VM run` — this is intentional and correct for this slice. S02 will replace these with real `STATUS: ok` values after VM execution. The structural tests pass because `pending VM run` is in VALID_STATUSES.
