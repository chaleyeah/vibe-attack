---
id: T01
parent: S01
milestone: M011
key_files:
  - tests/wizard_proofs.rs
key_decisions:
  - Kept REQUIRED_FIELDS, VALID_STATUSES, assert_transcript, and wizard_readme_contains_four_scenario_headings unchanged — task scope was strictly per-distro function replacement
duration: 
verification_result: passed
completed_at: 2026-04-29T01:12:38.534Z
blocker_discovered: false
---

# T01: Rewrote tests/wizard_proofs.rs to use the four M011 target distros (Debian 13, Ubuntu 26.04, Fedora 44, CachyOS), replacing stale debian12/fedora39/arch functions

**Rewrote tests/wizard_proofs.rs to use the four M011 target distros (Debian 13, Ubuntu 26.04, Fedora 44, CachyOS), replacing stale debian12/fedora39/arch functions**

## What Happened

The existing `tests/wizard_proofs.rs` had three per-distro test functions pointing at `wizard/debian12`, `wizard/fedora39`, and `wizard/arch` directories that no longer exist. The four new wizard proof directories (debian13, ubuntu2604, fedora44, cachyos) were already in place at planning time.

Two surgical edits were made:
1. Added the M011 distro list comment `// Supported distros (M011): Debian 13, Ubuntu 26.04, Fedora 44, CachyOS` to the leading comment block, matching the style used in `tests/distribution_proofs.rs`.
2. Replaced the three stale `#[test]` functions (debian12, fedora39, arch) with four new ones: `debian13_wizard_transcript_has_required_fields`, `ubuntu2604_wizard_transcript_has_required_fields`, `fedora44_wizard_transcript_has_required_fields`, `cachyos_wizard_transcript_has_required_fields`. Each points at the matching `docs/distribution-proofs/wizard/{distro}/transcript.md`.

The `assert_transcript` helper, `REQUIRED_FIELDS`, `VALID_STATUSES` constants, and the `wizard_readme_contains_four_scenario_headings` test were all left untouched as specified.

## Verification

Ran `cargo test --test distribution_proofs --test wizard_proofs -- --test-threads=1`: all 16 tests passed (11 distribution_proofs + 5 wizard_proofs, including the four new per-distro functions and `wizard_readme_contains_four_scenario_headings`). Then confirmed `! grep -E 'debian12|fedora39|arch' tests/wizard_proofs.rs` exits 0, confirming zero references to old distro names.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test distribution_proofs --test wizard_proofs -- --test-threads=1 2>&1 | tail -40` | 0 | ✅ pass — 16/16 tests passed | 260ms |
| 2 | `! grep -E 'debian12|fedora39|arch' tests/wizard_proofs.rs && echo clean` | 0 | ✅ pass — no stale distro names remain | 5ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `tests/wizard_proofs.rs`
