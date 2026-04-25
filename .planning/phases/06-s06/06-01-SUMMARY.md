---
phase: "06"
plan: "01"
---

# T01: Created tests/documentation.rs with 11 structural tests defining the doc contracts for README.md, CONTRIBUTING.md, docs/troubleshooting.md, and docs/configuration.md

**Created tests/documentation.rs with 11 structural tests defining the doc contracts for README.md, CONTRIBUTING.md, docs/troubleshooting.md, and docs/configuration.md**

## What Happened

Wrote `tests/documentation.rs` from scratch following the exact `env!("CARGO_MANIFEST_DIR")` structural test pattern established in `tests/ui_distribution.rs`. No reference file existed at the target path before execution.

The 11 tests implement the full TDD contract for S06:
1. `readme_exists` — asserts README.md is present
2. `readme_has_installation_section` — asserts `## Installation` heading
3. `readme_has_usage_section` — asserts `## Usage` or `## Running`
4. `readme_has_correct_project_name` — asserts `hd-linux-voice` present and `vibe-attack` absent
5. `readme_does_not_reference_portaudio` — regression guard for removed dependency
6. `troubleshooting_doc_exists` — asserts docs/troubleshooting.md present
7. `troubleshooting_has_uinput_section` — case-insensitive search for `uinput`
8. `contributing_exists` — asserts CONTRIBUTING.md present
9. `contributing_has_build_section` — asserts `cargo build` or `## Build`
10. `configuration_doc_exists` — asserts docs/configuration.md present
11. `configuration_has_ptt_section` — case-insensitive search for `ptt`

All tests use only `std::fs` and `std::path` from the standard library — no `use` imports needed. File/content checks follow the same idioms as the S05 packaging tests. Tests are intentionally failing until T02 and T03 create the documented files.

## Verification

Ran `grep -c '#[test]' tests/documentation.rs` which returned 11, meeting the >= 11 requirement from the task plan. File exists at `tests/documentation.rs` and all test functions are syntactically correct Rust. Cargo compile check was blocked by shell policy; syntactic correctness confirmed by manual review against the established pattern from ui_distribution.rs.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `grep -c '#\[test\]' tests/documentation.rs` | 0 | ✅ pass — 11 test functions present (>= 11 required) | 50ms |

## Deviations

none

## Known Issues

Tests will fail until T02 (docs + README) and T03 (verification pass) complete — this is expected per the TDD approach.

## Files Created/Modified

- `tests/documentation.rs`
