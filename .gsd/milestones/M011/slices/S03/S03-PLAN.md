# S03: UI polish from proof-run findings

**Goal:** Ship the UI polish items identified in S03-RESEARCH from code inspection of wizard.rs, vibe-attack-config.rs, and tray.rs — five concrete pre-existing UX/correctness bugs whose root cause is visible in the source without requiring VM runs. Defer the wizard-transcript-findings triage to a post-VM-run follow-up since all four wizard transcripts are still STATUS: pending VM run.

Demo (proven in auto-mode): cargo build --release --features gui --bin vibe-attack-config compiles cleanly; cargo test --features gui passes including new unit tests covering (a) PttCaptureState carries manual_key across frames, (b) tray tooltip description varies with active_mode, (c) tray TrayHandle exposes a shutdown flag and takes a quit request without process::exit. Manual smoke on dev host: vibe-attack-config --skip-wizard launches, main config screen renders the new "(configured in wizard)" affordance next to the PTT key row, and tray Quit cleanly closes the window via the shutdown flag (no abrupt termination).

Demo (deferred to follow-up, not blocking): VM-run-findings task and four-distro re-verification — gated on at least one wizard transcript reaching SCENARIO_A: ok|failed:* (currently all pending). Tracked as Follow-up; not in this slice's task list.
**Demo:** wizard flow, config screen, and tray menu issues found during VM runs are fixed; changes verified in the four distro environments.

## Must-Haves

- All five code-visible bugs from S03-RESEARCH are fixed: (1) manual_key frame-local reset in show_configure_ptt; (2) DownloadStatus::Done auto-advance gap in show_install_model; (3) tray Quit using std::process::exit(0); (4) tray tooltip ignoring active_mode; (5) uinput-step YELLOW warning legibility.
- New unit tests cover the manual_key state-carry behavior, the tooltip-by-mode mapping, and the tray shutdown-flag take/clear semantics.
- HuggingFace-redirect failure message in DownloadStatus::Failed gains a friendlier hint (research item, low-risk one-liner).
- cargo test --features gui and cargo test --lib pass on the dev host.
- Manual smoke confirms wizard launches, main config screen shows the new PTT-key affordance, and tray Quit closes cleanly.
- Wizard-finding-driven items are documented as a Follow-up in S03-SUMMARY (they cannot be planned now — there are no findings yet).

## Proof Level

- This slice proves: Operational on dev host (Ubuntu 26.04). Real runtime required: yes (cargo build/test + manual launch). Human/UAT required for the deferred VM-finding triage: yes — but that work is a Follow-up, not part of this slice. Four-distro re-verification is human-bound and tracked as a Follow-up.

## Integration Closure

- Upstream surfaces consumed: src/ui/wizard.rs (PttCaptureState, ModelDownloadState, show_configure_ptt, show_install_model, show_setup_uinput), src/ui/tray.rs (TrayHandle, VibeTray::tool_tip, VibeTray::menu Quit item), src/bin/vibe-attack-config.rs (eframe ui loop polling tray.take_open_request).
- New wiring introduced: TrayHandle gains a quit AtomicBool flag exposed via take_quit_request(); the eframe ui loop polls it alongside take_open_request and calls ctx.send_viewport_cmd(ViewportCommand::Close) when set. PttCaptureState gains a manual_key: String field; show_configure_ptt borrows &mut ptt.manual_key.
- What remains before milestone is usable end-to-end: S04 (release CI + AppImage publish) and S05 (Publish v1.0.0). Plus the deferred wizard-finding-triage task (post-VM-run). None of these block M011 closure beyond their own slice gates.

## Verification

- Runtime signals: existing tracing::info/error in wizard.rs (download/PTT/uinput) and tray.rs (D-Bus warn) remain unchanged. New tray Quit path emits tracing::info!("Tray quit requested") so a future agent can correlate clean shutdown with the quit menu item.
- Inspection surfaces: TrayHandle::take_open_request() pattern is mirrored by TrayHandle::take_quit_request() — both readable from test code without D-Bus.
- Failure visibility: Wizard download Failed branch keeps the raw ureq error and prepends a hint line so logs and the UI both contain the underlying string. PTT manual entry now persists across frames so a future user-reported "I typed but nothing happened" can be ruled out by reading the field in PttCaptureState.
- Redaction constraints: none — no secrets touched.

## Tasks

- [x] **T01: Fix wizard usability bugs (manual_key state, install-model auto-advance, uinput note legibility, download error hint)** `est:2h`
  Address the four wizard-side bugs identified in S03-RESEARCH that are visible from code inspection of src/ui/wizard.rs.

(1) manual_key frame-local reset — show_configure_ptt currently uses `let mut manual_key = String::new()` inside the panel function (wizard.rs:653), resetting to empty every frame so the user cannot type. Add a `manual_key: String` field to PttCaptureState (alongside listening, captured_key, handle, error). Update PttCaptureState::new() and Default to initialize it to String::new(). In show_configure_ptt, replace the local with `&mut ptt.manual_key`. After a successful manual save, clear `ptt.manual_key`.

(2) Install-model auto-advance — when the wizard re-renders show_install_model with DownloadStatus::Done but the wizard step is still InstallModel (because the user re-entered the wizard after the download handle was reaped on a previous run), the panel sits on a 'Re-check' button. Inside the DownloadStatus::Done arm of show_install_model (currently wizard.rs:384–390), call `*state = probe::run()` once before rendering the button. This is idempotent — probe::run() reads the filesystem so re-entering the panel re-detects the file and advances steps automatically. Keep the manual 'Re-check' button as a fallback.

(3) Uinput note legibility — replace the bare `egui::Color32::YELLOW` colored_label at wizard.rs:570–573 with a visually-bounded warning: wrap the note text in an `egui::Frame` with a subtle background fill (egui::Color32::from_rgb(64, 50, 0) or similar dark-amber) and use `egui::Color32::from_rgb(255, 200, 60)` for the text. Pattern matches the existing copy_command_row Frame usage (wizard.rs:604) so no new egui APIs are needed.

(4) HuggingFace redirect hint — in download_model's failure path (wizard.rs ~423), when ureq returns an error, prepend a one-line hint to the failure message: 'HuggingFace serves a 302 redirect to a CDN — if your network blocks the CDN this will fail.' Concatenate before the raw error so both are visible.

Unit tests:
- Add `tests::manual_key_persists_in_state` to wizard.rs `#[cfg(test)] mod tests`: construct PttCaptureState::new(), mutate ptt.manual_key.push_str('KEY_F13'), assert ptt.manual_key == 'KEY_F13'. This is a state round-trip test (matches MEM083 pattern of testing wizard predicates without GUI).
- Add `tests::manual_key_default_empty`: PttCaptureState::default().manual_key.is_empty().

Must stay inside the `mod inner` block (wizard.rs:14) since PttCaptureState is exported via `pub use inner::*` (line 11). Do not add fields outside `inner`.
  - Files: `src/ui/wizard.rs`
  - Verify: cargo test --features gui --lib ui::wizard:: -- --test-threads=1 && cargo build --release --features gui --bin vibe-attack-config

- [x] **T02: Fix tray Quit + tooltip and add active-mode awareness** `est:2h`
  Address the two tray-side bugs identified in S03-RESEARCH.

(1) Tray Quit using std::process::exit(0) — the current Quit handler at src/ui/tray.rs:336 is `Box::new(|_| std::process::exit(0))`. This bypasses eframe shutdown and can drop pending writes (per S03-RESEARCH and MEM042 — ksni callbacks must be non-blocking and must not jump out of the runtime). Add a new `quit_window: Arc<AtomicBool>` field to TrayHandle (line 33) and to VibeTray (line 157), initialize alongside open_window in TrayHandle::spawn() and the VibeTray construction (lines 44–66). Add a public method `pub fn take_quit_request(&self) -> bool { self.quit_window.swap(false, Ordering::AcqRel) }` mirroring take_open_request (line 135). In the Quit menu item (line 332–340), replace the `std::process::exit(0)` closure with `Box::new(move |_this: &mut Self| { tracing::info!("Tray quit requested"); quit_flag.store(true, Ordering::Release); })` — capture an `Arc::clone(&self.quit_window)` into the closure as `quit_flag`, mirroring the open_flag pattern at line 204.

In src/bin/vibe-attack-config.rs (in `impl eframe::App for VibeAttackConfigApp::ui` at line 240, immediately after the existing `take_open_request` block at line 244), add: `if self.tray.as_ref().is_some_and(|t| t.take_quit_request()) { ctx.send_viewport_cmd(egui::ViewportCommand::Close); return; }`. The early return prevents the rest of the frame from drawing after a close request.

(2) Tooltip ignores active_mode — VibeTray::tool_tip (tray.rs:184–197) currently maps DaemonState::Idle to the literal 'Idle — listening for wake word', which is wrong in PTT mode. Change the Idle arm to read self.current_state().active_mode and produce: PTT → 'Idle — waiting for PTT key'; Wake → 'Idle — listening for wake word'; None → 'Idle'. Extract this into a free pub(crate) function `tooltip_description_for(state: Option<&DaemonState>, mode: Option<&ActivationMode>) -> String` (mirrors icon_name_for_state at line 147 — MEM045 convention) so it is unit-testable without D-Bus. Use the new function from tool_tip().

Unit tests in tray.rs `#[cfg(test)] mod tests` at line 346:
- `tooltip_description_idle_ptt` → tooltip_description_for(Some(&DaemonState::Idle), Some(&ActivationMode::Ptt)) contains 'PTT'.
- `tooltip_description_idle_wake` → contains 'wake word'.
- `tooltip_description_idle_unknown` → equals 'Idle' when mode is None.
- `tooltip_description_recording_unaffected_by_mode` → Recording variant returns the same string regardless of mode.
- `tray_handle_take_quit_request_clears_flag` → construct an `Arc<AtomicBool>::new(true)`, call swap(false, AcqRel), assert returned true and the flag is now false. (Tests the take pattern without spawning a real tray, matching MEM045 free-function-test convention.)

New wiring inside vibe-attack-config.rs is two lines plus the early return; do not refactor the surrounding ui() function.
  - Files: `src/ui/tray.rs`, `src/bin/vibe-attack-config.rs`
  - Verify: cargo test --features gui --lib ui::tray:: -- --test-threads=1 && cargo build --release --features gui --bin vibe-attack-config

- [x] **T03: Add config-screen PTT-key affordance and run dev-host smoke verification** `est:1h`
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
  - Files: `src/bin/vibe-attack-config.rs`
  - Verify: cargo build --release --features gui --bin vibe-attack-config && cargo test --features gui --lib && cargo test --test distribution_proofs -- --test-threads=1 && cargo test --test wizard_proofs -- --test-threads=1

## Files Likely Touched

- src/ui/wizard.rs
- src/ui/tray.rs
- src/bin/vibe-attack-config.rs
