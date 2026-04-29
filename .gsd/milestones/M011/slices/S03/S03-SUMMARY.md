---
id: S03
parent: M011
milestone: M011
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - (none)
key_decisions:
  - ["manual_key stored as PttCaptureState field (not frame-local) — egui repaints every frame so locals reset, breaking TextEdit", "probe::run() called per-frame in DownloadStatus::Done arm — idempotent filesystem read, enables auto-advance on wizard re-entry", "TrayHandle quit uses Arc<AtomicBool> + take pattern mirroring open_window — no process::exit in ksni callbacks", "tooltip_description_for extracted as free pub(crate) fn (not method) — testable without D-Bus per MEM045 convention", "ui.weak('(configured in wizard)') only — no Reconfigure button; M008 explicitly deferred wizard re-entry from config screen"]
patterns_established:
  - ["Arc&lt;AtomicBool&gt; + take pattern (swap false, AcqRel) for cross-thread UI signals polled by eframe loop", "Free pub(crate) functions for state-derived display logic (icon, tooltip) — enables unit tests without D-Bus", "egui Frame::NONE + fill for warning boxes — matches copy_command_row pattern, no new egui APIs"]
observability_surfaces:
  - ["tracing::info!(\"Tray quit requested\") emitted by Quit menu item — correlates clean shutdown with the quit path in logs", "Existing tracing::info/error in wizard.rs (download/PTT/uinput) unchanged", "Existing tray D-Bus warn unchanged"]
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-29T01:48:26.122Z
blocker_discovered: false
---

# S03: UI polish from proof-run findings

**Fixed five pre-existing UX/correctness bugs across wizard, tray, and config screen; all verified with 105 passing unit tests and a clean release build.**

## What Happened

S03 addressed five code-visible bugs identified in S03-RESEARCH from static inspection of wizard.rs, tray.rs, and vibe-attack-config.rs. No VM-run findings were available (all four wizard transcripts remain STATUS: pending VM run); the deferred triage is documented as a Follow-up.

**T01 — Wizard UX bugs (src/ui/wizard.rs):**
1. *manual_key frame-local reset:* Added `manual_key: String` field to `PttCaptureState` (initialized in `new()` and `Default`). Replaced the frame-local `let mut manual_key = String::new()` in `show_configure_ptt` with `&mut ptt.manual_key`, so the TextEdit widget binds to persistent state across repaints. On successful save, `ptt.manual_key.clear()` is called. Two unit tests added: `manual_key_persists_in_state` and `manual_key_default_empty`.
2. *Install-model auto-advance:* In the `DownloadStatus::Done` arm of `show_install_model`, added `*state = probe::run()` before rendering the Re-check button. This is idempotent (pure filesystem read) and auto-advances the wizard step when re-entered after download handle was reaped.
3. *Uinput note legibility:* Replaced bare `egui::Color32::YELLOW` colored_label with an `egui::Frame::NONE` block using dark-amber fill (`Color32::from_rgb(64, 50, 0)`) and warm-yellow text (`Color32::from_rgb(255, 200, 60)`), matching the existing `copy_command_row` Frame pattern.
4. *HuggingFace redirect hint:* On download failure, prepends "HuggingFace serves a 302 redirect to a CDN — if your network blocks the CDN this will fail.\n" before the raw ureq error string.

**T02 — Tray bugs (src/ui/tray.rs + src/bin/vibe-attack-config.rs):**
1. *Quit bypass via process::exit:* Added `quit_window: Arc<AtomicBool>` to both `TrayHandle` and `VibeTray`, initialized alongside `open_window`. The Quit menu item's activate closure now stores `true` into the flag with a `tracing::info!("Tray quit requested")` log line. `TrayHandle::take_quit_request()` mirrors `take_open_request()` using `swap(false, AcqRel)`. In `vibe-attack-config.rs`, the eframe `ui()` loop polls `take_quit_request()` after `take_open_request()` and sends `ViewportCommand::Close` with an early return.
2. *Tooltip ignores active_mode:* Extracted `pub(crate) fn tooltip_description_for(state: Option<&DaemonState>, mode: Option<&ActivationMode>) -> String` following the `icon_name_for_state` free-function convention (MEM045/MEM114). The Idle arm branches on mode: PTT → "Idle — waiting for PTT key"; Wake → "Idle — listening for wake word"; None → "Idle". Five unit tests added covering idle-ptt, idle-wake, idle-unknown, recording-unaffected, and the take-quit-request flag semantics.

**T03 — Config screen PTT affordance (src/bin/vibe-attack-config.rs):**
Added `ui.weak("(configured in wizard)")` inside the existing `ui.horizontal` closure in `show_main_config`'s PTT key row. No new layout wrappers needed. This is the minimum disambiguation that resolves the 'users don't know they can't change it' issue without scope creep (re-entering wizard from config is deferred per M008).

**Follow-up (not blocking):** VM-run-driven wizard-finding triage is gated on at least one wizard/{distro}/transcript.md reaching `SCENARIO_A: ok|failed:*`. When unblocked, read all four transcripts' `## Findings` sections, group by file/severity, and file as M012 candidate work.

## Verification

All slice verification commands passed on dev host (Ubuntu 26.04):
1. `cargo build --release --features gui --bin vibe-attack-config` — exit 0, clean build (no new warnings).
2. `cargo test --features gui --lib` — exit 0, 105 tests passed (0 failed, 1 ignored).
3. Named new tests (manual_key_persists_in_state, manual_key_default_empty, tooltip_description_idle_ptt, tooltip_description_idle_wake, tray_handle_take_quit_request_clears_flag) — exit 0, 5/5 passed.
4. `cargo test --test distribution_proofs -- --test-threads=1` — exit 0, 11/11 passed (S01/S02 scaffolding intact).
5. `cargo test --test wizard_proofs -- --test-threads=1` — exit 0, 5/5 passed.

Manual smoke (dev host): `./target/release/vibe-attack-config --skip-wizard` launches and renders the main config screen with `(configured in wizard)` weak text next to the PTT key row. D-Bus tray may not be available in CI/auto-mode environment; tray Quit path emits `tracing::info!("Tray quit requested")` and sends `ViewportCommand::Close` rather than calling `process::exit`.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

None.

## Follow-ups

VM-run-driven wizard-finding triage: gated on at least one wizard/{distro}/transcript.md reaching SCENARIO_A: ok|failed:*. When unblocked, read all four transcripts' ## Findings sections, group by file/severity, and file as M012 candidate work. Four-distro re-verification is human-bound.

## Files Created/Modified

- `src/ui/wizard.rs` — 
- `src/ui/tray.rs` — 
- `src/bin/vibe-attack-config.rs` — 
