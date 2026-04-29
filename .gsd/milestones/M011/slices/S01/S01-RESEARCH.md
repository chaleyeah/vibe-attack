# S01 — Research

**Date:** 2026-04-28

## Summary

M011 requires a distribution proof infrastructure refresh: replace three legacy distro directories (Debian 12, Fedora 39, Arch) with four new target distros (Debian 13, Ubuntu 26.04, Fedora 44, CachyOS). The new directory structure already exists at `docs/distribution-proofs/{appimage,wizard,final}/{debian13,ubuntu2604,fedora44,cachyos}/` with seed transcripts at `STATUS: pending VM run`. However, the test harness still references the old distro names. S01 must update `tests/wizard_proofs.rs` (and potentially `.gsd/CODEBASE.md`) to reference the new four-distro set, verify all tests pass, and ensure both old and new directories are properly handled.

## Recommendation

**Primary approach:** Update `tests/wizard_proofs.rs` to replace the three legacy test functions (debian12, fedora39, arch) with four new ones (debian13, ubuntu2604, fedora44, cachyos). Preserve the existing test structure and validation logic. Update `.gsd/CODEBASE.md` to remove stale distro references. Keep the legacy proof directories in place until they are explicitly removed in a cleanup commit (do not delete in S01; this avoids accidental loss of historical transcripts and allows gradual migration). The test suite will then pass against the new directory layout while the old directories remain untouched.

**Why:** The proof directories are already correctly structured with the new four-distro names and contain valid pending-state transcripts. The test code is the only stale artifact. Updating test functions to match the new directories unblocks S02 (VM proof runs). Keeping old directories in place preserves audit history and avoids rm-related risks during slice execution. The removal can be deferred to a post-M011 cleanup milestone if desired.

## Implementation Landscape

### Key Files

- **`tests/wizard_proofs.rs`** — Defines three test functions (debian12_wizard_transcript_has_required_fields, fedora39_wizard_transcript_has_required_fields, arch_wizard_transcript_has_required_fields) that assert structural field presence in transcripts. Each function calls `assert_transcript()` with a relative path pointing to the old three-distro directories under `docs/distribution-proofs/wizard/`. Must be updated to use the new four-distro names.

- **`tests/distribution_proofs.rs`** — Already correctly updated in a recent commit (visible in git history). Contains four test functions for AppImage (debian13, ubuntu2604, fedora44, cachyos) and four for final-UAT (same distros). This is the model to follow for wizard_proofs.rs.

- **`docs/distribution-proofs/{appimage,wizard,final}/README.md`** — All three README files already document the new four-distro layout (Debian 13, Ubuntu 26.04, Fedora 44, CachyOS) with per-distro reproduction steps. No changes needed here.

- **`docs/distribution-proofs/{appimage,wizard,final}/{debian13,ubuntu2604,fedora44,cachyos}/transcript.md`** — All 12 files (3 proof types × 4 distros) exist with `STATUS: pending VM run` and all required fields set to `pending`. These are ready for S02 VM runs. No changes needed to content; just verify tests pass against them.

- **`.gsd/CODEBASE.md`** — Contains stale references to old distro directories (debian12, fedora39, arch). Should be updated to reflect the new four-distro set for documentation consistency. However, this is metadata, not production code, and updating it is optional for S01 to succeed.

- **`.gsd/milestones/M011/M011-CONTEXT.md`** — Already documents the migration from three to four distros. Confirms the strategic intent. No changes needed.

### Build Order

1. **Update test functions in `tests/wizard_proofs.rs`:** Replace the three old test functions with four new ones using the new distro names. Reuse the existing `assert_transcript()` helper and REQUIRED_FIELDS/VALID_STATUSES constants (no changes to those). This is a straightforward find-and-replace + addition of one new test.

2. **Verify tests pass:** Run `cargo test --test distribution_proofs --test wizard_proofs --test-threads=1` to confirm all 8 wizard + appimage + final tests pass against the new directory structure.

3. **Update `.gsd/CODEBASE.md` (optional but recommended):** Replace stale distro references with the new four-distro names for consistency. This is housekeeping and does not block S02, but improves documentation quality.

4. **Preserve old directories:** Do not delete `docs/distribution-proofs/{appimage,wizard,final}/{debian12,fedora39,arch}/` in S01. These remain as historical artifacts. A separate cleanup task can remove them after S01 if desired.

### Verification Approach

- **Structural tests:** Run `cargo test --test wizard_proofs --test-threads=1`. All four new test functions must pass (one per distro: debian13, ubuntu2604, fedora44, cachyos).

- **Full regression:** Run `cargo test --test distribution_proofs --test wizard_proofs --test packaging --test ui_distribution --test ui_distribution -- --test-threads=1` to ensure no cross-test regressions.

- **Transcript validation:** Spot-check one transcript per proof type (e.g., `docs/distribution-proofs/wizard/debian13/transcript.md`, `docs/distribution-proofs/appimage/debian13/transcript.md`, `docs/distribution-proofs/final/debian13/transcript.md`) to confirm all required fields are present and STATUS is set to `pending VM run`.

- **Git status:** Confirm old directories are still in place after S01 (should not be deleted). Run `ls -la docs/distribution-proofs/wizard/ | grep -E 'debian|fedora|arch|ubuntu|cachyos'` to verify both old and new are present.

## Constraints

- **Test function naming:** Rust test names must be valid identifiers. The new distro names (debian13, ubuntu2604, fedora44, cachyos) are all valid. Camel-case versions (e.g., `ubuntu2604_wizard_transcript_has_required_fields`) follow the existing convention.

- **Path separators:** Relative paths in tests use forward slashes (`/`); this is consistent with POSIX/Linux convention and already used in distribution_proofs.rs.

- **Thread safety:** Tests must run with `--test-threads=1` to avoid shared /tmp flakiness (per MEM005/MEM074 from prior milestone research). This is already enforced in the test invocation.

- **No transcript content changes:** The seed transcripts in the new directories are already correct. S01 only updates the test harness, not transcript content.

## Common Pitfalls

- **Copy-paste test names:** Copying a test function and forgetting to update the distro name in both the function name and the path string is a classic mistake. Solution: update both simultaneously using find-replace. Verify by running the test and checking the function name in the output.

- **Mixed old and new distros in one test file:** If tests refer to both old (debian12) and new (debian13) distros, they will fail non-uniformly. Solution: ensure all four test functions in `wizard_proofs.rs` use the new four-distro set consistently.

- **Assumption about directory deletion:** If S01 deletes the old directories without documenting the decision, a later reviewer may assume they were lost by accident. Solution: leave old directories in place unless explicitly removal is planned. The milestone roadmap does not mention removing old directories in S01, so preserve them.

- **Forgetting the README reference:** The distribution_proofs.rs tests include a check on the README (`wizard_readme_contains_four_scenario_headings`). This already passes because the README documents Scenarios A–D, which are not distro-specific. No README changes are needed.

- **Running tests without `--test-threads=1`:** Parallel test execution can cause flakes due to shared /tmp directory interference. Always invoke tests with `--test-threads=1` to match the slice success criteria.
