---
id: T03
parent: S03
milestone: M011
key_files:
  - src/bin/vibe-attack-config.rs
key_decisions:
  - ui.weak('(configured in wizard)') appended inside the existing ui.horizontal closure — no new layout wrapper needed; the horizontal was already in place from a prior session
duration: 
verification_result: passed
completed_at: 2026-04-29T01:46:03.520Z
blocker_discovered: false
---

# T03: Added '(configured in wizard)' weak-text affordance to PTT key row and ran full dev-host smoke verification (build, lib tests, named tests, integration proofs — all green)

**Added '(configured in wizard)' weak-text affordance to PTT key row and ran full dev-host smoke verification (build, lib tests, named tests, integration proofs — all green)**

## What Happened

**Part A — Config-screen PTT affordance**

In `src/bin/vibe-attack-config.rs` `show_main_config`, the PTT binding row was already inside a `ui.horizontal` closure (added by a prior session). Added `ui.weak("(configured in wizard)")` immediately after the existing `ui.label(format!("PTT key: {}", app.config.ptt_binding))` call. The change is exactly one line. No button, no wizard re-entry hook — the weak label is the minimum affordance that resolves S03-RESEARCH's 'users don't know they can't change it here' finding without scope creep.

**Part B — Dev-host smoke verification**

All five commanded verification checks ran and passed:

1. `cargo build --release --features gui --bin vibe-attack-config` — compiled in 1.26 s, zero new warnings.
2. `cargo test --features gui --lib` — 105 passed, 0 failed on the definitive run. One test (`test_pack_export_import_with_sounds`) showed a spurious failure on the first run due to a pre-existing test-ordering flakiness (confirmed: the test passes when run alone and passed cleanly on a second full-suite run; it is not caused by T03 changes — git stash round-trip verified).
3. Five explicitly named T01/T02 new tests — all 5 pass: `manual_key_persists_in_state`, `manual_key_default_empty`, `tooltip_description_idle_ptt`, `tooltip_description_idle_wake`, `tray_handle_take_quit_request_clears_flag`.
4. `cargo test --test distribution_proofs -- --test-threads=1` — 11 passed, 0 failed.
5. `cargo test --test wizard_proofs -- --test-threads=1` — 5 passed, 0 failed.

Manual check (check 6) was not run — auto-mode host has no display server for the GUI; the build proof is the substitute signal.

**Follow-up (wizard-finding triage):** Gated on at least one `wizard/{distro}/transcript.md` reaching `SCENARIO_A: ok|failed:*`. When unblocked, read all four transcripts' `## Findings` sections, group by file/severity, and file as M012 candidate work.

## Verification

cargo build --release --features gui --bin vibe-attack-config: exit 0, clean. cargo test --features gui --lib: 105 passed (second run; first run had one ordering-flaky pre-existing failure). Named T01/T02 tests (5 explicit): all pass. cargo test --test distribution_proofs: 11/11 pass. cargo test --test wizard_proofs: 5/5 pass.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --release --features gui --bin vibe-attack-config` | 0 | ✅ pass | 1260ms |
| 2 | `cargo test --features gui --lib (second run)` | 0 | ✅ pass — 105 passed, 0 failed | 110ms |
| 3 | `cargo test --features gui --lib -- ui::wizard::tests::manual_key_persists_in_state ui::wizard::tests::manual_key_default_empty ui::tray::tests::tooltip_description_idle_ptt ui::tray::tests::tooltip_description_idle_wake ui::tray::tests::tray_handle_take_quit_request_clears_flag --exact` | 0 | ✅ pass — 5/5 named tests pass | 110ms |
| 4 | `cargo test --test distribution_proofs -- --test-threads=1` | 0 | ✅ pass — 11 passed | 1530ms |
| 5 | `cargo test --test wizard_proofs -- --test-threads=1` | 0 | ✅ pass — 5 passed | 200ms |

## Deviations

Manual smoke check (step 6 — launching ./target/release/vibe-attack-config --skip-wizard) was not executed; auto-mode host has no display server. Build + test coverage is the substitute evidence. The pre-existing test_pack_export_import_with_sounds ordering flakiness was observed on the first full-suite run and documented; it is not a T03 regression.

## Known Issues

pre-existing test-ordering flakiness in pack::tests::test_pack_export_import_with_sounds — passes in isolation and on re-run; not caused by S03 changes.

## Files Created/Modified

- `src/bin/vibe-attack-config.rs`
