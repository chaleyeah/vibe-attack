---
estimated_steps: 11
estimated_files: 1
skills_used: []
---

# T03: Add config-screen PTT-key affordance and run dev-host smoke verification

Two parts.

Part A — Config-screen affordance: in src/bin/vibe-attack-config.rs `show_main_config` PTT row (lines 387–390), the current label 'PTT key: KEY_F13' has no indication that this is set in the wizard and that re-entering requires the wizard. Replace the single-line label with a horizontal layout: keep `ui.label(format!("PTT key: {}", app.config.ptt_binding))` and add `ui.weak("(configured in wizard)")` next to it. Do not add a 'Reconfigure…' button — re-entering the wizard from main config is out of scope per M008's explicit deferral and would expand the slice. The weak-text affordance is the minimum disambiguation that resolves S03-RESEARCH's 'users don't know they can't change it' issue without scope creep.

Part B — Dev-host smoke verification (final demo of the slice): execute and capture output for the following commands. Each must succeed; record the exit code and a one-line summary in the SUMMARY artifact at slice close.
  1. `cargo build --release --features gui --bin vibe-attack-config` — must compile cleanly with no warnings introduced by T01/T02.
  2. `cargo test --features gui --lib` — full lib test suite under gui feature must pass including the new tests added in T01 and T02.
  3. `cargo test --features gui --lib ui::wizard::tests::manual_key_persists_in_state ui::wizard::tests::manual_key_default_empty ui::tray::tests::tooltip_description_idle_ptt ui::tray::tests::tooltip_description_idle_wake ui::tray::tests::tray_handle_take_quit_request_clears_flag -- --exact` — explicitly named new tests must run and pass.
  4. `cargo test --test distribution_proofs -- --test-threads=1` — must remain green (sanity that nothing broke S01/S02 scaffolding).
  5. `cargo test --test wizard_proofs -- --test-threads=1` — must remain green.

Manual checks (record observations in summary, not asserted in CI):
  6. Launch `./target/release/vibe-attack-config --skip-wizard`. Confirm the main config screen renders, the new '(configured in wizard)' weak text appears next to the PTT key row, and tray icon (if D-Bus available — may be None on the auto-mode host) registers. Right-click tray (if present) → Quit → window closes without abrupt termination (look for 'Tray quit requested' in stderr).

No VM-run findings are populated in this slice — all four wizard transcripts remain STATUS: pending VM run. Document the 'wizard-finding triage' task as a Follow-up in S03-SUMMARY: 'Gated on at least one wizard/{distro}/transcript.md reaching SCENARIO_A: ok|failed:*. When unblocked, read all four transcripts' ## Findings sections, group by file/severity, and file as M012 candidate work.'

## Inputs

- `src/bin/vibe-attack-config.rs`
- `src/ui/wizard.rs`
- `src/ui/tray.rs`

## Expected Output

- `src/bin/vibe-attack-config.rs`

## Verification

cargo build --release --features gui --bin vibe-attack-config && cargo test --features gui --lib && cargo test --test distribution_proofs -- --test-threads=1 && cargo test --test wizard_proofs -- --test-threads=1

## Observability Impact

Smoke run on dev host produces stderr containing 'Tray quit requested' — that string is the new operational signal indicating clean shutdown via tray (vs SIGTERM or window-X). distribution_proofs and wizard_proofs continuing to pass is the regression signal proving S01/S02 scaffolding is intact.
