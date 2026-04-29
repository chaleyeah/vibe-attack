---
estimated_steps: 11
estimated_files: 2
skills_used: []
---

# T02: Fix tray Quit + tooltip and add active-mode awareness

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

## Inputs

- `src/ui/tray.rs`
- `src/bin/vibe-attack-config.rs`
- `.gsd/milestones/M011/slices/S03/S03-RESEARCH.md`

## Expected Output

- `src/ui/tray.rs`
- `src/bin/vibe-attack-config.rs`

## Verification

cargo test --features gui --lib ui::tray:: -- --test-threads=1 && cargo build --release --features gui --bin vibe-attack-config

## Observability Impact

tracing::info!("Tray quit requested") emitted at the moment the user clicks Quit — correlates clean shutdown with the menu action in journald/stderr. The new free function tooltip_description_for is fully unit-testable without D-Bus, mirroring icon_name_for_state and reducing the dark area where 'tooltip says wrong thing' bugs hide.
